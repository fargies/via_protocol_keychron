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

use crate::{
    VKCommand, VKCommandId, VKCommandMaker, ViaError, ViaKeychronProtocol, ViaReportData, ViaResult,
};

mod info;
pub use info::*;

mod nkro;
pub use nkro::VKNkroConfig;

mod dfu;
pub use dfu::{VKDfuChipType, VKDfuInfo, VKDfuInfoType};

mod debounce;
pub use debounce::{VKDebounceConfig, VKDebounceType};

mod snap_click;
pub use snap_click::{VKSnapClick, VKSnapClickConfig, VKSnapClickType};

mod wireless_lpm;
pub use wireless_lpm::VKWirelessLpmConfig;

mod report_rate;
pub use report_rate::VKReportRateConfig;

mod language;
pub use language::VKLanguageLayout;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum VKMiscCommandId {
    MiscGetProtocolVer = 0x01,
    DfuInfoGet = 0x02,
    LanguageGet = 0x03,
    LanguageSet = 0x04,
    DebounceGet = 0x05,
    DebounceSet = 0x06,
    SnapClickGetInfo = 0x07,
    SnapClickGet = 0x08,
    SnapClickSet = 0x09,
    SnapClickSave = 0x0A,
    WirelessLpmGet = 0x0B,
    WirelessLpmSet = 0x0C,
    ReportRateGet = 0x0D,
    ReportRateSet = 0x0E,
    DipSwitchGet = 0x0F,
    DipSwitchSet = 0x10,
    FactoryReset = 0x11,
    NkroGet = 0x12,
    NkroSet = 0x13,
}

impl VKMiscCommandId {
    pub const HEADER_BYTE_SIZE: usize = 3;

    /// Convenience function, checks reply returns payload if properly built
    #[tracing::instrument(level = "ERROR", err)]
    pub fn check_reply<'a>(&self, value: &'a ViaReportData) -> ViaResult<&'a [u8]> {
        if value[0] != VKCommandId::MiscCmdGroup as u8 {
            Err(ViaError::Protocol(format!(
                "invalid {self:?} packet, corrupted cmd: {}",
                value[0]
            )))
        } else if value[1] != *self as u8 {
            Err(ViaError::Protocol(format!(
                "invalid {self:?} packet, corrupted sub-cmd: {}",
                value[1]
            )))
        } else if value[2] != 0 {
            Err(ViaError::Protocol(format!(
                "invalid {self:?} packet, ret = {}",
                value[2]
            )))
        } else {
            Ok(&value[Self::HEADER_BYTE_SIZE..])
        }
    }
}

impl VKCommandMaker for VKMiscCommandId {
    /// @brief generate [VKCommand] from a [VKMiscCommandId]
    fn to_cmd(self) -> VKCommand {
        let mut report = [0u8; via_protocol::VIA_REPORT_SIZE + 1];
        report[0] = 0x00;
        report[1] = VKCommandId::MiscCmdGroup as u8;
        report[2] = self as u8;
        VKCommand { report, payload_offset: 3 }
    }
}

pub trait VKMiscTrait {
    /// @brief get the misc info
    fn get_misc_info(&self) -> ViaResult<Arc<VKMiscInfo>>;

    /// @brief get the debounce config
    fn get_debounce(&self) -> ViaResult<VKDebounceConfig>;

    /// @brief set the debounce config
    fn set_debounce(&self, debounce: &VKDebounceConfig) -> ViaResult<()>;

    /// @brief get the dfu info
    fn get_dfu_info(&self) -> ViaResult<VKDfuInfo>;

    /// @brief get the language layout
    fn get_language(&self) -> ViaResult<VKLanguageLayout>;

    /// @brief set the language layout
    fn set_language(&self, layout: &VKLanguageLayout) -> ViaResult<()>;

    /// @brief get the nkro config
    fn get_nkro(&self) -> ViaResult<VKNkroConfig>;

    /// @brief set the nkro config
    fn set_nkro(&self, value: &VKNkroConfig) -> ViaResult<()>;

    /// @brief get the report rate config
    fn get_report_rate(&self) -> ViaResult<VKReportRateConfig>;

    /// @brief set the report rate config
    fn set_report_rate(&self, config: &VKReportRateConfig) -> ViaResult<()>;

    /// @brief get the snap click count
    fn get_snap_click_count(&self) -> ViaResult<u8>;

    /// @brief get the snap click config
    fn get_snap_click(&self) -> ViaResult<VKSnapClickConfig>;

    /// @brief set the snap click config
    fn set_snap_click(&self, config: &VKSnapClickConfig) -> ViaResult<()>;

    /// @brief save the snap click config
    fn save_snap_click(&self, config: &VKSnapClickConfig) -> ViaResult<()>;

    /// @brief get the wireless lpm config
    fn get_wireless_lpm(&self) -> ViaResult<VKWirelessLpmConfig>;

    /// @brief set the wireless lpm config
    fn set_wireless_lpm(&self, config: &VKWirelessLpmConfig) -> ViaResult<()>;
}

impl VKMiscTrait for ViaKeychronProtocol<'_> {
    fn get_misc_info(&self) -> ViaResult<Arc<VKMiscInfo>> {
        VKMiscInfo::load(self)
    }

    fn get_debounce(&self) -> ViaResult<VKDebounceConfig> {
        VKDebounceConfig::load(self)
    }

    fn set_debounce(&self, debounce: &VKDebounceConfig) -> ViaResult<()> {
        debounce.send(self)
    }

    fn get_dfu_info(&self) -> ViaResult<VKDfuInfo> {
        VKDfuInfo::load(self)
    }

    fn get_language(&self) -> ViaResult<VKLanguageLayout> {
        VKLanguageLayout::load(self)
    }

    fn set_language(&self, layout: &VKLanguageLayout) -> ViaResult<()> {
        layout.save(self)
    }

    fn get_nkro(&self) -> ViaResult<VKNkroConfig> {
        VKNkroConfig::load(self)
    }

    fn set_nkro(&self, value: &VKNkroConfig) -> ViaResult<()> {
        value.send(self)
    }

    fn get_report_rate(&self) -> ViaResult<VKReportRateConfig> {
        VKReportRateConfig::load(self)
    }

    fn set_report_rate(&self, config: &VKReportRateConfig) -> ViaResult<()> {
        config.send(self)
    }

    fn get_snap_click_count(&self) -> ViaResult<u8> {
        VKSnapClickConfig::count(self)
    }

    fn get_snap_click(&self) -> ViaResult<VKSnapClickConfig> {
        VKSnapClickConfig::load(self)
    }

    fn set_snap_click(&self, config: &VKSnapClickConfig) -> ViaResult<()> {
        config.send(self)
    }

    fn save_snap_click(&self, config: &VKSnapClickConfig) -> ViaResult<()> {
        config.save(self)
    }

    fn get_wireless_lpm(&self) -> ViaResult<VKWirelessLpmConfig> {
        VKWirelessLpmConfig::load(self)
    }

    fn set_wireless_lpm(&self, config: &VKWirelessLpmConfig) -> ViaResult<()> {
        config.send(self)
    }
}
