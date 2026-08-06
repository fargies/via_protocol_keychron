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

use via_protocol::{VIA_REPORT_SIZE, ViaError, ViaResult};

use crate::{
    VKAnalogCommandId, VKAnalogGamepadData, VKAnalogInfo, VKAnalogKeyConfig, VKCommandMaker,
    VKOkmcConfig, VKSocdConfig, ViaKeychronProtocol, ViaReportData,
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct VKAnalogProfileInfo {
    pub data: Vec<u8>,
}

impl VKAnalogProfileInfo {
    pub fn load(proto: &ViaKeychronProtocol) -> ViaResult<Arc<Self>> {
        VKAnalogInfo::load(proto).map(|info| info.profile_info.clone())
    }

    pub(crate) fn load_key_count(&mut self, proto: &ViaKeychronProtocol) -> ViaResult<()> {
        let cmd = &VKAnalogCommandId::GetCalibratedValue;
        let resp = proto.device.raw_hid_send(&cmd.to_req(&[0xFF, 0xFF]))?;
        let payload = cmd.check_reply(&resp)?;
        self.data[6] = payload[0]; /* rows */
        self.data[7] = payload[2]; /* cols */
        Ok(())
    }

    /// @brief get current profile index
    pub fn get_current_profile(&self) -> usize {
        self.data[0] as usize
    }

    pub fn select_profile(proto: &ViaKeychronProtocol, index: usize) -> ViaResult<()> {
        let cmd = &VKAnalogCommandId::SelectProfile;
        let resp = proto.device.raw_hid_send(&cmd.to_req(&[index as u8]))?;
        cmd.check_reply(&resp)?;
        if let Some(info) = Arc::make_mut(&mut proto.get_info_mut())
            .analog
            .as_mut()
            .map(|info| &mut Arc::make_mut(info).profile_info)
        {
            Arc::make_mut(info).data[0] = index as u8;
        }
        Ok(())
    }

    pub fn save_profile(proto: &ViaKeychronProtocol, index: usize) -> ViaResult<()> {
        let cmd = &VKAnalogCommandId::SaveProfile;
        let resp = proto.device.raw_hid_send(&cmd.to_req(&[index as u8]))?;
        cmd.check_reply(&resp)?;
        Ok(())
    }

    /// @brief get number of profiles
    pub fn get_profile_count(&self) -> usize {
        self.data[1] as usize
    }

    /// @brief get profile size in bytes
    pub fn get_raw_profile_byte_size(&self) -> usize {
        u16::from_le_bytes(self.data[2..4].try_into().unwrap()) as usize
    }

    /// @brief One Key Multiple Command count
    pub fn get_okmc_count(&self) -> usize {
        self.data[4] as usize
    }

    /// @brief Simultaneous Opposing Cardinal Directions count
    pub fn get_socd_count(&self) -> usize {
        self.data[5] as usize
    }

    /// @brief Get profile row count
    pub fn get_row_count(&self) -> usize {
        self.data[6] as usize
    }

    /// @brief Get profile columns count
    pub fn get_col_count(&self) -> usize {
        self.data[7] as usize
    }

    /// @brief get the profile key count
    pub fn get_key_count(&self) -> usize {
        self.get_row_count() * self.get_col_count()
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

#[derive(Debug, Clone)]
pub struct VKAnalogProfile {
    pub data: Vec<u8>,
    pub index: usize,
    pub key_count: usize,
    pub okmc_count: usize,
    pub socd_count: usize,
    pub row_count: usize,
    pub col_count: usize,
}

impl VKAnalogProfile {
    pub const MAX_REQ_RAW_BYTES: usize =
        VIA_REPORT_SIZE - (VKAnalogCommandId::HEADER_BYTE_SIZE + 4/* args size */);

    /// maximum name length, not including terminating `\0`
    pub const MAX_NAME_LEN: usize = VIA_REPORT_SIZE - (VKAnalogCommandId::HEADER_BYTE_SIZE + 2);

    pub fn get_global_key_config<'a>(&'a self) -> ViaResult<VKAnalogKeyConfig<'a>> {
        VKAnalogKeyConfig::try_from(&self.data[0..VKAnalogKeyConfig::BYTE_SIZE])
    }

    #[inline]
    fn get_key_config_offset(&self, index: usize) -> usize {
        (1 + index) * VKAnalogKeyConfig::BYTE_SIZE
    }

    pub fn get_key_config<'a>(&'a self, index: usize) -> ViaResult<VKAnalogKeyConfig<'a>> {
        if index >= self.key_count {
            return Err(ViaError::Protocol(format!(
                "no such key index: {index} (max:{})",
                self.key_count - 1
            )));
        }
        let pos = self.get_key_config_offset(index);
        VKAnalogKeyConfig::try_from((&self.data[pos..pos + VKAnalogKeyConfig::BYTE_SIZE], index))
    }

    pub fn iter_key_config<'a>(&'a self) -> impl Iterator<Item = VKAnalogKeyConfig<'a>> {
        (0..self.key_count).filter_map(|i| self.get_key_config(i).ok())
    }

    #[inline]
    fn get_okmc_config_offset(&self, index: usize) -> usize {
        (1 + self.key_count) * VKAnalogKeyConfig::BYTE_SIZE + index * VKOkmcConfig::BYTE_SIZE
    }

    pub fn get_okmc_config<'a>(&'a self, index: usize) -> ViaResult<VKOkmcConfig<'a>> {
        if index >= self.okmc_count {
            return Err(ViaError::Protocol(format!(
                "no such okmc index: {index} (max:{})",
                self.okmc_count - 1
            )));
        }
        let pos = self.get_okmc_config_offset(index);
        VKOkmcConfig::try_from((&self.data[pos..pos + VKOkmcConfig::BYTE_SIZE], index))
    }

    pub fn iter_okmc_config<'a>(&'a self) -> impl Iterator<Item = VKOkmcConfig<'a>> {
        (0..self.okmc_count).filter_map(|index| self.get_okmc_config(index).ok())
    }

    pub fn get_socd_config<'a>(&'a self, index: usize) -> ViaResult<VKSocdConfig<'a>> {
        if index >= self.socd_count {
            return Err(ViaError::Protocol(format!(
                "no such socd index: {index} (max:{})",
                self.socd_count - 1
            )));
        }
        let pos = (1 + self.key_count) * VKAnalogKeyConfig::BYTE_SIZE
            + self.okmc_count * VKOkmcConfig::BYTE_SIZE
            + index * VKSocdConfig::BYTE_SIZE;
        VKSocdConfig::try_from((
            &self.data[pos..pos + VKSocdConfig::BYTE_SIZE],
            self.col_count,
        ))
    }

    pub fn iter_socd_config<'a>(&'a self) -> impl Iterator<Item = VKSocdConfig<'a>> {
        (0..self.socd_count).filter_map(|index| self.get_socd_config(index).ok())
    }

    pub fn get_name(&self) -> ViaResult<String> {
        let pos = (1 + self.key_count) * VKAnalogKeyConfig::BYTE_SIZE
            + self.okmc_count * VKOkmcConfig::BYTE_SIZE
            + self.socd_count * VKSocdConfig::BYTE_SIZE;
        std::char::decode_utf16(
            self.data[pos..self.data.len() - 2]
                .chunks(2)
                .map_while(|k| {
                    k.as_array::<2>()
                        .map(|k| u16::from_be_bytes(*k))
                        .filter(|v| v != &0)
                }),
        )
        .collect::<Result<String, _>>()
        .map_err(|e| ViaError::Protocol(format!("failed to parse utf16: {e}")))
    }

    pub fn set_name<K>(&mut self, name: K) -> ViaResult<()>
    where
        K: AsRef<str>,
    {
        let pos = (1 + self.key_count) * VKAnalogKeyConfig::BYTE_SIZE
            + self.okmc_count * VKOkmcConfig::BYTE_SIZE
            + self.socd_count * VKSocdConfig::BYTE_SIZE;
        let mut buffer = [0; 2];
        let mut out = self.data[pos..pos + Self::MAX_NAME_LEN].iter_mut();
        for char in name.as_ref().chars() {
            let buffer = char.encode_utf16(&mut buffer);

            for c in buffer {
                for b in c.to_be_bytes() {
                    *out.next().ok_or_else(|| {
                        ViaError::Protocol("name too long for VKAnalogProfile".into())
                    })? = b;
                }
            }
        }
        for c in out {
            *c = 0;
        }
        Ok(())
    }

    pub fn send_name(&self, proto: &ViaKeychronProtocol) -> ViaResult<()> {
        let cmd = &VKAnalogCommandId::SetProfileName;
        let pos = (1 + self.key_count) * VKAnalogKeyConfig::BYTE_SIZE
            + self.okmc_count * VKOkmcConfig::BYTE_SIZE
            + self.socd_count * VKSocdConfig::BYTE_SIZE;
        let mut pkt = cmd.to_cmd();
        pkt.report[3] = self.index as u8;
        pkt.report[4] = Self::MAX_NAME_LEN as u8;
        pkt.report[5..].copy_from_slice(&self.data[pos..pos + Self::MAX_NAME_LEN]);
        let resp = proto.device.raw_hid_send(&pkt)?;
        cmd.check_reply(&resp)?;

        Ok(())
    }

    /// @brief select the current profile
    pub fn select(&self, proto: &ViaKeychronProtocol) -> ViaResult<()> {
        VKAnalogProfileInfo::select_profile(proto, self.index)
    }

    pub fn load(proto: &ViaKeychronProtocol, index: usize) -> ViaResult<VKAnalogProfile> {
        let info = VKAnalogProfileInfo::load(proto)?;
        let total_size = info.get_raw_profile_byte_size();
        let cmd = &VKAnalogCommandId::GetProfileRaw;
        let mut data = Vec::with_capacity(total_size);

        let mut offset = 0;
        while offset < total_size {
            let size = Self::MAX_REQ_RAW_BYTES.min(total_size - offset);
            let resp = proto.device.raw_hid_send(&cmd.to_req(&[
                index as u8,
                (offset & 0xFF) as u8,
                ((offset >> 8) & 0xFF) as u8,
                size as u8,
            ]))?;
            let payload = cmd.check_reply(&resp)?;
            data.extend_from_slice(&payload[4..4 + size]);
            offset += size;
        }
        Ok(VKAnalogProfile {
            data,
            index,
            okmc_count: info.get_okmc_count(),
            socd_count: info.get_socd_count(),
            key_count: info.get_key_count(),
            row_count: info.get_row_count(),
            col_count: info.get_col_count(),
        })
    }
}

pub enum VKAdvModeArg<'a> {
    Okmc(&'a VKOkmcConfig<'a>),
    GamePad(VKAnalogGamepadData),
}
