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

use via_protocol::ViaResult;

use crate::{
    VKCommandMaker, VKMiscCommandId, VKMiscFeatures, VKProtocolType, VKProtocolVersion,
    ViaKeychronProtocol,
};

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct VKMiscInfo {
    pub protocol_version: u16,
    pub features: VKMiscFeatures,
}

impl VKMiscInfo {
    pub fn load(proto: &ViaKeychronProtocol) -> ViaResult<Arc<Self>> {
        if let Some(value) = proto.get_info().misc.as_ref() {
            Ok(value.clone())
        } else {
            let cmd = &VKMiscCommandId::MiscGetProtocolVer;
            let resp = proto.device.raw_hid_send(&cmd.to_cmd())?;
            let payload = cmd.check_reply(&resp)?;
            let features = match VKProtocolVersion::load(proto)?.protocol {
                VKProtocolType::Zmk => VKMiscFeatures::DEBOUNCE,
                VKProtocolType::Qmk => VKMiscFeatures::from_bits_retain(payload[2]),
            };

            let ret = Arc::new(Self {
                protocol_version: u16::from_le_bytes([payload[0], payload[1]]),
                features,
            });
            Arc::make_mut(&mut proto.get_info_mut())
                .misc
                .replace(ret.clone());
            Ok(ret)
        }
    }
}
