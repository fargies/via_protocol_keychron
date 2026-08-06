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

use bitflags::bitflags;
use via_protocol::{ViaError, ViaResult};

use crate::utils::{DebugAsDisplay, DebugIter};

use std::{borrow::Cow, fmt::Display, mem::size_of};

pub type VKKeyCode = u16;

#[derive(Debug, Clone)]
pub struct VKOkmcConfig<'a> {
    pub data: Cow<'a, [u8]>,
    pub index: usize
}


impl VKOkmcConfig<'_> {
    pub const BYTE_SIZE: usize = VKOkmcTravel::BYTE_SIZE + 16;
    pub const KEYCODE_COUNT: usize = 4;

    pub fn is_empty(&self) -> bool {
        self.get_travel_config().map_or(true, |c| c.is_empty())
    }

    pub fn get_travel_config<'a>(&'a self) -> ViaResult<VKOkmcTravel<'a>> {
        VKOkmcTravel::try_from(self.data.as_ref())
    }

    /// @brief get keycode value
    pub fn get_keycode(&self, index: usize) -> ViaResult<u16> {
        if index >= 4 {
            return Err(ViaError::Protocol(format!(
                "invalid index for Okmc: {index} (max:4)"
            )));
        }
        let pos = VKOkmcTravel::BYTE_SIZE + (index * 2);
        Ok(u16::from_le_bytes(
            *self.data[pos..pos + 2].as_array::<2>().unwrap(),
        ))
    }

    /// @brief iterate over keycodes
    pub fn iter_keycodes(&self) -> impl Iterator<Item = u16> {
        (0..Self::KEYCODE_COUNT).filter_map(|i| self.get_keycode(i).ok())
    }

    pub fn get_action(&self, key: usize, activation: VKOkmcActivation) -> ViaResult<VKOkmcAction> {
        let pos = VKOkmcTravel::BYTE_SIZE
            + (size_of::<VKKeyCode>() * Self::KEYCODE_COUNT)
            + (size_of::<u16>() * key);
        Ok(match activation {
            VKOkmcActivation::ShallowActivation => {
                VKOkmcAction::from_bits_retain(self.data[pos] & 0x0F)
            }
            VKOkmcActivation::ShallowDeactivation => {
                VKOkmcAction::from_bits_retain(self.data[pos] >> 4)
            }
            VKOkmcActivation::DeepActivation => {
                VKOkmcAction::from_bits_retain(self.data[pos + 1] & 0x0F)
            }
            VKOkmcActivation::DeepDeactivation => {
                VKOkmcAction::from_bits_retain(self.data[pos + 1] >> 4)
            }
        })
    }
}

impl Display for VKOkmcConfig<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = f.debug_struct("VKAnalogOkmcConfig");
        s.field("index", &self.index);

        let travel = self.get_travel_config();
        match travel {
            Ok(travel) => s.field("travel", &DebugAsDisplay::Borrowed(&travel)),
            Err(_) => s.field("travel", &"invalid"),
        };
        s.field("keycodes", &DebugIter::new(self.iter_keycodes()));
        s.field(
            "actions",
            &DebugIter::new((0..Self::KEYCODE_COUNT).map(|k| {
                VKOkmcActivation::iter()
                    .map(|act| DebugAsDisplay::Owned(self.get_action(k, act).unwrap()))
                    .collect::<Vec<_>>()
            })),
        );

        s.finish()
    }
}

/// @brief build Okmc from raw data and index
impl<'a> TryFrom<(&'a [u8], usize)> for VKOkmcConfig<'a> {
    type Error = ViaError;

    fn try_from((value, index): (&'a [u8], usize)) -> Result<Self, Self::Error> {
        if value.len() < Self::BYTE_SIZE {
            Err(ViaError::Protocol(
                "buffer too small for VKOkmcConfig".into(),
            ))
        } else {
            Ok(VKOkmcConfig {
                data: Cow::Borrowed(&value[0..Self::BYTE_SIZE]),
                index
            })
        }
    }
}


#[derive(Debug, Eq, PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum VKOkmcActivation {
    ShallowActivation = 0,
    ShallowDeactivation = 1,
    DeepActivation = 2,
    DeepDeactivation = 3,
}

impl VKOkmcActivation {
    pub fn iter() -> impl Iterator<Item = VKOkmcActivation> {
        (0..).map_while(|i| Self::try_from(i).ok())
    }
}

impl TryFrom<u8> for VKOkmcActivation {
    type Error = ViaError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > Self::DeepDeactivation as u8 {
            return Err(ViaError::Protocol(format!(
                "invalid activation value: {value} (max: 4)"
            )));
        }
        Ok(unsafe { std::mem::transmute::<u8, Self>(value) })
    }
}

bitflags! {
    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    pub struct VKOkmcAction: u8 {
        const PreRealase = 0b1;
        const Press = 0b10;
        const Release = 0b100;
        const PostPress = 0b1000;
    }
}

