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

use crate::{VKCommandMaker, ViaKeychronProtocol};

use super::{VKRgbCommandId, VKRgbMixedInfo, VKRgbProtocolVersion};

#[derive(Debug, Clone)]
pub struct VKRgbInfo {
    pub protocol_version: Arc<VKRgbProtocolVersion>,
    pub mixed: Arc<VKRgbMixedInfo>,
    pub led_count: usize,
}

impl VKRgbInfo {
    pub fn load(proto: &ViaKeychronProtocol) -> ViaResult<Arc<Self>> {
        if let Some(value) = proto.get_info().rgb.as_ref() {
            Ok(value.clone())
        } else {
            let protocol_version = Arc::new({
                let cmd = &VKRgbCommandId::RgbGetProtocolVer;
                let resp = proto.raw_send(&cmd.to_cmd())?;
                VKRgbProtocolVersion::try_from(resp)?
            });
            let mixed = Arc::new({
                let cmd = &VKRgbCommandId::MixedEffectRgbGetInfo;
                let resp = proto.raw_send(&cmd.to_cmd())?;
                VKRgbMixedInfo::try_from(resp)?
            });
            let led_count = {
                let cmd = &VKRgbCommandId::RgbGetLedCount;
                let resp = proto.raw_send(&cmd.to_cmd())?;
                cmd.check_reply(&resp)?[0] as usize
            };
            let ret = Arc::new(Self {
                protocol_version,
                mixed,
                led_count,
            });
            Arc::make_mut(&mut proto.get_info_mut())
                .rgb
                .replace(Arc::clone(&ret));
            Ok(ret)
        }
    }
}
