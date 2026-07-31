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

use std::{borrow::Cow, sync::Arc};

use via_protocol::{VIA_REPORT_SIZE, ViaError, ViaResult};

use super::VKAnalogKeyConfig;
use crate::{VKAnalogCommandId, VKCommandMaker, ViaKeychronProtocol, ViaReportData};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct VKAnalogProfileInfo {
    pub data: Vec<u8>,
}

impl VKAnalogProfileInfo {
    pub fn load(proto: &ViaKeychronProtocol) -> ViaResult<Arc<Self>> {
        if let Some(value) = proto.get_info().analog.as_ref() {
            Ok(value.clone())
        } else {
            let resp = proto
                .device
                .raw_hid_send(&VKAnalogCommandId::GetProfilesInfo.to_cmd())?;
            let mut profile = Self::try_from(resp)?;

            profile.load_key_count(proto)?;
            let profile = Arc::new(profile);
            Arc::make_mut(&mut proto.get_info_mut())
                .analog
                .replace(profile.clone());
            Ok(profile)
        }
    }

    fn load_key_count(&mut self, proto: &ViaKeychronProtocol) -> ViaResult<()> {
        let cmd = &VKAnalogCommandId::GetCalibratedValue;
        let resp = proto.device.raw_hid_send(&cmd.to_req(&[0xFF, 0xFF]))?;
        let payload = cmd.check_reply(&resp)?;
        if payload[2] != 0 {
            return Err(ViaError::Protocol(format!(
                "invalid {cmd:?} packet, ret = {}",
            payload[2]
            )))
        }
        self.data[6] = payload[3]; /* rows */
        self.data[7] = payload[5]; /* cols */
        Ok(())
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

    /// @brief Get profile row count
    pub fn get_row_count(&self) -> u8 {
        self.data[6]
    }

    /// @brief Get profile columns count
    pub fn get_col_count(&self) -> u8 {
        self.data[7]
    }

    /// @brief get the profile key count
    pub fn get_key_count(&self) -> usize {
        (self.get_row_count() * self.get_col_count()) as usize
    }

    /// @brief get the name size in bytes (encoded in UTF-16)
    pub fn get_name_size(&self) -> usize {}
}

impl std::fmt::Display for VKAnalogProfileInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VKAnalogProfileInfo")
            .field("current", &self.get_current_profile())
            .field("count", &self.get_profile_count())
            .field("byte_size", &self.get_raw_profile_byte_size())
            .field("okmc_count", &self.get_okmc_count())
            .field("socd_count", &self.get_socd_count())
            .field("row_count", &self.get_row_count())
            .field("col_count", &self.get_col_count())
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

pub struct VKAnalogOkmcConfig<'a> {
    pub data: Cow<'a, [u8]>,
}

pub struct VKAnalogSocdConfig<'a> {
    pub data: Cow<'a, [u8]>,
}

pub struct VKAnalogProfile {
    pub data: Vec<u8>,
}

impl VKAnalogProfile {
    pub const MAX_REQ_RAW_BYTES: usize =
        VIA_REPORT_SIZE - (VKAnalogCommandId::HEADER_BYTE_SIZE + 4/* args size */);

    pub fn get_global_key_config<'a>(&'a self) -> ViaResult<VKAnalogKeyConfig<'a>> {
        VKAnalogKeyConfig::try_from(&self.data[0..VKAnalogKeyConfig::BYTE_SIZE])
    }
    pub fn get_key_config<'a>(&'a self, index: usize) -> ViaResult<VKAnalogKeyConfig<'a>> {
        let pos = (1 + index) * VKAnalogKeyConfig::BYTE_SIZE;
        VKAnalogKeyConfig::try_from(&self.data[pos..pos + VKAnalogKeyConfig::BYTE_SIZE])
    }
    pub fn get_okmc_config<'a>(&'a self) -> ViaResult<VKAnalogOkmcConfig<'a>> {
        todo!()
    }
    pub fn get_socd_config<'a>(&'a self) -> ViaResult<VKAnalogSocdConfig<'a>> {
        todo!()
    }
    pub fn get_name(&self) -> ViaResult<String> {
        todo!()
    }

    /// @brief select the current profile
    pub fn select(proto: &ViaKeychronProtocol, index: u8) -> ViaResult<()> {
        let cmd = &VKAnalogCommandId::SelectProfile;
        let resp = proto.device.raw_hid_send(&cmd.to_req(&[index]))?;
        cmd.check_reply(&resp)?;
        if let Some(info) = Arc::make_mut(&mut proto.get_info_mut()).analog.as_mut() {
            Arc::make_mut(info).data[0] = index;
        }
        Ok(())
    }

    pub fn load(proto: &ViaKeychronProtocol, index: u8) -> ViaResult<VKAnalogProfile> {
        let total_size = VKAnalogProfileInfo::load(proto)?.get_raw_profile_byte_size() as usize;
        let cmd = &VKAnalogCommandId::GetProfileRaw;
        let mut data = Vec::with_capacity(total_size);

        let mut offset = 0;
        while offset < total_size {
            let size = Self::MAX_REQ_RAW_BYTES.min(total_size - offset);
            let resp = proto.device.raw_hid_send(&cmd.to_req(&[
                index,
                (offset & 0xFF) as u8,
                ((offset >> 8) & 0xFF) as u8,
                size as u8,
            ]))?;
            let payload = cmd.check_reply(&resp)?;
            data.extend_from_slice(&payload[4..4 + size]);
            offset += size;
        }
        Ok(VKAnalogProfile { data })
    }
}
