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

#[derive(Debug)]
pub struct VKWirelessLpmConfig {
    data: Vec<u8>,
}

impl VKWirelessLpmConfig {
    pub fn load(proto: &ViaKeychronProtocol) -> ViaResult<Self> {
        proto.get_wireless_lpm()
    }

    pub fn send(&self, proto: &ViaKeychronProtocol) -> ViaResult<()> {
        proto.set_wireless_lpm(self)
    }

    pub fn get_backlit_disable_time(&self) -> u16 {
        u16::from_le_bytes([self.data[0], self.data[1]])
    }

    pub fn set_backlit_disable_time(&mut self, value: u16) {
        self.data[0..2].copy_from_slice(&value.to_le_bytes());
    }

    pub fn get_connected_idle_time(&self) -> u16 {
        u16::from_le_bytes([self.data[2], self.data[3]])
    }

    pub fn set_connected_idle_time(&mut self, value: u16) {
        self.data[2..3].copy_from_slice(&value.to_le_bytes());
    }
}

impl Display for VKWirelessLpmConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VKWirelessLpmConfig")
            .field("backlit_disable_time", &self.get_backlit_disable_time())
            .field("connected_idle_time", &self.get_connected_idle_time())
            .finish()
    }
}

impl TryFrom<ViaReportData> for VKWirelessLpmConfig {
    type Error = ViaError;

    fn try_from(value: ViaReportData) -> Result<Self, Self::Error> {
        let payload = VKMiscCommandId::WirelessLpmGet.check_reply(&value)?;
        Ok(VKWirelessLpmConfig {
            data: payload.into(),
        })
    }
}

pub trait VKWirelessLpmTrait {
    fn get_wireless_lpm(&self) -> ViaResult<VKWirelessLpmConfig>;
    fn set_wireless_lpm(&self, config: &VKWirelessLpmConfig) -> ViaResult<()>;
}

impl VKWirelessLpmTrait for ViaKeychronProtocol<'_> {
    fn get_wireless_lpm(&self) -> ViaResult<VKWirelessLpmConfig> {
        let cmd = &VKMiscCommandId::WirelessLpmGet;
        let resp = self.device.raw_hid_send(&cmd.to_cmd())?;
        VKWirelessLpmConfig::try_from(resp)
    }

    fn set_wireless_lpm(&self, config: &VKWirelessLpmConfig) -> ViaResult<()> {
        let cmd = &VKMiscCommandId::WirelessLpmSet;
        let req = cmd.to_req(config.data.as_ref());

        let resp = self.device.raw_hid_send(&req)?;
        cmd.check_reply(&resp)?;
        Ok(())
    }
}