impl Display for VKOkmcAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("(")?;

        let mut first = true;
        for (name, _) in self.iter_names() {
            if !first {
                f.write_str(", ")?;
            }
            name.fmt(f)?;
            first = false;
        }
        f.write_str(")")
    }
}

#[derive(Clone, Debug)]
pub struct VKOkmcTravel<'a> {
    pub data: Cow<'a, [u8]>,
}

impl VKOkmcTravel<'_> {
    pub const BYTE_SIZE: usize = 3;

    /// @brief get shallow activation distance in 0.1mm units
    pub fn get_depth(&self, action: VKOkmcActivation) -> u8 {
        match action {
            VKOkmcActivation::ShallowActivation => self.data[0] & 0x3F,
            VKOkmcActivation::ShallowDeactivation => self.data[0] >> 6 | (self.data[1] & 0x0F) << 2,
            VKOkmcActivation::DeepActivation => self.data[1] >> 4 | (self.data[2] & 0x03) << 4,
            VKOkmcActivation::DeepDeactivation => self.data[2] >> 2,
        }
    }

    pub fn iter_depth(&self) -> impl Iterator<Item = u8> {
        VKOkmcActivation::iter().map(|a| self.get_depth(a))
    }

    pub fn set_depth(&mut self, action: VKOkmcActivation, value: u8) {
        let data = self.data.to_mut();
        match action {
            VKOkmcActivation::ShallowActivation => {
                data[0] &= 0xC0;
                data[0] |= value & 0x3F;
            }
            VKOkmcActivation::ShallowDeactivation => {
                data[0] &= 0x3F;
                data[0] |= value << 6;
                data[1] &= 0xF0;
                data[1] |= (value >> 2) & 0x0F
            }
            VKOkmcActivation::DeepActivation => {
                data[1] &= 0x0F;
                data[1] |= value << 4;
                data[2] &= 0xFC;
                data[2] |= value & 0x03;
            }
            VKOkmcActivation::DeepDeactivation => {
                data[2] &= 0x03;
                data[2] |= value << 2;
            }
        }
    }

    /// @brief test if VKOkmcTravel is empty
    /// @details it is empty when all depths are zeroed
    pub fn is_empty(&self) -> bool {
        self.iter_depth().all(|v| v == 0)
    }
}

impl<'a> TryFrom<&'a [u8]> for VKOkmcTravel<'a> {
    type Error = ViaError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        if value.len() < Self::BYTE_SIZE {
            return Err(ViaError::Protocol(
                "buffer too small for VKAnalogOkmcTravel".into(),
            ));
        }
        let ret = VKOkmcTravel {
            data: Cow::Borrowed(&value[0..Self::BYTE_SIZE]),
        };
        Ok(ret)
    }
}

impl Display for VKOkmcTravel<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = f.debug_struct("VKAnalogOkmcTravel");

        for travel in VKOkmcActivation::iter() {
            s.field(
                format!("{travel:?} travel").as_str(),
                &self.get_depth(travel),
            );
        }
        s.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use via_protocol::ViaResult;

    #[test]
    fn okmc_travel_accessors() -> ViaResult<()> {
        let mut okmc = VKOkmcTravel::try_from([0, 0, 0].as_slice())?;
        okmc.set_depth(VKOkmcActivation::ShallowActivation, 0x12);
        okmc.set_depth(VKOkmcActivation::ShallowDeactivation, 0x34);
        okmc.set_depth(VKOkmcActivation::DeepActivation, 0x15);
        okmc.set_depth(VKOkmcActivation::DeepDeactivation, 0x36);
        tracing::info!(?okmc.data);
        assert_eq!(0x12, okmc.get_depth(VKOkmcActivation::ShallowActivation));
        assert_eq!(0x34, okmc.get_depth(VKOkmcActivation::ShallowDeactivation));
        assert_eq!(0x15, okmc.get_depth(VKOkmcActivation::DeepActivation));
        assert_eq!(0x36, okmc.get_depth(VKOkmcActivation::DeepDeactivation));

        okmc.set_depth(VKOkmcActivation::ShallowDeactivation, 0);
        okmc.set_depth(VKOkmcActivation::DeepDeactivation, 0);
        assert_eq!(0x12, okmc.get_depth(VKOkmcActivation::ShallowActivation));
        assert_eq!(0, okmc.get_depth(VKOkmcActivation::ShallowDeactivation));
        assert_eq!(0x15, okmc.get_depth(VKOkmcActivation::DeepActivation));
        assert_eq!(0, okmc.get_depth(VKOkmcActivation::DeepDeactivation));

        okmc.set_depth(VKOkmcActivation::ShallowActivation, 0xFF);
        okmc.set_depth(VKOkmcActivation::DeepActivation, 0xFF);
        assert_eq!(0x3F, okmc.get_depth(VKOkmcActivation::ShallowActivation));
        assert_eq!(0, okmc.get_depth(VKOkmcActivation::ShallowDeactivation));
        assert_eq!(0x3F, okmc.get_depth(VKOkmcActivation::DeepActivation));
        assert_eq!(0, okmc.get_depth(VKOkmcActivation::DeepDeactivation));
        Ok(())
    }
}
