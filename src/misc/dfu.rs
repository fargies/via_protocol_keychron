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

use crate::{VKCommandMaker, VKMiscCommandId, ViaKeychronProtocol, ViaReportData};

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[repr(u8)]
pub enum VKDfuInfoType {
    End = 0,
    ChipName = 1,
    ChipType = 2,
}

impl TryFrom<u8> for VKDfuInfoType {
    type Error = ViaError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(VKDfuInfoType::End),
            1 => Ok(VKDfuInfoType::ChipName),
            2 => Ok(VKDfuInfoType::ChipType),
            _ => Err(ViaError::Protocol(format!(
                "invalid VKDfuInfoType: {value}"
            ))),
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq, Copy, Clone)]
#[repr(u8)]
pub enum VKDfuChipType {
    #[default]
    Unknown = 0,
    STM32 = 1,
    WB32 = 2,
}

impl TryFrom<u8> for VKDfuChipType {
    type Error = ViaError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(VKDfuChipType::Unknown),
            1 => Ok(VKDfuChipType::STM32),
            2 => Ok(VKDfuChipType::WB32),
            _ => Err(ViaError::Protocol(format!(
                "invalid VKDfuChipType: {value}"
            ))),
        }
    }
}

#[derive(Debug, Default)]
pub struct VKDfuInfo {
    pub name: String,
    pub chip: VKDfuChipType,
}

impl VKDfuInfo {
    pub fn load(proto: &ViaKeychronProtocol) -> ViaResult<Self> {
        proto
            .device
            .raw_hid_send(&VKMiscCommandId::DfuInfoGet.to_cmd())
            .and_then(Self::try_from)
    }
}

fn parse_dfu_str(data: &[u8]) -> ViaResult<String> {
    if data.is_empty() {
        Err(ViaError::Protocol("empty dfu string payload".to_string()))
    } else {
        let len = data[0];
        if data.len() < (len + 1) as usize {
            Err(ViaError::Protocol("dfu string too large".to_string()))
        } else {
            str::from_utf8(&data[1..(len + 1) as usize])
                .map_err(|e| ViaError::Protocol(format!("failed to parse dfu string: {e}")))
                .map(|r| r.to_string())
        }
    }
}

impl TryFrom<ViaReportData> for VKDfuInfo {
    type Error = ViaError;

    fn try_from(value: ViaReportData) -> Result<Self, Self::Error> {
        let value = VKMiscCommandId::DfuInfoGet.check_reply(&value)?;
        let mut ret = VKDfuInfo::default();
        let mut idx = 0;

        while idx < value.len() {
            match VKDfuInfoType::try_from(value[idx])? {
                VKDfuInfoType::ChipName => {
                    ret.name = parse_dfu_str(&value[idx + 1..])?;
                    idx += 2 + ret.name.len();
                }
                VKDfuInfoType::ChipType if idx + 3 < value.len() => {
                    ret.chip = value[idx + 2].try_into()?;
                    idx += 3;
                }
                VKDfuInfoType::End => break,
                _ => break,
            }
        }
        if ret.name.is_empty() || ret.chip == VKDfuChipType::Unknown {
            Err(ViaError::Protocol(format!(
                "invalid DfuInfo packet, invalid packet: {:?}",
                value
            )))
        } else {
            Ok(ret)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::VKCommandId;

    use super::*;

    #[test]
    fn parse() -> ViaResult<()> {
        let mut data = ViaReportData::default();

        let info = VKDfuInfo::try_from(data).expect_err("should fail");
        assert!(
            info.to_string().contains("corrupted cmd"),
            "invalid err: {}",
            info
        );
        data[0] = VKCommandId::MiscCmdGroup as u8;

        let info = VKDfuInfo::try_from(data).expect_err("should fail");
        assert!(
            info.to_string().contains("corrupted sub-cmd"),
            "invalid err: {}",
            info
        );

        data[1] = VKMiscCommandId::DfuInfoGet as u8;
        // both type and name must be set
        VKDfuInfo::try_from(data).expect_err("should fail");

        data[3] = VKDfuInfoType::ChipType as u8;
        data[4] = 1;
        data[5] = 42;
        VKDfuInfo::try_from(data).expect_err("should fail");
        data[5] = VKDfuChipType::STM32 as u8;
        VKDfuInfo::try_from(data).expect_err("should fail");

        data[6] = VKDfuInfoType::ChipName as u8;
        data[7] = 3;
        data[8] = b't';
        data[9] = b's';
        data[10] = b't';
        let info = VKDfuInfo::try_from(data)?;
        assert_eq!(info.name.as_str(), "tst");

        data[11] = 44;
        VKDfuInfo::try_from(data).expect_err("should fail");
        Ok(())
    }
}
