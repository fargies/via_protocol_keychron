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

use via_protocol::{ViaError, ViaResult};

use crate::{VKCommandMaker, VKMiscCommandId, ViaKeychronProtocol, ViaReportData};

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

impl TryFrom<u8> for VKSnapClickType {
    type Error = ViaError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > VKSnapClickType::Neutral as u8 {
            Err(ViaError::Protocol(format!(
                "unknown VKSnapClickType: {value}"
            )))
        } else {
            Ok(unsafe { std::mem::transmute::<u8, VKSnapClickType>(value) })
        }
    }
}

#[derive(Debug)]
pub struct VKSnapClick {
    pub snap_type: VKSnapClickType,
    pub keycode: [u8; 2],
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
    pub fn load(start: u8, count: u8, proto: &ViaKeychronProtocol) -> ViaResult<Self> {
        proto.get_snap_click(start, count)
    }

    pub fn count(proto: &ViaKeychronProtocol) -> ViaResult<u8> {
        proto.get_snap_click_info()
    }

    pub fn send(&self, proto: &ViaKeychronProtocol) -> ViaResult<()> {
        proto.set_snap_click(self)
    }

    pub fn save(&self, proto: &ViaKeychronProtocol) -> ViaResult<()> {
        proto.save_snap_click()
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

pub trait VKSnapClickTrait {
    /// @returns Supported snap_click count
    fn get_snap_click_info(&self) -> ViaResult<u8>;

    /// get snap_click info
    ///
    /// @params [count] must be <= 9
    /// start + count must be < `SNAP_CLICK_COUNT` as returned by [get_snap_click_info]
    fn get_snap_click(&self, start: u8, count: u8) -> ViaResult<VKSnapClickConfig>;

    fn set_snap_click(&self, config: &VKSnapClickConfig) -> ViaResult<()>;

    fn save_snap_click(&self) -> ViaResult<()>;
}

impl VKSnapClickTrait for ViaKeychronProtocol<'_> {
    /// @returns Supported snap_click count
    fn get_snap_click_info(&self) -> ViaResult<u8> {
        let cmd = &VKMiscCommandId::SnapClickGetInfo;
        let resp = self.device.raw_hid_send(&cmd.to_cmd())?;
        Ok(cmd.check_reply(&resp)?[0])
    }

    /// get snap_click info
    ///
    /// @params [count] must be <= 9
    /// start + count must be < `SNAP_CLICK_COUNT` as returned by [get_snap_click_info]
    fn get_snap_click(&self, start: u8, count: u8) -> ViaResult<VKSnapClickConfig> {
        let cmd = &VKMiscCommandId::SnapClickGet;
        let resp = self.device.raw_hid_send(&cmd.to_req(&[start, count]))?;
        let mut ret = VKSnapClickConfig::try_from(resp)?;
        ret.start = start;
        Ok(ret)
    }

    fn set_snap_click(&self, config: &VKSnapClickConfig) -> ViaResult<()> {
        let cmd = &VKMiscCommandId::SnapClickSet;
        if config.config.len() > 9 {
            return Err(ViaError::Protocol(format!(
                "too many VKSnapClickConfig, (max=9): {}",
                config.config.len()
            )));
        }
        let mut data = Vec::with_capacity(2 + 3 * config.config.len());
        data.push(config.start);
        data.push(config.config.len() as u8);
        for cfg in config.config.iter() {
            data.push(cfg.snap_type as u8);
            data.push(cfg.keycode[0]);
            data.push(cfg.keycode[1]);
        }
        let resp = self.device.raw_hid_send(&cmd.to_req(data.as_ref()))?;
        cmd.check_reply(&resp)?;
        Ok(())
    }

    fn save_snap_click(&self) -> ViaResult<()> {
        let cmd = &VKMiscCommandId::SnapClickSave;
        let resp = self.device.raw_hid_send(&cmd.to_cmd())?;
        cmd.check_reply(&resp)?;
        Ok(())
    }
}
