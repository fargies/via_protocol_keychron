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

use crate::{VKCommandMaker, VKHsv, VKRgbCommandId, ViaKeychronProtocol, ViaReportData};

bitflags! {
    /// @brief Represents available indicators
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    pub struct VKRgbIndicators: u8 {
        const NUM_LOCK = 0b1;
        const CAPS_LOCK = 0b10;
        const SCROLL_LOCK = 0b100;
        const COMPOSE_LOCK = 0b1000;
        const KANA_LOCK = 0b1_0000;
    }
}

#[derive(Debug)]
pub struct VKRgbIndicatorsConfig {
    data: Vec<u8>,
}

impl VKRgbIndicatorsConfig {
    pub fn load(proto: &ViaKeychronProtocol) -> ViaResult<Self> {
        let cmd = &VKRgbCommandId::GetIndicatorsConfig;
        let resp = proto.raw_send(&cmd.to_cmd())?;
        Self::try_from(resp)
    }

    pub fn send(&self, proto: &ViaKeychronProtocol) -> ViaResult<()> {
        let cmd = &VKRgbCommandId::SetIndicatorsConfig;
        let resp = proto.raw_send(&cmd.to_req(&self.data[1..]))?;

        cmd.check_reply(&resp)?;
        Ok(())
    }

    pub fn get_indicators(&self) -> VKRgbIndicators {
        VKRgbIndicators::from_bits_retain(self.data[0])
    }

    pub fn is_enabled(&self) -> bool {
        self.data[1] != 0
    }

    pub fn enable(&mut self, value: bool) {
        self.data[1] = if value { 1 } else { 0 };
    }

    pub fn get_color(&self) -> VKHsv {
        VKHsv::try_from(&self.data[2..=4]).unwrap()
    }

    pub fn set_color(&mut self, value: &VKHsv) {
        value.serialize(&mut self.data[2..]).unwrap();
    }
}

impl std::fmt::Display for VKRgbIndicatorsConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VKRgbIndicatorsConfig")
            .field("indicators", &self.get_indicators())
            .field("enabled", &self.is_enabled())
            .field("color", &self.get_color())
            .finish()
    }
}

impl TryFrom<ViaReportData> for VKRgbIndicatorsConfig {
    type Error = ViaError;

    fn try_from(value: ViaReportData) -> Result<Self, Self::Error> {
        let payload = VKRgbCommandId::GetIndicatorsConfig.check_reply(&value)?;
        Ok(VKRgbIndicatorsConfig {
            data: payload.into(),
        })
    }
}
