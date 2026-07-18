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

use crate::{VKCommandMaker, VKRgbCommandId, ViaKeychronProtocol, ViaReportData};

#[derive(Debug, Clone)]
pub struct VKRgbMixedInfo {
    data: Vec<u8>,
}

impl VKRgbMixedInfo {
    pub fn load(proto: &ViaKeychronProtocol) -> ViaResult<Arc<Self>> {
        proto.get_mixed_info()
    }

    /// @brief get number of layers for mixed mode
    pub fn get_layers(&self) -> u8 {
        self.data[0]
    }

    /// @brief get number of effects per layer in mixed mode
    pub fn get_effects_per_layer(&self) -> u8 {
        self.data[1]
    }
}

impl std::fmt::Display for VKRgbMixedInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VKRgbMixedInfo")
            .field("layers", &self.get_layers())
            .field("effects_per_layer", &self.get_effects_per_layer())
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

    pub fn load(proto: &ViaKeychronProtocol, start: u8, count: u8) -> ViaResult<Self> {
        proto.get_mixed_regions(start, count)
    }

    pub fn send(&self, proto: &ViaKeychronProtocol) -> ViaResult<()> {
        proto.set_mixed_regions(self)
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

    pub fn load(proto: &ViaKeychronProtocol, region: u8, start: u8, count: u8) -> ViaResult<Self> {
        proto.get_mixed_effects(region, start, count)
    }

    pub fn try_from(value: ViaReportData, region: u8, start: u8, count: u8) -> ViaResult<Self> {
        if count > Self::MAX_REQ_ITEMS as u8 {
            return Err(ViaError::Protocol(format!(
                "invalid effects count for VKRgbMixedEffectList: {count}"
            )));
        }

        let value = VKRgbCommandId::MixedEffectRgbSetEffectList.check_reply(&value)?;
        let mut ret = Self {
            region,
            start,
            effects: Vec::with_capacity(count as usize),
        };
        for i in 0..count as usize {
            ret.effects
                .push(VKRgbMixedEffect::try_from(&value[i * 8..])?);
        }
        Ok(ret)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct VKRgbMixedEffect {
    pub effect: u8,
    pub hue: u8,
    pub satuation: u8,
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
                satuation: value[2],
                speed: value[3],
                time: u32::from_le_bytes(*value[4..8].as_array::<4>().unwrap()),
            })
        }
    }
}

pub trait VKRgbMixedTrait {
    fn get_mixed_info(&self) -> ViaResult<Arc<VKRgbMixedInfo>>;
    fn get_mixed_regions(&self, start: u8, count: u8) -> ViaResult<VKRgbMixedRegions>;
    fn set_mixed_regions(&self, regions: &VKRgbMixedRegions) -> ViaResult<()>;
    fn get_mixed_effects(
        &self,
        region: u8,
        start: u8,
        count: u8,
    ) -> ViaResult<VKRgbMixedEffectList>;
}

impl VKRgbMixedTrait for ViaKeychronProtocol<'_> {
    fn get_mixed_info(&self) -> ViaResult<Arc<VKRgbMixedInfo>> {
        if let Some(info) = self.get_info().mixed_info.as_ref() {
            Ok(Arc::clone(info))
        } else {
            let cmd = &VKRgbCommandId::MixedEffectRgbGetInfo;
            let resp = self.device.raw_hid_send(&cmd.to_cmd())?;
            VKRgbMixedInfo::try_from(resp)
                .map(Arc::new)
                .inspect(|info| {
                    Arc::make_mut(&mut self.get_info_mut())
                        .mixed_info
                        .replace(Arc::clone(info));
                })
        }
    }

    fn get_mixed_regions(&self, start: u8, count: u8) -> ViaResult<VKRgbMixedRegions> {
        let cmd = &VKRgbCommandId::MixedEffectRgbGetRegions;
        let resp = self.device.raw_hid_send(&cmd.to_req(&[start, count]))?;
        VKRgbMixedRegions::try_from(resp, start, count)
    }

    fn set_mixed_regions(&self, value: &VKRgbMixedRegions) -> ViaResult<()> {
        let cmd = &VKRgbCommandId::MixedEffectRgbSetRegions;
        if value.regions.len() > VKRgbMixedRegions::MAX_REQ_ITEMS {
            return Err(ViaError::Protocol(format!(
                "too many VKRgbMixedRegions items, (max=29): {}",
                value.regions.len()
            )));
        }
        let mut data = Vec::with_capacity(2 + value.regions.len());
        data.push(value.start);
        data.push(value.regions.len() as u8);
        data.splice(2.., value.regions.iter().cloned());
        tracing::trace!(req = ?data);
        let resp = self.device.raw_hid_send(&cmd.to_req(data.as_ref()))?;
        cmd.check_reply(&resp)?;
        Ok(())
    }

    fn get_mixed_effects(
        &self,
        region: u8,
        start: u8,
        count: u8,
    ) -> ViaResult<VKRgbMixedEffectList> {
        let cmd = &VKRgbCommandId::MixedEffectRgbGetEffectList;
        let resp = self
            .device
            .raw_hid_send(&cmd.to_req(&[region, start, count]))?;
        VKRgbMixedEffectList::try_from(resp, region, start, count)
    }
}
