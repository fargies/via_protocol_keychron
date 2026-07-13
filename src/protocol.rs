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

use via_protocol::{KeyboardDevice, KeyboardInfo, VIA_USAGE, VIA_USAGE_PAGE, ViaError, ViaResult};

use crate::{
    VKCommandId, VKCommandMaker, VKFeatures, VKMiscCommandId, VKMiscFeatures, VKProtocolVersion,
};

pub const KEYCHRON_VENDOR_ID: u16 = 0x3434;

pub struct ViaKeychronProtocol<'a> {
    pub device: &'a KeyboardDevice,
}

impl<'a> ViaKeychronProtocol<'a> {
    pub fn new(device: &'a KeyboardDevice) -> Self {
        Self { device }
    }

    pub fn get_protocol_version(&self) -> ViaResult<VKProtocolVersion> {
        VKProtocolVersion::try_from(
            &self
                .device
                .raw_hid_send(&VKCommandId::GetProtocolVersion.to_cmd())?,
        )
    }

    pub fn get_firmware_version(&self) -> ViaResult<String> {
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
        .map(|r| r.to_string())
    }

    pub fn get_support_features(&self) -> ViaResult<VKFeatures> {
        let cmd = &VKCommandId::GetSupportFeature;
        let resp = self.device.raw_hid_send(&cmd.to_cmd())?;
        let payload = cmd.check_reply(&resp)?;
        Ok(VKFeatures::from_bits_retain(u16::from_le_bytes([
            payload[1], payload[2],
        ])))
    }

    /// @returns `(default_layer_state, layer_state)`
    pub fn get_default_layer(&self) -> ViaResult<(u8, u8)> {
        let cmd = &VKCommandId::GetDefaultLayer;
        let resp = self.device.raw_hid_send(&cmd.to_cmd())?;
        let payload = cmd.check_reply(&resp)?;
        Ok((payload[0], payload[1]))
    }

    pub fn get_misc_protcol_version(&self) -> ViaResult<(u16, VKMiscFeatures)> {
        let cmd = &VKMiscCommandId::MiscGetProtocolVer;
        let resp = self.device.raw_hid_send(&cmd.to_cmd())?;
        let payload = cmd.check_reply(&resp)?;
        Ok((
            u16::from_le_bytes([payload[0], payload[1]]),
            VKMiscFeatures::from_bits_retain(payload[2]),
        ))
    }

    pub fn get_language(&self) -> ViaResult<u8> {
        let cmd = &VKMiscCommandId::LanguageGet;
        let resp = self.device.raw_hid_send(&cmd.to_cmd())?;

        Ok(cmd.check_reply(&resp)?[0])
    }

    pub fn set_language(&self, language: u8) -> ViaResult<()> {
        let cmd = &VKMiscCommandId::LanguageSet;
        let resp = self.device.raw_hid_send(&cmd.to_req(&[language]))?;

        cmd.check_reply(&resp)?;
        Ok(())
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

#[cfg(test)]
pub mod tests {
    use std::sync::LazyLock;

    use super::*;
    use crate::{
        VKDebounceConfig, VKDfuInfo, VKNkroConfig, VKReportRateConfig, VKSnapClickConfig,
        VKWirelessLpmConfig,
    };
    use hidapi::HidApi;
    use serial_test::serial;
    use via_protocol::{KeyboardDevice, ViaResult};

    pub static HID: LazyLock<HidApi> = LazyLock::new(|| HidApi::new().expect("failed to open hidapi"));

    pub fn get_keyboard(api: &HidApi) -> ViaResult<KeyboardDevice> {
        let keyboards = discover_keyboards(api);
        assert!(!keyboards.is_empty());
        KeyboardDevice::open(api, keyboards.first().unwrap().clone())
    }

    #[test]
    #[serial(keyboard)]
    fn connect() -> ViaResult<()> {
        get_keyboard(&HID).and(Ok(()))
    }

    #[test]
    #[serial(keyboard)]
    fn info() -> ViaResult<()> {
        let kbd = get_keyboard(&HID)?;
        let proto = ViaKeychronProtocol::new(&kbd);

        let ret = proto.get_protocol_version()?;
        tracing::info!(protocol_version = ?ret);

        let ret = proto.get_firmware_version()?;
        tracing::info!(firmware_version = ?ret);

        let support_features = proto.get_support_features()?;
        tracing::info!(?support_features);

        let ret = proto.get_default_layer()?;
        tracing::info!(default_layer = ?ret);
        Ok(())
    }

    #[test]
    #[serial(keyboard)]
    fn misc() -> ViaResult<()> {
        let kbd = get_keyboard(&HID)?;
        let proto = ViaKeychronProtocol::new(&kbd);

        let ret = proto.get_misc_protcol_version()?;
        tracing::info!(misc_protocol_version = ?ret);
        let features = ret.1;

        if features.contains(VKMiscFeatures::DFU_INFO) {
            let dfu_info = VKDfuInfo::load(&proto)?;
            tracing::info!(?dfu_info);
        } else {
            VKDfuInfo::load(&proto).expect_err("should fail");
        }

        if features.contains(VKMiscFeatures::LANGUAGE) {
            let ret = proto.get_language()?;
            tracing::info!(language = ?ret);
            proto.set_language(ret)?;
        } else {
            proto.get_language().expect_err("should fail");
        }

        if features.contains(VKMiscFeatures::DEBOUNCE) {
            let debounce = VKDebounceConfig::load(&proto)?;
            tracing::info!(%debounce);
            tracing::trace!(?debounce);
            debounce.send(&proto)?;
        } else {
            VKDebounceConfig::load(&proto).expect_err("should fail");
        }

        if features.contains(VKMiscFeatures::SNAP_CLICK) {
            let snap_click_count = VKSnapClickConfig::count(&proto)?;
            tracing::info!(snap_click_count);
            assert!(snap_click_count >= 9);
            let snaps = VKSnapClickConfig::load(0, 9, &proto)?;
            assert_eq!(snaps.config.len(), 9);
            tracing::info!(%snaps);
            tracing::trace!(?snaps);
            snaps.send(&proto)?;
            snaps.save(&proto)?;
        } else {
            VKSnapClickConfig::count(&proto).expect_err("should fail");
        }

        if features.contains(VKMiscFeatures::WIRELESS_LPM) {
            let wireless_lpm = VKWirelessLpmConfig::load(&proto)?;
            tracing::info!(%wireless_lpm);
            tracing::trace!(?wireless_lpm);
            wireless_lpm.send(&proto)?;
        } else {
            VKWirelessLpmConfig::load(&proto).expect_err("should fail");
        }

        if features.contains(VKMiscFeatures::REPORT_RATE) {
            let report_rate = VKReportRateConfig::load(&proto)?;
            tracing::info!(%report_rate);
            tracing::trace!(?report_rate);
            report_rate.send(&proto)?;
        } else {
            VKReportRateConfig::load(&proto).expect_err("should fail");
        }

        if features.contains(VKMiscFeatures::NKRO) {
            let nkro = VKNkroConfig::load(&proto)?;
            tracing::info!(%nkro);
            tracing::trace!(?nkro);
            nkro.send(&proto)?;
        } else {
            VKNkroConfig::load(&proto).expect_err("should fail");
        }
        Ok(())
    }
}
