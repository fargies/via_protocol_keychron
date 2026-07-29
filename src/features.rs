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

use std::sync::Arc;

use bitflags::bitflags;
use via_protocol::ViaResult;

use crate::{
    VKCommandId, VKCommandMaker, VKProtocolVersion, ViaKeychronProtocol, version::VKProtocolType,
};

bitflags! {
    /// @brief features supported by the Keychron keyboard
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub struct VKFeatures: u16 {
        /// @brief default layer is supported
        const DEFAULT_LAYER    = 0b1;
        /// @brief bluetooth is supported
        const BLUETOOTH        = 0b10;
        /// @brief P24G is supported
        const P24G             = 0b100;
        /// @brief analog matrix is supported
        const ANALOG_MATRIX    = 0b1000;
        /// @brief state notify is supported
        const STATE_NOTIFY     = 0b1_0000;
        /// @brief dynamic debounce is supported
        const DYNAMIC_DEBOUNCE = 0b10_0000;
        /// @brief snap click is supported
        const SNAP_CLICK       = 0b100_0000;
        /// @brief keychron RGB is supported
        const KEYCHRON_RGB     = 0b1000_0000;
        /// @brief quick start is supported
        const QUICK_START      = 0b1_0000_0000;
        /// @brief NKRO is supported
        const NKRO             = 0b10_0000_0000;
    }
}

impl VKFeatures {
    /// @brief loads the features from the device
    pub fn load(proto: &ViaKeychronProtocol) -> ViaResult<Arc<Self>> {
        if let Some(value) = proto.get_info().features.as_ref() {
            Ok(value.clone())
        } else {
            let cmd = &VKCommandId::GetSupportFeature;
            let resp = proto.device.raw_hid_send(&cmd.to_cmd())?;

            let payload = cmd.check_reply(&resp)?;
            let features = match VKProtocolVersion::load(proto)?.protocol {
                VKProtocolType::Zmk => [payload[0], payload[1]],
                VKProtocolType::Qmk => [payload[1], payload[2]],
            };
            let ret = Arc::new(VKFeatures::from_bits_retain(u16::from_le_bytes(features)));
            Arc::make_mut(&mut proto.get_info_mut())
                .features
                .replace(ret.clone());
            Ok(ret)
        }
    }
}
