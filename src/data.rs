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

use bitflags::bitflags;
use via_protocol::{VIA_REPORT_SIZE, ViaError};

use crate::VKCommandId;

pub type ViaReportData = [u8; VIA_REPORT_SIZE];

#[derive(Debug)]
pub struct VKProtocolVersion {
    pub protocol_version: u8,
    pub qmk_version: u8,
}

impl TryFrom<&ViaReportData> for VKProtocolVersion {
    type Error = ViaError;

    fn try_from(value: &ViaReportData) -> Result<Self, Self::Error> {
        let value = VKCommandId::GetProtocolVersion.check_reply(value)?;
        Ok(VKProtocolVersion {
            protocol_version: value[0],
            qmk_version: value[2],
        })
    }
}

bitflags! {
    #[derive(Debug)]
    pub struct VKFeatures: u16 {
        const DEFAULT_LAYER    = 0b1;
        const BLUETOOTH        = 0b10;
        const P24G             = 0b100;
        const ANALOG_MATRIX    = 0b1000;
        const STATE_NOTIFY     = 0b1_0000;
        const DYNAMIC_DEBOUNCE = 0b10_0000;
        const SNAP_CLICK       = 0b100_0000;
        const KEYCHRON_RGB     = 0b1000_0000;
        const QUICK_START      = 0b1_0000_0000;
        const NKRO             = 0b10_0000_0000;
    }
}

bitflags! {
    #[derive(Debug)]
    pub struct VKMiscFeatures: u8 {
        const MISC_DFU_INFO            = 0b1;
        const MISC_LANGUAGE            = 0b10;
        const MISC_DEBOUNCE            = 0b100;
        const MISC_SNAP_CLICK          = 0b1000;
        const MISC_WIRELESS_LPM        = 0b1_0000;
        const MISC_REPORT_REATE        = 0b10_0000;
        const MISC_QUICK_START         = 0b100_0000;
        const MISC_NKRO                = 0b1000_0000;
    }
}
