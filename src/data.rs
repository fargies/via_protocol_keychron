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
use via_protocol::VIA_REPORT_SIZE;

pub type ViaReportData = [u8; VIA_REPORT_SIZE];

bitflags! {
    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    pub struct VKMiscFeatures: u8 {
        const DFU_INFO            = 0b1;
        const LANGUAGE            = 0b10;
        const DEBOUNCE            = 0b100;
        const SNAP_CLICK          = 0b1000;
        const WIRELESS_LPM        = 0b1_0000;
        const REPORT_RATE         = 0b10_0000;
        const QUICK_START         = 0b100_0000;
        const NKRO                = 0b1000_0000;
    }
}
