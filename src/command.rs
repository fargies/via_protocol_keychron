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

use crate::ViaReportData;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum VKCommandId {
    GetProtocolVersion = 0xA0,
    GetFirmwareVersion = 0xA1,
    GetSupportFeature = 0xA2,
    GetDefaultLayer = 0xA3,
    MiscCmdGroup = 0xA7,
    KeychronRgb = 0xA8,
    AnalogMatrix = 0xA9,
    WirelessDfu = 0xAA,
    FactoryTest = 0xAB,
}

impl VKCommandId {
    /// Convenience function, checks reply returns payload if properly built
    pub fn check_reply<'a>(&self, value: &'a ViaReportData) -> ViaResult<&'a [u8]> {
        if value[0] != *self as u8 {
            Err(ViaError::Protocol(format!(
                "invalid {self:?} packet, corrupted cmd: {}",
                value[0]
            )))
        } else {
            Ok(&value[1..])
        }
    }
}

pub trait VKCommandMaker {
    fn to_cmd(self) -> VKCommand;
    fn to_req(self, data: &[u8]) -> VKCommand;
}

impl VKCommandMaker for VKCommandId {
    fn to_cmd(self) -> VKCommand {
        let mut report = [0u8; via_protocol::VIA_REPORT_SIZE + 1];
        report[0] = 0x00;
        report[1] = self as u8;
        VKCommand { report }
    }

    fn to_req(self, data: &[u8]) -> VKCommand {
        let mut ret = Self::to_cmd(self);
        let copy_len = data.len().min(via_protocol::VIA_REPORT_SIZE - 1);
        ret.report[2..2 + copy_len].copy_from_slice(&data[..copy_len]);
        ret
    }
}

pub struct VKCommand {
    pub report: [u8; via_protocol::VIA_REPORT_SIZE + 1],
}

impl std::ops::Deref for VKCommand {
    type Target = [u8; via_protocol::VIA_REPORT_SIZE + 1];

    fn deref(&self) -> &Self::Target {
        &self.report
    }
}
