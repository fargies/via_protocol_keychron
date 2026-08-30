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

/// @brief Represents the data of a VIA report
pub type ViaReportData = [u8; VIA_REPORT_SIZE];

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

bitflags! {
    /// @brief Represents the miscellaneous features of the device
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    pub struct VKMiscFeatures: u8 {
        /// @brief DFU info is available
        const DFU_INFO            = 0b1;
        /// @brief Language is available
        const LANGUAGE            = 0b10;
        /// @brief Debounce is available
        const DEBOUNCE            = 0b100;
        /// @brief Snap click is available
        const SNAP_CLICK          = 0b1000;
        /// @brief Wireless LPM is available
        const WIRELESS_LPM        = 0b1_0000;
        /// @brief Report rate is available
        const REPORT_RATE         = 0b10_0000;
        /// @brief Quick start is available
        const QUICK_START         = 0b100_0000;
        /// @brief NKRO is available
        const NKRO                = 0b1000_0000;
    }
}
