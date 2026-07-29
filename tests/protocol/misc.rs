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

use serial_test::serial;

use crate::common::*;

use via_protocol_keychron::{VKMiscFeatures, VKMiscTrait, ViaKeychronProtocol, ViaResult};

#[test]
#[serial(keyboard)]
fn misc() -> ViaResult<()> {
    let kbd = get_keyboard(&HID)?;
    let proto = ViaKeychronProtocol::new(&kbd);

    let ret = proto.get_misc_info()?;
    tracing::info!(misc_protocol_version = ?ret);
    let features = ret.features;

    if features.contains(VKMiscFeatures::DFU_INFO) {
        let dfu_info = proto.get_dfu_info()?;
        tracing::info!(?dfu_info);
    } else {
        proto.get_dfu_info().expect_err("should fail");
    }

    if features.contains(VKMiscFeatures::LANGUAGE) {
        let ret = proto.get_language()?;
        tracing::info!(language = ?ret);
        proto.set_language(&ret)?;
    } else {
        proto.get_language().expect_err("should fail");
    }

    if features.contains(VKMiscFeatures::DEBOUNCE) {
        let debounce = proto.get_debounce()?;
        tracing::info!(%debounce);
        tracing::trace!(?debounce);
        proto.set_debounce(&debounce)?;
    } else {
        proto.get_debounce().expect_err("should fail");
    }

    if features.contains(VKMiscFeatures::SNAP_CLICK) {
        let snap_click_count = proto.get_snap_click_count()?;
        tracing::info!(snap_click_count);
        assert!(snap_click_count >= 9);
        let snaps = proto.get_snap_click()?;
        assert_eq!(snaps.config.len(), snap_click_count as usize);
        tracing::info!(%snaps);
        tracing::trace!(?snaps);
        proto.set_snap_click(&snaps)?;
        proto.save_snap_click(&snaps)?;
    } else {
        proto.get_snap_click_count().expect_err("should fail");
    }

    if features.contains(VKMiscFeatures::WIRELESS_LPM) {
        let wireless_lpm = proto.get_wireless_lpm()?;
        tracing::info!(%wireless_lpm);
        tracing::trace!(?wireless_lpm);
        proto.set_wireless_lpm(&wireless_lpm)?;
    } else {
        proto.get_wireless_lpm().expect_err("should fail");
    }

    if features.contains(VKMiscFeatures::REPORT_RATE) {
        let report_rate = proto.get_report_rate()?;
        tracing::info!(%report_rate);
        tracing::trace!(?report_rate);
        proto.set_report_rate(&report_rate)?;
    } else {
        proto.get_report_rate().expect_err("should fail");
    }

    if features.contains(VKMiscFeatures::NKRO) {
        let nkro = proto.get_nkro()?;
        tracing::info!(%nkro);
        tracing::trace!(?nkro);
        if nkro.is_available() {
            proto.set_nkro(&nkro)?;
        }
    } else {
        proto.get_nkro().expect_err("should fail");
    }
    Ok(())
}
