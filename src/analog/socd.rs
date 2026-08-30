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

use crate::{VKAnalogCommandId, VKCommandMaker, ViaKeychronProtocol};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum VKSocdType {
    None = 0,
    // TODO: check difference with [DeeperTravelSingle]
    DeeperTravel,
    /// @brief deepest pressed key has priority
    DeeperTravelSingle,
    /// @brief last key pressed has priority
    LastKeystroke,
    /// @brief first key has priority
    Key1,
    /// @brief second key has priority
    Key2,
    /// @brief no priority
    /// @details no key should be pressed when both are hit simultaneously
    Neutral,
}

impl TryFrom<u8> for VKSocdType {
    type Error = ViaError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > VKSocdType::Neutral as u8 {
            return Err(ViaError::Protocol(format!("invalid VKSocdType: {value}")));
        }
        Ok(unsafe { std::mem::transmute::<u8, Self>(value) })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum VKSocdKey {
    Key1,
    Key2,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct VKSocdConfig<'a> {
    pub data: Cow<'a, [u8]>,
    pub index: Option<usize>,
    pub profile: Option<usize>,
    col_count: usize,
}

impl VKSocdConfig<'_> {
    pub const BYTE_SIZE: usize = 3;

    pub fn is_empty(&self) -> bool {
        self.get_type().map_or(true, |t| t == VKSocdType::None)
    }

    /// @brief retrieve the key index
    pub fn get_key(&self, key: VKSocdKey) -> usize {
        let data = match key {
            VKSocdKey::Key1 => self.data[0],
            VKSocdKey::Key2 => self.data[1],
        };
        (((data & 0x7) * self.col_count as u8) | (data >> 3)) as usize
    }

    pub fn set_key(&mut self, key: VKSocdKey, key_index: usize) {
        let (col, row) = (
            (key_index % self.col_count) as u8,
            (key_index / self.col_count) as u8,
        );
        let data = self.data.to_mut();
        match key {
            VKSocdKey::Key1 => data[0] = row | (col << 3),
            VKSocdKey::Key2 => data[1] = row | (col << 3),
        }
    }

    pub fn iter_key(&self) -> impl Iterator<Item = usize> {
        [VKSocdKey::Key1, VKSocdKey::Key2]
            .into_iter()
            .map(|k| self.get_key(k))
    }

    /// @brief get the VKSocdType
    pub fn get_type(&self) -> ViaResult<VKSocdType> {
        VKSocdType::try_from(self.data[2])
    }

    pub fn set_type(&mut self, key_type: VKSocdType) {
        self.data.to_mut()[2] = key_type as u8;
    }

    pub fn send(&self, proto: &ViaKeychronProtocol) -> ViaResult<()> {
        let cmd = &VKAnalogCommandId::SetSOCD;
        let mut req = cmd.to_cmd();
        let payload = req.payload_mut();
        payload[0] = self.profile.ok_or_else(|| {
            ViaError::Protocol(
                "destination profile must be set to invoke VKSocdConfig::send".into(),
            )
        })? as u8;
        payload[1] = self.data[0] & 0x7;
        payload[2] = self.data[0] >> 3;
        payload[3] = self.data[1] & 0x7;
        payload[4] = self.data[1] >> 3;
        payload[5] = self.index.ok_or_else(|| {
            ViaError::Protocol("Socd index must be set to invoke VKSocdConfig::send".into())
        })? as u8;
        payload[6] = self.data[2];
        let resp = proto.raw_send(&req)?;
        cmd.check_reply(&resp)?;
        Ok(())
    }
}

impl Display for VKSocdConfig<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = f.debug_struct("VKSocdConfig");
        s.field("key1", &self.get_key(VKSocdKey::Key1))
            .field("key2", &self.get_key(VKSocdKey::Key2));

        match self.get_type() {
            Ok(t) => s.field("type", &t),
            Err(_) => s.field("type", &"invalid"),
        };
        s.finish()
    }
}

impl<'a> TryFrom<(&'a [u8], usize)> for VKSocdConfig<'a> {
    type Error = ViaError;

    fn try_from((value, col_count): (&'a [u8], usize)) -> Result<Self, Self::Error> {
        if value.len() < Self::BYTE_SIZE {
            Err(ViaError::Protocol(
                "buffer too small for VKSocdConfig".into(),
            ))
        } else {
            let ret = VKSocdConfig {
                data: Cow::Borrowed(&value[0..Self::BYTE_SIZE]),
                index: None,
                profile: None,
                col_count,
            };
            ret.get_type()?; // sanity check
            Ok(ret)
        }
    }
}
