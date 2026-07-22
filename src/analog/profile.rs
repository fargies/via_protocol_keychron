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

use via_protocol::{ViaError, ViaResult};

use crate::{VKAnalogCommandId, VKCommandMaker, ViaKeychronProtocol, ViaReportData};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct VKAnalogProfileInfo {
    data: Vec<u8>,
}

impl VKAnalogProfileInfo {
    pub fn load(proto: &ViaKeychronProtocol) -> ViaResult<Arc<Self>> {
        if let Some(info) = proto.get_info().analog_info.as_ref() {
            Ok(Arc::clone(info))
        } else {
            let cmd = &VKAnalogCommandId::GetProfilesInfo;
            let resp = proto.device.raw_hid_send(&cmd.to_cmd())?;
            Self::try_from(resp).map(Arc::new).inspect(|info| {
                Arc::make_mut(&mut proto.get_info_mut())
                    .analog_info
                    .replace(Arc::clone(info));
            })
        }
    }

    /// @brief get current profile index
    pub fn get_current_profile(&self) -> u8 {
        self.data[0]
    }

    /// @brief get number of profiles
    pub fn get_profile_count(&self) -> u8 {
        self.data[1]
    }

    /// @brief get profile size in bytes
    pub fn get_raw_profile_byte_size(&self) -> u16 {
        u16::from_le_bytes(self.data[2..4].try_into().unwrap())
    }

    /// @brief One Key Multiple Command count
    pub fn get_okmc_count(&self) -> u8 {
        self.data[4]
    }

    /// @brief Simultaneous Opposing Cardinal Directions count
    pub fn get_socd_count(&self) -> u8 {
        self.data[5]
    }
}

impl std::fmt::Display for VKAnalogProfileInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VKAnalogProfileInfo")
            .field("current", &self.get_current_profile())
            .field("count", &self.get_profile_count())
            .field("byte_size", &self.get_raw_profile_byte_size())
            .field("okmc_count", &self.get_okmc_count())
            .field("socd_count", &self.get_socd_count())
            .finish()
    }
}

impl TryFrom<ViaReportData> for VKAnalogProfileInfo {
    type Error = ViaError;

    fn try_from(value: ViaReportData) -> Result<Self, Self::Error> {
        let payload = VKAnalogCommandId::GetProfilesInfo.check_reply(&value)?;
        Ok(VKAnalogProfileInfo {
            data: payload.into(),
        })
    }
}

pub struct VKAnalogProfile {}

impl VKAnalogProfile {
    /// @brief select the current profile
    pub fn select(proto: &ViaKeychronProtocol, index: u8) -> ViaResult<()> {
        let cmd = &VKAnalogCommandId::SelectProfile;
        let resp = proto.device.raw_hid_send(&cmd.to_req(&[index]))?;
        cmd.check_reply(&resp)?;
        if let Some(info) = Arc::make_mut(&mut proto.get_info_mut()).analog_info.as_mut() {
            Arc::make_mut(info).data[0] = index;
        }
        Ok(())
    }

    pub fn load_raw_profile(proto: &ViaKeychronProtocol, index: u8) -> ViaResult<Vec<u8>> {
        todo!()
    }
}
