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

use std::{borrow::Cow, fmt::Debug};

use via_protocol::{ViaError, ViaResult};

use crate::{
    VKAnalogCommandId, VKAnalogProfileInfo, VKCommandMaker, VKOkmcConfig, ViaKeychronProtocol,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VKAnalogKeyConfig<'a> {
    pub data: Cow<'a, [u8]>,
    pub index: Option<usize>,
    pub profile: Option<usize>,
}

impl VKAnalogKeyConfig<'_> {
    pub const BYTE_SIZE: usize = 4;

    pub fn get_mode(&self) -> ViaResult<VKAnalogKeyConfigMode> {
        let value = self.data[2] >> 4;

        if value != 0 {
            VKAnalogKeyConfigMode::try_from(value)
        } else {
            VKAnalogKeyConfigMode::try_from(self.data[0] & 0x03)
        }
    }

    /// set mode
    pub fn set_mode(&mut self, mode: VKAnalogKeyConfigMode) {
        let data = self.data.to_mut();
        match mode {
            VKAnalogKeyConfigMode::Global
            | VKAnalogKeyConfigMode::Regular
            | VKAnalogKeyConfigMode::Rapid => {
                data[0] = (data[0] & 0xFC) | ((mode as u8) & 0x03);
                data[2] &= 0x0F;
            }
            VKAnalogKeyConfigMode::DKS
            | VKAnalogKeyConfigMode::Gamepad
            | VKAnalogKeyConfigMode::Toggle => {
                data[2] = (data[2] & 0x0F) | ((mode as u8) << 4);
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        !self
            .get_mode()
            .is_ok_and(|m| m != VKAnalogKeyConfigMode::Global)
    }

    /// @brief Actuation point in 1/10 mm
    /// @details maximum value depends on keyboard, minimum value is generaly 0,2mm
    pub fn get_actuation_point(&self) -> u8 {
        self.data[0] >> 2
    }

    /// @brief rapid trigger activation in 1/10mm
    pub fn get_rapid_trig_sen(&self) -> u8 {
        self.data[1] & 0x3F
    }

    /// @brief rapid trigger deactivation in 1/10mm
    pub fn get_rapid_trig_sen_deact(&self) -> u8 {
        self.data[1] >> 6 | (self.data[2] & 0x0F) << 2
    }

    /// @details
    /// - not used in Regular or Rapid mode
    /// - in [VKAnalogKeyConfigMode::DKS] this is the OKMC index
    ///
    pub fn get_adv_mode_info(&self) -> ViaResult<VKAnalogKeyConfigAdvData> {
        match self.get_mode() {
            Ok(VKAnalogKeyConfigMode::DKS) => Ok(VKAnalogKeyConfigAdvData::Okmc {
                index: self.data[3] as usize,
            }),
            Ok(VKAnalogKeyConfigMode::Gamepad) => {
                VKAnalogGamepadData::try_from(self.data[3]).map(VKAnalogKeyConfigAdvData::Gamepad)
            }
            Ok(mode) => Err(ViaError::Protocol(format!("no data for mode: {mode:?}"))),
            Err(e) => Err(e),
        }
    }

    /// Convenience function to access okmc index
    pub fn get_okmc_index(&self) -> Option<usize> {
        if let Ok(VKAnalogKeyConfigMode::DKS) = self.get_mode() {
            Some(self.data[3] as usize)
        } else {
            None
        }
    }

    /// set advanced mode data
    pub fn set_adv_mode_info<K>(&mut self, value: K)
    where
        K: Into<VKAnalogKeyConfigAdvData>,
    {
        let data = self.data.to_mut();
        data[3] = value.into().into();
    }

    pub fn send(&self, proto: &ViaKeychronProtocol, okmc: Option<&VKOkmcConfig>) -> ViaResult<()> {
        let profile = self.profile.ok_or_else(|| {
            ViaError::Protocol(
                "destination profile must be set to invoke VKAnalogKeyConfig::send".into(),
            )
        })?;

        self.send_travel(proto, profile)?;
        match self.get_mode()? {
            VKAnalogKeyConfigMode::Global
            | VKAnalogKeyConfigMode::Regular
            | VKAnalogKeyConfigMode::Rapid => {
                self.send_adv_mode(proto, VKAnalogKeyConfigAdvMode::Clear, profile, None)?;
            }
            VKAnalogKeyConfigMode::DKS => {
                self.send_adv_mode(proto, VKAnalogKeyConfigAdvMode::Okmc, profile, okmc)?;
            }
            VKAnalogKeyConfigMode::Gamepad => {
                self.send_adv_mode(proto, VKAnalogKeyConfigAdvMode::Gamepad, profile, None)?;
            }
            VKAnalogKeyConfigMode::Toggle => {
                self.send_adv_mode(proto, VKAnalogKeyConfigAdvMode::Toggle, profile, None)?;
            }
        }
        Ok(())
    }

    fn send_travel(&self, proto: &ViaKeychronProtocol, profile: usize) -> ViaResult<()> {
        let cmd = &VKAnalogCommandId::SetTravel;
        let mut req = cmd.to_cmd();
        let payload = req.payload_mut();
        payload[0] = profile as u8;
        payload[1] = self.data[0] & 0x03; /* only "main" part of mode */
        payload[2] = self.get_actuation_point();
        payload[3] = self.get_rapid_trig_sen();
        payload[4] = self.get_rapid_trig_sen_deact();
        if let Some(index) = self.index {
            payload[5] = 0;
            let col_count = VKAnalogProfileInfo::load(proto)?.get_col_count();
            let row_offset = 6 + (index / col_count) * size_of::<u32>();
            let col = index % col_count;
            payload[row_offset..row_offset + size_of::<u32>()]
                .copy_from_slice((1u32 << col).to_le_bytes().as_slice());
        } else {
            payload[5] = 1;
        }

        let resp = proto.device.raw_hid_send(&req)?;
        cmd.check_reply(&resp)?;
        Ok(())
    }

    fn send_adv_mode(
        &self,
        proto: &ViaKeychronProtocol,
        mode: VKAnalogKeyConfigAdvMode,
        profile: usize,
        okmc: Option<&VKOkmcConfig>,
    ) -> ViaResult<()> {
        let key_index = self.index.ok_or_else(|| {
            ViaError::Protocol("OKMC mode may not be set on global KeyConfig".into())
        })? as u8;

        let col_count = VKAnalogProfileInfo::load(proto)?.get_col_count() as u8;
        let cmd = &VKAnalogCommandId::SetAdvancedMode;
        let mut req = cmd.to_cmd();
        let data = req.payload_mut();
        data[0] = profile as u8;
        data[1] = mode as u8;
        data[2] = key_index / col_count;
        data[3] = key_index % col_count;

        match mode {
            VKAnalogKeyConfigAdvMode::Okmc => {
                let okmc = okmc.ok_or_else(|| {
                    ViaError::Protocol(
                        "VKOkmcConfig is required to set VKKeyConfig in OKMC mode".into(),
                    )
                })?;
                match self.get_adv_mode_info()? {
                    VKAnalogKeyConfigAdvData::Okmc { index } if index == okmc.index => {}
                    VKAnalogKeyConfigAdvData::Okmc { index } => {
                        return Err(ViaError::Protocol(format!(
                            "OkmcConfig index is different from VKKeyConfig: {} != {}",
                            index, okmc.index,
                        )));
                    }
                    _ => panic!("inconsistent adv_mode_inf for DKS mode"), /* should not happen */
                };
                data[4] = okmc.index as u8;
                data[5..5 + VKOkmcConfig::BYTE_SIZE].copy_from_slice(okmc.data.as_ref());
            }
            VKAnalogKeyConfigAdvMode::Gamepad => {
                data[4] = self.get_adv_mode_info()?.into();
            }
            _ => {}
        }

        let resp = proto.device.raw_hid_send(&req)?;
        cmd.check_reply(&resp)?;
        Ok(())
    }
}

impl<'a> TryFrom<&'a [u8]> for VKAnalogKeyConfig<'a> {
    type Error = ViaError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        if value.len() < Self::BYTE_SIZE {
            return Err(ViaError::Protocol(
                "buffer too small for VKAnalogKeyConfigMode".into(),
            ));
        }
        let ret = VKAnalogKeyConfig {
            data: Cow::Borrowed(&value[0..Self::BYTE_SIZE]),
            index: None,
            profile: None,
        };
        ret.get_mode()?;
        Ok(ret)
    }
}

