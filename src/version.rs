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

use crate::{VKCommandId, VKCommandMaker, ViaKeychronProtocol, ViaReportData};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum VKProtocolType {
    Zmk = 1,
    Qmk = 2,
}

impl TryFrom<u8> for VKProtocolType {
    type Error = ViaError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(VKProtocolType::Zmk),
            2 => Ok(VKProtocolType::Qmk),
            _ => Err(ViaError::Protocol(format!(
                "invalid VKProtocolType: {value}"
            ))),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct VKProtocolVersion {
    pub protocol: VKProtocolType,
    pub version: u8,
}

impl VKProtocolVersion {
    pub fn load(proto: &ViaKeychronProtocol) -> ViaResult<Self> {
        proto.get_protocol_version()
    }
}

impl TryFrom<&ViaReportData> for VKProtocolVersion {
    type Error = ViaError;

    fn try_from(value: &ViaReportData) -> Result<Self, Self::Error> {
        let value = VKCommandId::GetProtocolVersion.check_reply(value)?;
        Ok(VKProtocolVersion {
            protocol: VKProtocolType::try_from(value[0])?,
            version: value[2],
        })
    }
}

pub trait VKProtocolVersionTrait {
    fn get_protocol_version(&self) -> ViaResult<VKProtocolVersion>;
}

impl VKProtocolVersionTrait for ViaKeychronProtocol<'_> {
    fn get_protocol_version(&self) -> ViaResult<VKProtocolVersion> {
        if let Some(proto) = *self.protocol.lock().unwrap() {
            Ok(proto)
        } else {
            VKProtocolVersion::try_from(
                &self
                    .device
                    .raw_hid_send(&VKCommandId::GetProtocolVersion.to_cmd())?,
            )
            .inspect(|proto| {
                self.protocol.lock().unwrap().replace(*proto);
            })
        }
    }
}
