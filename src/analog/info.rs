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
    VKAnalogCommandId, VKAnalogProfileInfo, VKAnalogProtocolVersion, VKCommandMaker,
    ViaKeychronProtocol,
};

#[derive(Debug, Clone)]
pub struct VKAnalogInfo {
    pub protocol_version: Arc<VKAnalogProtocolVersion>,
    pub profile_info: Arc<VKAnalogProfileInfo>,
}

impl VKAnalogInfo {
    pub fn load(proto: &ViaKeychronProtocol) -> ViaResult<Arc<Self>> {
        if let Some(value) = proto.get_info().analog.as_ref() {
            Ok(value.clone())
        } else {
            let protocol_version = Arc::new({
                let cmd = &VKAnalogCommandId::GetProtocolVersion;
                let resp = proto.raw_send(&cmd.to_cmd())?;
                VKAnalogProtocolVersion::try_from(resp)?
            });
            let profile_info = Arc::new({
                let resp = proto.raw_send(&VKAnalogCommandId::GetProfilesInfo.to_cmd())?;
                let mut profile_info = VKAnalogProfileInfo::try_from(resp)?;

                profile_info.load_key_count(proto)?;
                profile_info
            });
            let info = Arc::new(Self {
                protocol_version,
                profile_info,
            });
            Arc::make_mut(&mut proto.get_info_mut())
                .analog
                .replace(info.clone());
            Ok(info)
        }
    }
}
