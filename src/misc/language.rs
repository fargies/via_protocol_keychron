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

#![allow(non_camel_case_types)]

use via_protocol::{ViaError, ViaResult};

use crate::{VKCommandMaker, VKMiscCommandId, ViaKeychronProtocol};
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum VKLanguageLayout {
    EN_US = 1,
    JA_JP = 2,
    EN_UK = 3,
    DE = 4,
    DA_DK = 5,
    FR = 6,
    DE_CH = 7,
    ES = 8,
    IT = 9,
    TR = 10,
    SK_UA = 11,
    PT = 12,
    CS = 13,
    SK = 14,
    HU = 15,
    NL = 16,
    KO = 17,
    SL = 18,
}

impl VKLanguageLayout {
    pub const FI_FI: Self = Self::DA_DK;
    pub const IS_IS: Self = Self::DA_DK;
    pub const NO_NO: Self = Self::DA_DK;
    pub const SV_SE: Self = Self::DA_DK;
    pub const FR_CH: Self = Self::DE_CH;
    pub const IT_CH: Self = Self::DE_CH;

    pub fn load(proto: &ViaKeychronProtocol<'_>) -> ViaResult<Self> {
        proto.get_language()
    }

    pub fn save(&self, proto: &ViaKeychronProtocol) -> ViaResult<()> {
        proto.set_language(self)
    }
}

impl TryFrom<u8> for VKLanguageLayout {
    type Error = ViaError;

    #[tracing::instrument(level = "ERROR", err)]
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value == 0 || value > VKLanguageLayout::SL as u8 {
            Err(ViaError::Protocol(format!(
                "invalid VKLanguageLayout: {value}"
            )))
        } else {
            Ok(unsafe { std::mem::transmute::<u8, VKLanguageLayout>(value) })
        }
    }
}

pub trait VKLanguageLayoutTrait {
    fn get_language(&self) -> ViaResult<VKLanguageLayout>;
    fn set_language(&self, layout: &VKLanguageLayout) -> ViaResult<()>;
}

impl VKLanguageLayoutTrait for ViaKeychronProtocol<'_> {
    fn get_language(&self) -> ViaResult<VKLanguageLayout> {
        let cmd = &VKMiscCommandId::LanguageGet;
        let resp = self.device.raw_hid_send(&cmd.to_cmd())?;
        let payload = cmd.check_reply(&resp)?;
        VKLanguageLayout::try_from(payload[0])
    }

    fn set_language(&self, layout: &VKLanguageLayout) -> ViaResult<()> {
        let cmd = &VKMiscCommandId::LanguageSet;
        let resp = self.device.raw_hid_send(&cmd.to_req(&[*layout as u8]))?;

        cmd.check_reply(&resp)?;
        Ok(())
    }
}
