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

use crate::{VKCommandMaker, VKRgbCommandId, ViaKeychronProtocol, ViaReportData};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct VKRgbProtocolVersion {
    pub major: u8,
    pub minor: u8
}

impl VKRgbProtocolVersion {
    pub fn load(proto: &ViaKeychronProtocol) -> ViaResult<Self> {
        proto.get_rgb_protocol_version()
    }
}

impl TryFrom<ViaReportData> for VKRgbProtocolVersion {
    type Error = ViaError;

    fn try_from(value: ViaReportData) -> Result<Self, Self::Error> {
        let payload = VKRgbCommandId::RgbGetProtocolVer.check_reply(&value)?;
        Ok(Self { major: payload[0], minor: payload[1] })
    }
}

pub trait VKRgbProtocolVersionTrait {
    fn get_rgb_protocol_version(&self) -> ViaResult<VKRgbProtocolVersion>;
}

impl VKRgbProtocolVersionTrait for ViaKeychronProtocol<'_> {
    fn get_rgb_protocol_version(&self) -> ViaResult<VKRgbProtocolVersion> {
        let cmd = &VKRgbCommandId::RgbGetProtocolVer;
        let resp = self.device.raw_hid_send(&cmd.to_cmd())?;
        VKRgbProtocolVersion::try_from(resp)
    }
}
