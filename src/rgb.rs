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

use palette::{FromColor, Hsv, IntoColor};
use via_protocol::ViaError;

use crate::{
    VKCommand, VKCommandId, VKCommandMaker, ViaKeychronProtocol, ViaReportData, ViaResult,
};

mod info;
pub use info::*;

mod version;
pub use version::*;

mod indicators;
pub use indicators::*;

mod per_key;
pub use per_key::*;

mod mixed;
pub use mixed::*;

/// RGB command IDs
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
    pub const HEADER_BYTE_SIZE: usize = 3;

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
            Ok(&value[Self::HEADER_BYTE_SIZE..])
        }
    }
}

impl VKCommandMaker for VKRgbCommandId {
    /// @brief generate [VKCommand] from a [VKRgbCommandId]
    fn to_cmd(self) -> VKCommand {
        let mut report = [0u8; via_protocol::VIA_REPORT_SIZE + 1];
        report[0] = 0x00;
        report[1] = VKCommandId::KeychronRgb as u8;
        report[2] = self as u8;
        VKCommand { report, payload_offset: 3 }
    }

    /// @brief generate [VKCommand] from a [VKRgbCommandId] and data
    fn to_req(self, data: &[u8]) -> VKCommand {
        let mut ret = Self::to_cmd(self);
        let copy_len = data.len().min(via_protocol::VIA_REPORT_SIZE - 2);
        ret.report[3..3 + copy_len].copy_from_slice(&data[..copy_len]);
        ret
    }
}

pub struct VKRgb {}

impl VKRgb {
    /// @brief save current config to EEPROM
    pub fn save(proto: &ViaKeychronProtocol) -> ViaResult<()> {
        let cmd = &VKRgbCommandId::RgbSave;
        let resp = proto.raw_send(&cmd.to_cmd())?;
        cmd.check_reply(&resp)?;
        Ok(())
    }

    /// get device's led count
    pub fn get_led_count(proto: &ViaKeychronProtocol) -> ViaResult<usize> {
        VKRgbInfo::load(proto).map(|info| info.led_count)
    }
}

pub trait VKRgbTrait {
    /// @brief get RGB info
    fn get_rgb_info(&self) -> ViaResult<Arc<VKRgbInfo>>;

    /// @brief save RGB config to EEPROM
    fn save_rgb(&self) -> ViaResult<()>;

    /// @brief get LED count
    fn get_led_count(&self) -> ViaResult<usize>;

    /// @brief get RGB indicators
    fn get_indicators(&self) -> ViaResult<VKRgbIndicatorsConfig>;

    /// @brief set RGB indicators
    fn set_indicators(&self, value: &VKRgbIndicatorsConfig) -> ViaResult<()>;

    /// @brief get RGB mixed info
    fn get_mixed_info(&self) -> ViaResult<Arc<VKRgbMixedInfo>>;

    /// @brief get RGB mixed regions
    fn get_mixed_regions(&self) -> ViaResult<VKRgbMixedRegions>;

    /// @brief set RGB mixed regions
    fn set_mixed_regions(&self, regions: &VKRgbMixedRegions) -> ViaResult<()>;

    /// @brief get RGB mixed effects
    fn get_mixed_effects(&self, region: u8) -> ViaResult<VKRgbMixedEffectList>;

    /// @brief get RGB per key type
    fn get_pk_type(&self) -> ViaResult<VKRgbPerKeyType>;

    /// @brief set RGB per key type
    fn set_pk_type(&self, value: &VKRgbPerKeyType) -> ViaResult<()>;

    /// @brief get RGB per key color
    fn get_pk_led_color(&self) -> ViaResult<VKRgbPerKeyConfig>;

    /// @brief set RGB per key color
    fn set_pk_led_color(&self, value: &VKRgbPerKeyConfig) -> ViaResult<()>;
}

