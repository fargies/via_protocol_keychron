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

use std::fmt::Display;

use via_protocol::{ViaError, ViaResult};

use crate::{VKCommandMaker, VKMiscCommandId, ViaKeychronProtocol, ViaReportData};

#[derive(Debug)]
pub struct VKReportRateConfig {
    data: Vec<u8>,
}

impl VKReportRateConfig {
    pub fn load(proto: &ViaKeychronProtocol) -> ViaResult<Self> {
        proto
            .raw_send(&VKMiscCommandId::ReportRateGet.to_cmd())
            .and_then(Self::try_from)
    }

    pub fn send(&self, proto: &ViaKeychronProtocol) -> ViaResult<()> {
        let cmd = &VKMiscCommandId::ReportRateSet;

        let resp = proto.raw_send(&cmd.to_req(self.data.as_ref()))?;
        cmd.check_reply(&resp)?;
        Ok(())
    }

    pub fn get_div(&self) -> u8 {
        self.data[0]
    }

    pub fn set_div(&mut self, value: u8) {
        self.data[0] = value;
    }
}

impl Display for VKReportRateConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VKReportRateConfig")
            .field("div", &self.get_div())
            .finish()
    }
}

impl TryFrom<ViaReportData> for VKReportRateConfig {
    type Error = ViaError;

    fn try_from(value: ViaReportData) -> Result<Self, Self::Error> {
        let payload = VKMiscCommandId::ReportRateGet.check_reply(&value)?;
        Ok(VKReportRateConfig {
            data: payload.into(),
        })
    }
}
