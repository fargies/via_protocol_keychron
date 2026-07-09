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
    ViaKeychronCommand, ViaKeychronCommandId, ViaKeychronFeatures, ViaKeychronProtocolVersion,
};

pub const KEYCHRON_VENDOR_ID: u16 = 0x3434;

pub struct ViaKeychronProtocol<'a> {
    device: &'a KeyboardDevice,
}

impl<'a> ViaKeychronProtocol<'a> {
    pub fn new(device: &'a KeyboardDevice) -> Self {
        Self { device }
    }

    pub fn get_protocol_version(&self) -> ViaResult<ViaKeychronProtocolVersion> {
        Ok(ViaKeychronProtocolVersion::from(self.device.raw_hid_send(
            &ViaKeychronCommand::simple(ViaKeychronCommandId::GetProtocolVersion),
        )?))
    }

    pub fn get_firmware_version(&self) -> ViaResult<String> {
        let resp = self.device.raw_hid_send(&ViaKeychronCommand::simple(
            ViaKeychronCommandId::GetFirmwareVersion,
        ))?;
        str::from_utf8(&resp[1..resp.iter().position(|&c| c == b'\0').unwrap_or(resp.len())])
            .map_err(|e| ViaError::Protocol(format!("failed to parse firmware version: {e}")))
            .map(|r| r.to_string())
    }

    pub fn get_support_feature(&self) -> ViaResult<ViaKeychronFeatures> {
        let resp = self.device.raw_hid_send(&ViaKeychronCommand::simple(
            ViaKeychronCommandId::GetSupportFeature,
        ))?;
        Ok(ViaKeychronFeatures::from_bits_retain(u16::from_be_bytes([
            resp[1], resp[2],
        ])))
    }

    /// @returns `(default_layer_state, layer_state)`
    pub fn get_default_layer(&self) -> ViaResult<(u8, u8)> {
        let resp = self.device.raw_hid_send(&ViaKeychronCommand::simple(
            ViaKeychronCommandId::GetDefaultLayer,
        ))?;
        Ok((resp[1], resp[2]))
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
mod tests {
    use super::*;
    use hidapi::HidApi;
    use via_protocol::{KeyboardDevice, ViaResult};
    fn get_keyboard(api: &HidApi) -> ViaResult<KeyboardDevice> {
        let keyboards = discover_keyboards(api);
        assert!(!keyboards.is_empty());
        KeyboardDevice::open(api, keyboards.first().unwrap().clone())
    }

    #[test]
    fn connect() -> ViaResult<()> {
        let api = HidApi::new()?;
        get_keyboard(&api).and(Ok(()))
    }

    #[test]
    fn info() -> ViaResult<()> {
        let api = HidApi::new()?;
        let kbd = get_keyboard(&api)?;
        let proto = ViaKeychronProtocol::new(&kbd);

        let ret = proto.get_protocol_version()?;
        tracing::info!(protocol_version = ?ret);

        let ret = proto.get_firmware_version()?;
        tracing::info!(firmware_version = ?ret);

        let ret = proto.get_support_feature()?;
        tracing::info!(support_feature = ?ret);

        let ret = proto.get_default_layer()?;
        tracing::info!(default_layer = ?ret);
        Ok(())
    }
}