impl VKRgbTrait for ViaKeychronProtocol<'_> {
    fn save_rgb(&self) -> ViaResult<()> {
        VKRgb::save(self)
    }

    fn get_led_count(&self) -> ViaResult<usize> {
        VKRgb::get_led_count(self)
    }

    fn get_rgb_info(&self) -> ViaResult<Arc<VKRgbInfo>> {
        VKRgbInfo::load(self)
    }

    fn get_indicators(&self) -> ViaResult<VKRgbIndicatorsConfig> {
        VKRgbIndicatorsConfig::load(self)
    }

    fn set_indicators(&self, value: &VKRgbIndicatorsConfig) -> ViaResult<()> {
        value.send(self)
    }

    fn get_mixed_info(&self) -> ViaResult<Arc<VKRgbMixedInfo>> {
        VKRgbMixedInfo::load(self)
    }

    fn get_mixed_regions(&self) -> ViaResult<VKRgbMixedRegions> {
        VKRgbMixedRegions::load(self)
    }

    fn set_mixed_regions(&self, regions: &VKRgbMixedRegions) -> ViaResult<()> {
        regions.send(self)
    }

    fn get_mixed_effects(&self, region: u8) -> ViaResult<VKRgbMixedEffectList> {
        VKRgbMixedEffectList::load(self, region)
    }

    fn get_pk_type(&self) -> ViaResult<VKRgbPerKeyType> {
        VKRgbPerKeyType::load(self)
    }

    fn set_pk_type(&self, value: &VKRgbPerKeyType) -> ViaResult<()> {
        value.send(self)
    }

    fn get_pk_led_color(&self) -> ViaResult<VKRgbPerKeyConfig> {
        VKRgbPerKeyConfig::load(self)
    }

    fn set_pk_led_color(&self, value: &VKRgbPerKeyConfig) -> ViaResult<()> {
        value.send(self)
    }
}

/// @brief RGB HSV color as used by Keychron keyboards
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct VKHsv {
    pub hue: u8,
    pub saturation: u8,
    pub value: u8,
}

impl VKHsv {
    /// @brief size of the HSV color in bytes
    pub const BYTE_SIZE: usize = 3;

    /// @brief serialize the HSV color into a buffer
    pub fn serialize(&self, buffer: &mut [u8]) -> ViaResult<()> {
        if buffer.len() < 3 {
            Err(ViaError::Protocol(
                "buffer too small to serialize VKHsv".into(),
            ))
        } else {
            buffer[0] = self.hue;
            buffer[1] = self.saturation;
            buffer[2] = self.value;
            Ok(())
        }
    }
}

impl TryFrom<&[u8]> for VKHsv {
    type Error = ViaError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < 3 {
            Err(ViaError::Protocol(
                "invalid hsv value: buffer too small".into(),
            ))
        } else {
            Ok(Self {
                hue: value[0],
                saturation: value[1],
                value: value[2],
            })
        }
    }
}

impl From<VKHsv> for Hsv {
    fn from(value: VKHsv) -> Self {
        Hsv::new(
            value.hue as f32 * 360.0 / 255.0,
            value.saturation as f32 / 255.0,
            value.value as f32 / 255.0,
        )
    }
}

impl<T> FromColor<T> for VKHsv
where
    T: IntoColor<Hsv>,
{
    fn from_color(value: T) -> Self {
        let value = value.into_color();
        Self {
            hue: (value.hue.into_positive_degrees() / 360.0 * 255.0).round() as u8,
            saturation: (value.saturation * 255.0).round() as u8,
            value: (value.value * 255.0).round() as u8,
        }
    }
}

#[cfg(test)]
mod tests {
    use palette::{FromColor, Hsv, named};

    use super::*;

    #[test]
    fn vkhsv() {
        let color = Hsv::from_color(named::BLUE.into_format::<f32>());
        let hsv: VKHsv = color.into_color();
        tracing::trace!(?hsv);

        assert_eq!(color, hsv.into());
    }
}
