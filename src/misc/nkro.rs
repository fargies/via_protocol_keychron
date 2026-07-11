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

#[derive(Debug)]
pub struct VKNkroConfig {
    data: Vec<u8>,
}

impl VKNkroConfig {
    pub fn load(proto: &ViaKeychronProtocol) -> ViaResult<Self> {
        proto.get_nkro()
    }

    pub fn send(&self, proto: &ViaKeychronProtocol) -> ViaResult<()> {
        proto.set_nkro(self.is_enabled())
    }

    pub fn is_enabled(&self) -> bool {
        (self.data[0] & 0b1) != 0
    }

    pub fn set_enabled(&mut self, value: bool) {
        if value {
            self.data[0] |= 0b1;
        } else {
            self.data[0] &= !0b1;
        }
    }

    pub fn is_available(&self) -> bool {
        (self.data[0] & 0b10) != 0
    }

    pub fn is_adaptive(&self) -> bool {
        (self.data[0] & 0b100) != 0
    }
}

impl std::fmt::Display for VKNkroConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VKNkroConfig")
            .field("enabled", &self.is_enabled())
            .field("available", &self.is_enabled())
            .field("adaptive", &self.is_enabled())
            .finish()
    }
}

impl TryFrom<ViaReportData> for VKNkroConfig {
    type Error = ViaError;

    fn try_from(value: ViaReportData) -> Result<Self, Self::Error> {
        let payload = VKMiscCommandId::NkroGet.check_reply(&value)?;
        Ok(VKNkroConfig {
            data: payload.into(),
        })
    }
}

pub trait VKNkroTrait {
    fn get_nkro(&self) -> ViaResult<VKNkroConfig>;
    fn set_nkro(&self, value: bool) -> ViaResult<()>;
}

impl VKNkroTrait for ViaKeychronProtocol<'_> {
    fn get_nkro(&self) -> ViaResult<VKNkroConfig> {
        let cmd = &VKMiscCommandId::NkroGet;
        let resp = self.device.raw_hid_send(&cmd.to_cmd())?;
        VKNkroConfig::try_from(resp)
    }

    fn set_nkro(&self, value: bool) -> ViaResult<()> {
        let cmd = &VKMiscCommandId::NkroSet;
        let req = cmd.to_req(&[if value { 1 } else { 0 }]);

        let resp = self.device.raw_hid_send(&req)?;
        cmd.check_reply(&resp)?;
        Ok(())
    }
}