impl std::fmt::Display for VKAnalogKeyConfig<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = f.debug_struct("VKAnalogKeyConfig");

        let mode = self.get_mode();
        match mode {
            Ok(mode) => s.field("mode", &mode),
            Err(_) => s.field("mode", &"invalid"),
        };
        s.field("acutation_point", &self.get_actuation_point())
            .field("rapid_trigger_sensitivity", &self.get_rapid_trig_sen())
            .field(
                "rapid_trigger_sensitivity_deactivation",
                &self.get_rapid_trig_sen_deact(),
            );

        match mode {
            Ok(VKAnalogKeyConfigMode::DKS) | Ok(VKAnalogKeyConfigMode::Gamepad) => {
                s.field("adv_mode_info", &self.get_adv_mode_info().unwrap());
            }
            _ => (),
        };
        s.finish()
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[repr(u8)]
pub enum VKAnalogGamepadData {
    LeftJoystickLeft = 0,
    LeftJoystickRight = 1,
    LeftJoystickBottom = 2,
    LeftJoystickUp = 3,
    ButtonLeftTop = 4,
    ButtonRightTop = 5,
    RigthJoystickLeft = 6,
    RightJoystickRight = 7,
    RightJoystickBottom = 8,
    RightJoystickUp = 9,
    ButtonA = 13,
    ButtonB = 14,
    ButtonX = 15,
    ButtonY = 16,
    ButtonLeftBottom = 17,
    ButtonRightBottom = 18,
    ButtonConfig = 19,
    ButtonMenu = 20,
    ButtonL3 = 21,
    ButtonR3 = 22,
    ButtonUp = 23,
    ButtonDown = 24,
    ButtonLeft = 25,
    ButtonRight = 26,
    ButtonCancel = 27,
}

