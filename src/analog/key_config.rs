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

use std::{borrow::Cow, fmt::Display};

use via_protocol::{ViaError, ViaResult};

pub struct VKAnalogKeyConfig<'a> {
    pub data: Cow<'a, [u8]>,
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
    Okmc { index: u8 },
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
                index: self.data[3],
            }),
            Ok(VKAnalogKeyConfigMode::Gamepad) => {
                VKAnalogGamepadData::try_from(self.data[3]).map(VKAnalogKeyConfigAdvData::Gamepad)
            }
            Ok(mode) => Err(ViaError::Protocol(format!("no data for mode: {mode:?}"))),
            Err(e) => Err(e),
        }
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
            data: Cow::Borrowed(value),
        };
        ret.get_mode()?;
        Ok(ret)
    }
}

impl Display for VKAnalogKeyConfig<'_> {
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
            _ => ()
        };
        s.finish()
    }
}
