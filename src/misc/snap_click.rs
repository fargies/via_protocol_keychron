/*
** Copyright (C) 2025 Sylvain Fargier
**
** This software is provided 'as-is', without any express or implied
** warranty.  In no event will the authors be held liable for any damages
** arising from the use of this software.
**
** Permission is granted to anyone to use this software for any purpose,
** including commercial applications, and to alter it and redistribute it
** freely, subject to the following restrictions:
**
** 1. The origin of this software must not be misrepresented; you must not
**    claim that you wrote the original software. If you use this software
**    in a product, an acknowledgment in the product documentation would be
**    appreciated but is not required.
** 2. Altered source versions must be plainly marked as such, and must not be
**    misrepresented as being the original software.
** 3. This notice may not be removed or altered from any source distribution.
**
** Author: Sylvain Fargier <fargier.sylvain@gmail.com>
*/

use via_protocol::{VIA_REPORT_SIZE, ViaError, ViaResult};

use crate::{VKCommandMaker, VKMiscCommandId, ViaKeychronProtocol, ViaReportData};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[repr(u8)]
pub enum VKSnapClickType {
    None = 0,
    Regular = 1,
    LastInput = 2,
    FirstKey = 3,
    SecondKey = 4,
    Neutral = 5,
}

impl VKSnapClickType {
    pub fn iter() -> impl Iterator<Item = Self> {
        (0..).map_while(|i| Self::try_from(i).ok())
    }
}

impl TryFrom<u8> for VKSnapClickType {
    type Error = ViaError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value <= VKSnapClickType::Neutral as u8 {
            Ok(unsafe { std::mem::transmute::<u8, VKSnapClickType>(value) })
        } else {
            Err(ViaError::Protocol(format!(
                "unknown VKSnapClickType: {value}"
            )))
        }
    }
}

#[derive(Debug)]
pub struct VKSnapClick {
    pub snap_type: VKSnapClickType,
    pub keycode: [u8; 2],
}

impl VKSnapClick {
    pub const BYTE_SIZE: usize = 3;
}

impl TryFrom<&[u8]> for VKSnapClick {
    type Error = ViaError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < 3 {
            Err(ViaError::Protocol(format!(
                "corrupted VKSnapClickConfig, missing bytes: {value:?}"
            )))
        } else {
            Ok(VKSnapClick {
                snap_type: VKSnapClickType::try_from(value[0])?,
                keycode: [value[1], value[2]],
            })
        }
    }
}

#[derive(Debug)]
pub struct VKSnapClickConfig {
    pub start: u8,
    pub config: Vec<VKSnapClick>,
}

impl VKSnapClickConfig {
    pub const MAX_REQ_ITEMS: usize =
        (VIA_REPORT_SIZE - VKMiscCommandId::HEADER_BYTE_SIZE) / VKSnapClick::BYTE_SIZE;

    pub fn load(proto: &ViaKeychronProtocol) -> ViaResult<Self> {
        let sc_count = Self::count(proto)? as usize;
        let cmd = &VKMiscCommandId::SnapClickGet;

        let mut ret = Self {
            start: 0,
            config: Vec::with_capacity(sc_count),
        };

        let mut start = 0;
        while start < sc_count {
            let count = (sc_count - start).min(Self::MAX_REQ_ITEMS);
            let resp = proto.raw_send(&cmd.to_req(&[start as u8, count as u8]))?;
            let value = cmd.check_reply(&resp)?;
            for i in 0..count {
                ret.config
                    .push(VKSnapClick::try_from(&value[i * VKSnapClick::BYTE_SIZE..])?)
            }
            start += count;
        }
        Ok(ret)
    }

    pub fn load_part(proto: &ViaKeychronProtocol, start: u8, count: u8) -> ViaResult<Self> {
        let cmd = &VKMiscCommandId::SnapClickGet;
        let resp = proto.raw_send(&cmd.to_req(&[start, count]))?;
        let mut ret = VKSnapClickConfig::try_from(resp)?;
        ret.start = start;
        Ok(ret)
    }

    pub fn count(proto: &ViaKeychronProtocol) -> ViaResult<u8> {
        let cmd = &VKMiscCommandId::SnapClickGetInfo;
        let resp = proto.raw_send(&cmd.to_cmd())?;
        Ok(cmd.check_reply(&resp)?[0])
    }

    pub fn send(&self, proto: &ViaKeychronProtocol) -> ViaResult<()> {
        let cmd = &VKMiscCommandId::SnapClickSet;
        if self.config.len() > 9 {
            return Err(ViaError::Protocol(format!(
                "too many VKSnapClickConfig, (max=9): {}",
                self.config.len()
            )));
        }
        let mut data = Vec::with_capacity(2 + 3 * self.config.len());
        data.push(self.start);
        data.push(self.config.len() as u8);
        for cfg in self.config.iter() {
            data.push(cfg.snap_type as u8);
            data.push(cfg.keycode[0]);
            data.push(cfg.keycode[1]);
        }
        let resp = proto.raw_send(&cmd.to_req(data.as_ref()))?;
        cmd.check_reply(&resp)?;
        Ok(())
    }

    pub fn save(&self, proto: &ViaKeychronProtocol) -> ViaResult<()> {
        let cmd = &VKMiscCommandId::SnapClickSave;
        let resp = proto.raw_send(&cmd.to_cmd())?;
        cmd.check_reply(&resp)?;
        Ok(())
    }
}

impl TryFrom<ViaReportData> for VKSnapClickConfig {
    type Error = ViaError;

    fn try_from(value: ViaReportData) -> Result<Self, Self::Error> {
        let payload = VKMiscCommandId::SnapClickGet.check_reply(&value)?;
        let mut ret = Self {
            start: 0,
            config: Vec::with_capacity(payload.len() / 3),
        };
        for cfg in payload.chunks(3) {
            if cfg.len() == 3 {
                ret.config.push(VKSnapClick::try_from(cfg)?);
            }
        }
        Ok(ret)
    }
}

impl std::fmt::Display for VKSnapClickConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VKSnapClickConfig")
            .field("start", &self.start)
            .field("config", &self.config)
            .finish()
    }
}
