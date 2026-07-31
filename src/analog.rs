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

use crate::{VKCommand, VKCommandId, VKCommandMaker, ViaReportData};

mod version;
pub use version::*;

mod profile;
pub use profile::*;

mod key_config;
pub use key_config::*;

/// @brief analog command IDs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum VKAnalogCommandId {
    GetProtocolVersion = 0x01,
    GetProfilesInfo = 0x10,
    SelectProfile = 0x11,
    GetProfileRaw = 0x12,
    SetProfileName = 0x13,
    SetTraval = 0x14,
    SetAdvancedMode = 0x15,
    SetSOCD = 0x16,
    ResetProfile = 0x1E,
    SaveProfile = 0x1F,
    GetCurve = 0x20,
    SetCurve = 0x21,
    GetGameControllerMode = 0x22,
    SetGameControllerMode = 0x23,
    GetRealtimeTraval = 0x30,
    Calibrate = 0x40,
    GetCalibrateState = 0x41,
    GetCalibratedValue = 0x42,
}

impl VKAnalogCommandId {
    /// @brief size of the analog command header in bytes
    pub const HEADER_BYTE_SIZE: usize = 2;

    /// @brief checks the reply packet for the given analog command
    #[tracing::instrument(level = "ERROR", err)]
    pub fn check_reply<'a>(&self, value: &'a ViaReportData) -> ViaResult<&'a [u8]> {
        if value[0] != VKCommandId::AnalogMatrix as u8 {
            Err(ViaError::Protocol(format!(
                "invalid {self:?} packet, corrupted cmd: {}",
                value[0]
            )))
        } else if value[1] != *self as u8 {
            Err(ViaError::Protocol(format!(
                "invalid {self:?} packet, corrupted sub-cmd: {}",
                value[1]
            )))
        } else {
            Ok(&value[2..])
        }
    }
}

impl VKCommandMaker for VKAnalogCommandId {
    fn to_cmd(self) -> VKCommand {
        let mut report = [0u8; via_protocol::VIA_REPORT_SIZE + 1];
        report[0] = 0x00;
        report[1] = VKCommandId::AnalogMatrix as u8;
        report[2] = self as u8;
        VKCommand { report }
    }

    fn to_req(self, data: &[u8]) -> VKCommand {
        let mut ret = Self::to_cmd(self);
        let copy_len = data.len().min(via_protocol::VIA_REPORT_SIZE - 2);
        ret.report[3..3 + copy_len].copy_from_slice(&data[..copy_len]);
        ret
    }
}
