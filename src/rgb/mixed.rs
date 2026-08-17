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

use via_protocol::{ViaError, ViaResult};

use crate::{VKCommandMaker, VKRgbCommandId, VKRgbInfo, ViaKeychronProtocol, ViaReportData, VKRgbTrait};

#[derive(Debug, Clone)]
pub struct VKRgbMixedInfo {
    data: Vec<u8>,
}

impl VKRgbMixedInfo {
    pub fn load(proto: &ViaKeychronProtocol) -> ViaResult<Arc<Self>> {
        VKRgbInfo::load(proto).map(|info| Arc::clone(&info.mixed))
    }

    /// @brief get number of layers for mixed mode
    pub fn get_region_count(&self) -> u8 {
        self.data[0]
    }

    /// @brief get number of effects per layer in mixed mode
    pub fn get_effects_per_region(&self) -> u8 {
        self.data[1]
    }
}

impl std::fmt::Display for VKRgbMixedInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VKRgbMixedInfo")
            .field("layers", &self.get_region_count())
            .field("effects_per_layer", &self.get_effects_per_region())
            .finish()
    }
}

impl TryFrom<ViaReportData> for VKRgbMixedInfo {
    type Error = ViaError;

    fn try_from(value: ViaReportData) -> Result<Self, Self::Error> {
        let payload = VKRgbCommandId::MixedEffectRgbGetInfo.check_reply(&value)?;
        Ok(VKRgbMixedInfo {
            data: payload.into(),
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct VKRgbMixedRegions {
    pub start: u8,
    pub regions: Vec<u8>,
}

impl VKRgbMixedRegions {
    pub const MAX_REQ_ITEMS: usize = 28;

    pub fn load(proto: &ViaKeychronProtocol) -> ViaResult<Self> {
        let key_count = proto.get_led_count()?;
        let cmd = &VKRgbCommandId::MixedEffectRgbGetRegions;

        let mut ret = Self {
            start: 0,
            regions: Vec::with_capacity(key_count),
        };
        let mut start = 0;
        while start < key_count {
            let count = (key_count - start).min(Self::MAX_REQ_ITEMS);
            let resp = proto
                .raw_send(&cmd.to_req(&[start as u8, count as u8]))?;
            ret.regions.extend(&cmd.check_reply(&resp)?[0..count]);
            start += count;
        }
        Ok(ret)
    }

    pub fn load_part(proto: &ViaKeychronProtocol, start: u8, count: u8) -> ViaResult<Self> {
        let cmd = &VKRgbCommandId::MixedEffectRgbGetRegions;
        let resp = proto.raw_send(&cmd.to_req(&[start, count]))?;
        Self::try_from(resp, start, count)
    }

    pub fn send(&self, proto: &ViaKeychronProtocol) -> ViaResult<()> {
        let cmd = &VKRgbCommandId::MixedEffectRgbSetRegions;

        let mut data =
            Vec::with_capacity(2 + self.regions.len().min(VKRgbMixedRegions::MAX_REQ_ITEMS));
        let mut start = 0;
        while start < self.regions.len() {
            let count = (self.regions.len() - start).min(Self::MAX_REQ_ITEMS);
            data.clear();
            data.push(self.start + start as u8);
            data.push(count as u8);
            data.extend(&self.regions[start..start + count]);
            tracing::trace!(req = ?data);
            let resp = proto.raw_send(&cmd.to_req(data.as_ref()))?;
            cmd.check_reply(&resp)?;
            start += count;
        }
        Ok(())
    }

    pub fn try_from(value: ViaReportData, start: u8, count: u8) -> ViaResult<Self> {
        if count > Self::MAX_REQ_ITEMS as u8 {
            return Err(ViaError::Protocol(format!(
                "invalid regions count for VKRgbMixedRegions: {count}"
            )));
        }

        let value = VKRgbCommandId::MixedEffectRgbGetRegions.check_reply(&value)?;
        Ok(Self {
            start,
            regions: value[0..count as usize].to_vec(),
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct VKRgbMixedEffectList {
    pub region: u8,
    pub start: u8,
    pub effects: Vec<VKRgbMixedEffect>,
}

impl VKRgbMixedEffectList {
    pub const MAX_REQ_ITEMS: usize = 3;

    pub fn load(proto: &ViaKeychronProtocol, region: u8) -> ViaResult<Self> {
        let effects_count = VKRgbMixedInfo::load(proto)?.get_effects_per_region() as usize;
        let cmd = &VKRgbCommandId::MixedEffectRgbGetEffectList;
        let mut ret = Self {
            region,
            start: 0,
            effects: Vec::with_capacity(effects_count),
        };
        let mut start = 0;
        while start < effects_count {
            let count = (effects_count - start).min(Self::MAX_REQ_ITEMS);
            let resp =
                proto
                    .raw_send(&cmd.to_req(&[region, start as u8, count as u8]))?;
            let payload = cmd.check_reply(&resp)?;
            for i in 0..count {
                ret.effects.push(VKRgbMixedEffect::try_from(
                    &payload[i * VKRgbMixedEffect::BYTE_SIZE..],
                )?);
            }
            start += count;
        }
        Ok(ret)
    }

    pub fn load_part(proto: &ViaKeychronProtocol, region: u8, start: u8, count: u8) -> ViaResult<Self> {
        let cmd = &VKRgbCommandId::MixedEffectRgbGetEffectList;
        let resp = proto
            .raw_send(&cmd.to_req(&[region, start, count]))?;
        let mut ret = Self {
            region,
            start,
            effects: Vec::with_capacity(count as usize),
        };
        let payload = cmd.check_reply(&resp)?;
        for i in 0..count as usize {
            ret.effects.push(VKRgbMixedEffect::try_from(
                &payload[i * VKRgbMixedEffect::BYTE_SIZE..],
            )?);
        }
        Ok(ret)
    }

    pub fn send(&self, proto: &ViaKeychronProtocol) -> ViaResult<()> {
        let cmd = &VKRgbCommandId::MixedEffectRgbSetEffectList;
        let mut data =
            Vec::with_capacity(3 + VKRgbMixedEffect::BYTE_SIZE * self.effects.len().min(Self::MAX_REQ_ITEMS));

        let mut start = 0;
        while start < self.effects.len() {
            let count = (self.effects.len() - start).min(Self::MAX_REQ_ITEMS);
            data.clear();
            data.push(self.region);
            data.push(self.start + start as u8);
            data.push(count as u8);
            data.resize(3 + VKRgbMixedEffect::BYTE_SIZE * count, 0);
            for i in 0..count {
                self.effects[start + i].serialize(&mut data[3 + VKRgbMixedEffect::BYTE_SIZE * i..])?;
            }
            tracing::trace!(req = ?data);
            let resp = proto.raw_send(&cmd.to_req(data.as_ref()))?;
            cmd.check_reply(&resp)?;
            start += count;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct VKRgbMixedEffect {
    pub effect: u8,
    pub hue: u8,
    pub saturation: u8,
    pub speed: u8,
    pub time: u32,
}

impl TryFrom<&[u8]> for VKRgbMixedEffect {
    type Error = ViaError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < 8 {
            Err(ViaError::Protocol(
                "invalid mixed-effect value: buffer too small".into(),
            ))
        } else {
            Ok(Self {
                effect: value[0],
                hue: value[1],
                saturation: value[2],
                speed: value[3],
                time: u32::from_le_bytes(*value[4..8].as_array::<4>().unwrap()),
            })
        }
    }
}

impl VKRgbMixedEffect {
    pub const BYTE_SIZE: usize = 8;

    pub fn serialize(&self, buffer: &mut [u8]) -> ViaResult<()> {
        if buffer.len() < Self::BYTE_SIZE {
            Err(ViaError::Protocol(
                "buffer too small to serialize VKRgbMixedEffect".into(),
            ))
        } else {
            buffer[0] = self.effect;
            buffer[1] = self.hue;
            buffer[2] = self.saturation;
            buffer[3] = self.speed;
            buffer[4..8].copy_from_slice(self.time.to_le_bytes().as_slice());
        Ok(())
        }

    }
}
