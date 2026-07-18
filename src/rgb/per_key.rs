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

use via_protocol::{ViaError, ViaResult};

use crate::{VKCommandMaker, VKHsv, VKRgbCommandId, ViaKeychronProtocol, ViaReportData};

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum VKRgbPerKeyType {
    Solid = 0,
    Breathing = 1,
    ReactiveSimple = 2,
    ReactiveMultiWide = 3,
    ReactiveSplash = 4,
}

impl VKRgbPerKeyType {
    /// @brief load [VKRgbPerKeyType] from device
    pub fn load(proto: &ViaKeychronProtocol) -> ViaResult<VKRgbPerKeyType> {
        proto.get_per_key_type()
    }

    /// @brief send [VKRgbPerKeyType] to device
    /// @details use [VKRgb::save] to persist changes
    pub fn send(&self, proto: &ViaKeychronProtocol) -> ViaResult<()> {
        proto.set_per_key_type(self)
    }
}

impl TryFrom<u8> for VKRgbPerKeyType {
    type Error = ViaError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > Self::ReactiveSplash as u8 {
            Err(ViaError::Protocol(format!(
                "invalid VKRgbPerKeyType: {value}"
            )))
        } else {
            Ok(unsafe { std::mem::transmute::<u8, VKRgbPerKeyType>(value) })
        }
    }
}

impl TryFrom<ViaReportData> for VKRgbPerKeyType {
    type Error = ViaError;

    fn try_from(value: ViaReportData) -> Result<Self, Self::Error> {
        let payload = VKRgbCommandId::PerKeyRgbGetType.check_reply(&value)?;
        VKRgbPerKeyType::try_from(payload[0])
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct VKRgbPerKeyConfig {
    pub start: u8,
    pub config: Vec<VKHsv>,
}

impl VKRgbPerKeyConfig {
    pub const MAX_REQ_ITEMS: usize = 9;

    pub fn load(proto: &ViaKeychronProtocol, start: u8, count: u8) -> ViaResult<Self> {
        proto.get_per_key_led_color(start, count)
    }

    pub fn send(&self, proto: &ViaKeychronProtocol) -> ViaResult<()> {
        proto.set_per_key_led_color(self)
    }

    pub fn try_from(value: ViaReportData, start: u8, count: u8) -> ViaResult<Self> {
        let value = VKRgbCommandId::PerKeyRgbGetColor.check_reply(&value)?;
        if count > Self::MAX_REQ_ITEMS as u8 {
            return Err(ViaError::Protocol(format!(
                "invalid key-count for VKRgbPerKeyConfig: {count}"
            )));
        }

        let mut ret = Self {
            start,
            config: Vec::with_capacity(count as usize),
        };

        for i in 0..count as usize {
            ret.config.push(VKHsv::try_from(&value[i * 3..])?)
        }
        Ok(ret)
    }
}

pub trait VKRgbPerKeyTrait {
    fn get_per_key_type(&self) -> ViaResult<VKRgbPerKeyType>;
    fn set_per_key_type(&self, value: &VKRgbPerKeyType) -> ViaResult<()>;

    fn get_per_key_led_color(&self, start: u8, count: u8) -> ViaResult<VKRgbPerKeyConfig>;
    fn set_per_key_led_color(&self, value: &VKRgbPerKeyConfig) -> ViaResult<()>;
}

impl VKRgbPerKeyTrait for ViaKeychronProtocol<'_> {
    fn get_per_key_type(&self) -> ViaResult<VKRgbPerKeyType> {
        let cmd = &VKRgbCommandId::PerKeyRgbGetType;
        let resp = self.device.raw_hid_send(&cmd.to_cmd())?;
        VKRgbPerKeyType::try_from(resp)
    }

    fn set_per_key_type(&self, value: &VKRgbPerKeyType) -> ViaResult<()> {
        let cmd = &VKRgbCommandId::PerKeyRgbSetType;
        let resp = self.device.raw_hid_send(&cmd.to_req(&[*value as u8]))?;

        cmd.check_reply(&resp)?;
        Ok(())
    }

    fn get_per_key_led_color(&self, start: u8, count: u8) -> ViaResult<VKRgbPerKeyConfig> {
        let cmd = &VKRgbCommandId::PerKeyRgbGetColor;
        let resp = self.device.raw_hid_send(&cmd.to_req(&[start, count]))?;
        VKRgbPerKeyConfig::try_from(resp, start, count)
    }

    fn set_per_key_led_color(&self, value: &VKRgbPerKeyConfig) -> ViaResult<()> {
        let cmd = &VKRgbCommandId::PerKeyRgbSetColor;
        if value.config.len() > VKRgbPerKeyConfig::MAX_REQ_ITEMS {
            return Err(ViaError::Protocol(format!(
                "too many VKRgbPerKeyConfig items, (max=9): {}",
                value.config.len()
            )));
        }
        let mut data = Vec::with_capacity(2 + 3 * value.config.len());
        data.push(value.start);
        data.push(value.config.len() as u8);
        data.resize(data.len() + 3 * value.config.len(), 0);
        for (index, hsv) in value.config.iter().enumerate() {
            hsv.serialize(&mut data[(2 + index * 3)..])?;
        }
        let resp = self.device.raw_hid_send(&cmd.to_req(data.as_ref()))?;
        cmd.check_reply(&resp)?;
        Ok(())
    }
}
