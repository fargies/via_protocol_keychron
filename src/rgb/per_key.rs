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

use crate::{VKCommandMaker, VKHsv, VKRgbCommandId, VKRgbTrait, ViaKeychronProtocol, ViaReportData};

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
        let cmd = &VKRgbCommandId::PerKeyRgbGetType;
        let resp = proto.device.raw_hid_send(&cmd.to_cmd())?;
        Self::try_from(resp)
    }

    /// @brief send [VKRgbPerKeyType] to device
    /// @details use [VKRgb::save] to persist changes
    pub fn send(&self, proto: &ViaKeychronProtocol) -> ViaResult<()> {
        let cmd = &VKRgbCommandId::PerKeyRgbSetType;
        let resp = proto.device.raw_hid_send(&cmd.to_req(&[*self as u8]))?;

        cmd.check_reply(&resp)?;
        Ok(())
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

    pub fn load(proto: &ViaKeychronProtocol) -> ViaResult<Self> {
        let key_count = proto.get_led_count()?;
        let cmd = &VKRgbCommandId::PerKeyRgbGetColor;

        let mut ret = Self {
            start: 0,
            config: Vec::with_capacity(key_count)
        };

        let mut start = 0;
        while start < key_count {
            let count = (key_count - start).min(Self::MAX_REQ_ITEMS);
            let resp = proto.device.raw_hid_send(&cmd.to_req(&[start as u8, count as u8]))?;
            let value = cmd.check_reply(&resp)?;
            for i in 0..count {
                ret.config.push(VKHsv::try_from(&value[i * VKHsv::BYTE_SIZE..])?)
            }
            start += count;
        }
        Ok(ret)
    }

    pub fn load_part(proto: &ViaKeychronProtocol, start: u8, count: u8) -> ViaResult<Self> {
        let cmd = &VKRgbCommandId::PerKeyRgbGetColor;
        let mut ret = Self {
            start,
            config: Vec::with_capacity(count as usize)
        };

        let resp = proto.device.raw_hid_send(&cmd.to_req(&[start, count]))?;
        let value = cmd.check_reply(&resp)?;
        for i in 0..count as usize {
            ret.config.push(VKHsv::try_from(&value[i * VKHsv::BYTE_SIZE..])?)
        }
        Ok(ret)
    }

    pub fn send(&self, proto: &ViaKeychronProtocol) -> ViaResult<()> {
        let cmd = &VKRgbCommandId::PerKeyRgbSetColor;

        let mut data = Vec::with_capacity(2 + VKHsv::BYTE_SIZE * self.config.len().min(Self::MAX_REQ_ITEMS));
        let mut start = 0;
        while start < self.config.len() {
            let count = (self.config.len() - start).min(Self::MAX_REQ_ITEMS);
            data.clear();
            data.push(start as u8 + self.start);
            data.push(count as u8);
            data.resize(2 + VKHsv::BYTE_SIZE * count, 0);
            for (index, hsv) in self.config[start..start+count].iter().enumerate() {
                hsv.serialize(&mut data[(2 + index * 3)..])?;
            }
            let resp = proto.device.raw_hid_send(&cmd.to_req(data.as_ref()))?;
            cmd.check_reply(&resp)?;
            start += count;
        }
        Ok(())
    }
}

pub trait VKRgbPerKeyTrait {
    fn get_per_key_type(&self) -> ViaResult<VKRgbPerKeyType>;
    fn set_per_key_type(&self, value: &VKRgbPerKeyType) -> ViaResult<()>;

    fn get_per_key_led_color(&self) -> ViaResult<VKRgbPerKeyConfig>;
    fn set_per_key_led_color(&self, value: &VKRgbPerKeyConfig) -> ViaResult<()>;
}

impl VKRgbPerKeyTrait for ViaKeychronProtocol<'_> {
    fn get_per_key_type(&self) -> ViaResult<VKRgbPerKeyType> {
        VKRgbPerKeyType::load(self)
    }

    fn set_per_key_type(&self, value: &VKRgbPerKeyType) -> ViaResult<()> {
        value.send(self)
    }

    fn get_per_key_led_color(&self) -> ViaResult<VKRgbPerKeyConfig> {
        VKRgbPerKeyConfig::load(self)

    }

    fn set_per_key_led_color(&self, value: &VKRgbPerKeyConfig) -> ViaResult<()> {
        value.send(self)
    }
}
