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

use std::fmt::Display;

use via_protocol::{ViaError, ViaResult};

use crate::{VKCommandMaker, VKMiscCommandId, ViaKeychronProtocol, ViaReportData};

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[repr(u8)]
pub enum VKDebounceType {
    SymDeferGlobal = 0,
    SymDeferPerRow = 1,
    SymDeferPerKey = 2,
    SymEagerPerRow = 3,
    SymEagerPerKey = 4,
    AsymEagerDeferPerKey = 5,
    None = 6,
    Max = 7,
}

impl TryFrom<u8> for VKDebounceType {
    type Error = ViaError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value <= VKDebounceType::Max as u8 {
            Ok(unsafe { std::mem::transmute::<u8, VKDebounceType>(value) })
        } else {
            Err(ViaError::Protocol(format!(
                "invalid VKDebounceType: {value}"
            )))
        }
    }
}

#[derive(Debug)]
pub struct VKDebounceConfig {
    data: Vec<u8>,
}

impl VKDebounceConfig {
    pub fn load(proto: &ViaKeychronProtocol) -> ViaResult<Self> {
        let cmd = &VKMiscCommandId::DebounceGet;
        proto.device.raw_hid_send(&cmd.to_cmd()).and_then(Self::try_from)
    }

    pub fn send(&self, proto: &ViaKeychronProtocol) -> ViaResult<()> {
        let cmd = &VKMiscCommandId::DebounceSet;
        let resp = proto
            .device
            .raw_hid_send(&cmd.to_req(&self.data))?;

        cmd.check_reply(&resp)?;
        Ok(())
    }

    pub fn get_type(&self) -> VKDebounceType {
        VKDebounceType::try_from(self.data[0]).unwrap()
    }

    pub fn set_type(&mut self, value: VKDebounceType) {
        self.data[0] = value as u8;
    }

    pub fn get_time(&self) -> u8 {
        self.data[1]
    }

    pub fn set_time(&mut self, value: u8) {
        self.data[1] = value;
    }
}

impl Display for VKDebounceConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VKDebounceConfig")
            .field("type", &self.get_type())
            .field("time", &self.get_time())
            .finish()
    }
}

impl TryFrom<ViaReportData> for VKDebounceConfig {
    type Error = ViaError;

    fn try_from(value: ViaReportData) -> Result<Self, Self::Error> {
        let payload = VKMiscCommandId::DebounceGet.check_reply(&value)?;
        VKDebounceType::try_from(payload[0])?;
        Ok(VKDebounceConfig { data: payload.into() })
    }
}

#[cfg(test)]
mod tests {
    use via_protocol::ViaResult;

    use super::*;

    #[test]
    fn parse() -> ViaResult<()> {
        assert_eq!(VKDebounceType::try_from(2)?, VKDebounceType::SymDeferPerKey);
        VKDebounceType::try_from(42).expect_err("should fail to convert");
        Ok(())
    }
}
