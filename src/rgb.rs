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

use via_protocol::ViaError;

use crate::{
    VKCommand, VKCommandId, VKCommandMaker, ViaKeychronProtocol, ViaReportData, ViaResult,
};

mod version;
pub use version::*;

mod indicators;
pub use indicators::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum VKRgbCommandId {
    RgbGetProtocolVer = 1,
    RgbSave = 2,
    GetIndicatorsConfig = 3,
    SetIndicatorsConfig = 4,
    RgbGetLedCount = 5,
    RgbGetLedIdx = 6,
    PerKeyRgbGetType = 7,
    PerKeyRgbSetType = 8,
    PerKeyRgbGetColor = 9,
    PerKeyRgbSetColor = 10,
    MixedEffectRgbGetInfo = 11,
    MixedEffectRgbGetRegions = 12,
    MixedEffectRgbSetRegions = 13,
    MixedEffectRgbGetEffectList = 14,
    MixedEffectRgbSetEffectList = 15,
}

impl VKRgbCommandId {
    /// Convenience function, checks reply returns payload if properly built
    #[tracing::instrument(level = "ERROR", err)]
    pub fn check_reply<'a>(&self, value: &'a ViaReportData) -> ViaResult<&'a [u8]> {
        if value[0] != VKCommandId::KeychronRgb as u8 {
            Err(ViaError::Protocol(format!(
                "invalid {self:?} packet, corrupted cmd: {}",
                value[0]
            )))
        } else if value[1] != *self as u8 {
            Err(ViaError::Protocol(format!(
                "invalid {self:?} packet, corrupted sub-cmd: {}",
                value[1]
            )))
        } else if value[2] != 0 {
            Err(ViaError::Protocol(format!(
                "invalid {self:?} packet, ret = {}",
                value[2]
            )))
        } else {
            Ok(&value[3..])
        }
    }
}

impl VKCommandMaker for VKRgbCommandId {
    fn to_cmd(self) -> VKCommand {
        let mut report = [0u8; via_protocol::VIA_REPORT_SIZE + 1];
        report[0] = 0x00;
        report[1] = VKCommandId::KeychronRgb as u8;
        report[2] = self as u8;
        VKCommand { report }
    }

    fn to_req(self, data: &[u8]) -> VKCommand {
        let mut ret = Self::to_cmd(self);
        let copy_len = data.len().min(via_protocol::VIA_REPORT_SIZE - 2);
        ret.report[3..3 + copy_len].copy_from_slice(&data[..copy_len]);
        ret
    }
}

pub trait VKRgbTrait: VKRgbProtocolVersionTrait + VKRgbIndicatorsTrait {
    fn save_rgb(&self) -> ViaResult<()>;
}

impl VKRgbTrait for ViaKeychronProtocol<'_> {
    fn save_rgb(&self) -> ViaResult<()> {
        let cmd = &VKRgbCommandId::RgbSave;
        let resp = self.device.raw_hid_send(&cmd.to_cmd())?;
        cmd.check_reply(&resp)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use palette::{Hsv, IntoColor, convert::{FromColorUnclamped, IntoColorUnclamped}, encoding::Srgb, named::AZURE};
    use serial_test::serial;

    use super::*;
    use crate::{
        VKFeatures, ViaKeychronProtocol,
        protocol::tests::{HID, get_keyboard},
    };

    #[test]
    #[serial(keyboard)]
    fn rgb() -> ViaResult<()> {
        let kbd = get_keyboard(&HID)?;
        let proto = ViaKeychronProtocol::new(&kbd);

        let support_features = proto.get_support_features()?;
        tracing::info!(?support_features);

        if support_features.contains(VKFeatures::KEYCHRON_RGB) {
            let rgb_ver = VKRgbProtocolVersion::load(&proto)?;
            tracing::info!(?rgb_ver);

            proto.save_rgb()?;
        } else {
            VKRgbProtocolVersion::load(&proto).expect_err("should fail");
            return Ok(());
        }

        let mut indicators = VKRgbIndicatorsConfig::load(&proto)?;
        tracing::info!(%indicators);
        let initial = indicators.get_color();
        indicators.set_color(&palette::named::RED.into_format::<f32>().into_color());
        indicators.send(&proto)?;
        indicators.set_color(&initial);
        indicators.send(&proto)?;
        Ok(())
    }
}
