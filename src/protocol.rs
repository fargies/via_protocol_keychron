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

use std::sync::{Arc, Mutex, MutexGuard};

use via_protocol::{KeyboardDevice, KeyboardInfo, VIA_USAGE, VIA_USAGE_PAGE, ViaError, ViaResult};

use crate::{
    VKAnalogProfileInfo, VKCommandId, VKCommandMaker, VKFeatures, VKMiscInfo, VKProtocolVersion,
    VKRgbInfo,
};

pub const KEYCHRON_VENDOR_ID: u16 = 0x3434;

pub struct ViaKeychronProtocol<'a> {
    /// @brief device pointer
    pub device: &'a KeyboardDevice,

    /// @brief cached device information
    pub info: Mutex<Arc<VKDeviceInfo>>,
}

#[derive(Debug, Default, Clone)]
pub struct VKDeviceInfo {
    /// @brief device protocol information
    /// @details fetched using [VKProtocolVersion::load]
    pub protocol: Option<Arc<VKProtocolVersion>>,

    /// @brief firmware version
    /// @details fetched using [VKDeviceInfo::get_firmware_version]
    pub firmware_version: Option<Arc<String>>,

    /// @brief device features
    /// @details fetched using [VKDeviceInfo::get_support_features]
    pub features: Option<Arc<VKFeatures>>,

    /// @brief Misc protocol information
    /// @details fetched using [VKMiscInfo::load]
    pub misc: Option<Arc<VKMiscInfo>>,

    /// @brief RgbMixed layers/effects info
    /// @details fetched using [VKRgbMixedInfo::load]
    pub rgb: Option<Arc<VKRgbInfo>>,

    /// @brief analog keyboards info
    /// @details fetched using [VKAnalogProfileInfo::load]
    pub analog: Option<Arc<VKAnalogProfileInfo<'static>>>,
}

impl<'a> ViaKeychronProtocol<'a> {
    pub fn new(device: &'a KeyboardDevice) -> Self {
        Self {
            device,
            info: Default::default(),
        }
    }

    /// @brief direct access to [VKDeviceInfo]
    /// @details info may not be loaded (yet), see [ViaKeychronProtocol::load_info]
    #[inline]
    pub fn get_info(&self) -> Arc<VKDeviceInfo> {
        Arc::clone(&self.info.lock().unwrap())
    }

    pub fn load_info(&self) -> ViaResult<Arc<VKDeviceInfo>> {
        VKProtocolVersion::load(self)?;
        self.get_firmware_version()?;
        VKFeatures::load(self)?;
        VKMiscInfo::load(self)?;
        VKRgbInfo::load(self)?;
        VKAnalogProfileInfo::load(self)?;
        Ok(self.get_info())
    }

    #[inline]
    pub(crate) fn get_info_mut(&self) -> MutexGuard<'_, Arc<VKDeviceInfo>> {
        self.info.lock().unwrap()
    }

    pub fn get_support_features(&self) -> ViaResult<Arc<VKFeatures>> {
        VKFeatures::load(self)
    }

    pub fn get_firmware_version(&self) -> ViaResult<Arc<String>> {
        if let Some(value) = self.get_info().firmware_version.as_ref() {
            Ok(value.clone())
        } else {
            let cmd = &VKCommandId::GetFirmwareVersion;
            let resp = self.device.raw_hid_send(&cmd.to_cmd())?;
            let payload = cmd.check_reply(&resp)?;
            str::from_utf8(
                &payload[..payload
                    .iter()
                    .position(|&c| c == b'\0')
                    .unwrap_or(resp.len())],
            )
            .map_err(|e| ViaError::Protocol(format!("failed to parse firmware version: {e}")))
            .map(|r| Arc::new(r.to_string()))
            .inspect(|p| {
                Arc::make_mut(&mut self.get_info_mut())
                    .firmware_version
                    .replace(Arc::clone(p));
            })
        }
    }

    /// @returns `(default_layer_state, layer_state)`
    pub fn get_default_layer(&self) -> ViaResult<(u8, u8)> {
        let cmd = &VKCommandId::GetDefaultLayer;
        let resp = self.device.raw_hid_send(&cmd.to_cmd())?;
        let payload = cmd.check_reply(&resp)?;
        Ok((payload[0], payload[1]))
    }
}

pub fn discover_keyboards(api: &hidapi::HidApi) -> Vec<KeyboardInfo> {
    let keyboards: Vec<_> = api
        .device_list()
        .filter(|dev| {
            dev.vendor_id() == KEYCHRON_VENDOR_ID
                && dev.usage_page() == VIA_USAGE_PAGE
                && dev.usage() == VIA_USAGE
        })
        .map(|dev| KeyboardInfo {
            vendor_id: dev.vendor_id(),
            product_id: dev.product_id(),
            manufacturer: dev.manufacturer_string().unwrap_or_default().to_string(),
            product: dev.product_string().unwrap_or_default().to_string(),
            serial_number: dev.serial_number().unwrap_or_default().to_string(),
            path: dev.path().to_string_lossy().into_owned(),
        })
        .collect();

    tracing::info!(count = keyboards.len(), "discovered Keychron keyboards");
    for kb in &keyboards {
        tracing::debug!(keyboard = %kb, path = %kb.path, "found keyboard");
    }

    keyboards
}