impl TryFrom<u8> for VKAnalogGamepadData {
    type Error = ViaError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value <= 27 && !(10..=12).contains(&value) {
            Ok(unsafe { std::mem::transmute::<u8, VKAnalogGamepadData>(value) })
        } else {
            Err(ViaError::Protocol(format!(
                "invalid VKAnalogGamepadData: {value}"
            )))
        }
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum VKAnalogKeyConfigAdvData {
    Unknown(u8),
    Gamepad(VKAnalogGamepadData),
    Okmc { index: usize },
}

impl From<u8> for VKAnalogKeyConfigAdvData {
    fn from(value: u8) -> Self {
        VKAnalogKeyConfigAdvData::Unknown(value)
    }
}

impl From<VKAnalogKeyConfigAdvData> for u8 {
    fn from(val: VKAnalogKeyConfigAdvData) -> Self {
        match val {
            VKAnalogKeyConfigAdvData::Unknown(value) => value,
            VKAnalogKeyConfigAdvData::Gamepad(value) => value as u8,
            VKAnalogKeyConfigAdvData::Okmc { index } => index as u8,
        }
    }
}

impl From<VKAnalogGamepadData> for VKAnalogKeyConfigAdvData {
    fn from(value: VKAnalogGamepadData) -> Self {
        VKAnalogKeyConfigAdvData::Gamepad(value)
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum VKAnalogKeyConfigMode {
    Global = 0,
    Regular = 1,
    Rapid = 2,
    /// Also named OneKeyMultipleCommands
    DKS = 3,
    Gamepad = 4,
    /// Long-press switch mode
    Toggle = 5,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
#[repr(u8)]
/// Only used in [VKAnalogKeyConfig::send_mode_okmc]
pub enum VKAnalogKeyConfigAdvMode {
    Clear = 0,
    Okmc = 1,
    Gamepad = 2,
    Toggle = 3,
}

impl TryFrom<u8> for VKAnalogKeyConfigMode {
    type Error = ViaError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value <= VKAnalogKeyConfigMode::Toggle as u8 {
            Ok(unsafe { std::mem::transmute::<u8, VKAnalogKeyConfigMode>(value) })
        } else {
            Err(ViaError::Protocol(format!(
                "invalid VKAnalogKeyConfigMode: {value}"
            )))
        }
    }
}
