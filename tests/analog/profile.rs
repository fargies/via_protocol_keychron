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
use via_protocol::ViaResult;
use via_protocol_keychron::{
    VKAnalogKeyConfigMode, VKAnalogTrait, ViaKeychronProtocol,
};

use crate::common::*;

#[test]
#[serial(keyboard)]
fn load_info() -> ViaResult<()> {
    let kbd = get_keyboard(&HID)?;
    let proto = ViaKeychronProtocol::new(&kbd);

    let info = proto.get_analog_info()?;
    tracing::info!(profile_info = %info.profile_info, protocol_version = ?info.protocol_version);
    Ok(())
}

#[test]
#[serial(keyboard)]
fn select() -> ViaResult<()> {
    let kbd = get_keyboard(&HID)?;
    let proto = ViaKeychronProtocol::new(&kbd);
    let info = proto.get_analog_info()?.profile_info.clone();
    let current_profile = info.get_current_profile();

    for profile in 0..info.get_profile_count() {
        proto.select_analog_profile(profile)?;
        let info = proto.get_analog_info()?.profile_info.clone();
        assert_eq!(info.get_current_profile(), profile, "failed to set profile");
    }

    // this is local and should not have been modified
    assert_eq!(current_profile, info.get_current_profile());
    proto.select_analog_profile(info.get_current_profile())?;
    assert_eq!(
        info.get_current_profile(),
        proto.get_analog_info()?.profile_info.get_current_profile()
    );
    Ok(())
}

#[test]
#[serial(keyboard)]
fn load_profile() -> ViaResult<()> {
    let kbd = get_keyboard(&HID)?;
    let proto = ViaKeychronProtocol::new(&kbd);
    let info = proto.get_analog_info()?.profile_info.clone();

    let profile = proto.get_analog_profile(0)?;
    assert_eq!(
        profile.data.len(),
        info.get_raw_profile_byte_size() as usize
    );

    let key_config = profile.get_global_key_config()?;
    tracing::info!("key_config={key_config} raw={:?}", key_config.data);
    let mode = key_config.get_mode()?;

    let key_config = profile.get_key_config(0)?;
    tracing::info!("key_config={key_config} raw={:?}", key_config.data);
    for (index, key_config) in profile.iter_key_config().enumerate() {
        if !key_config.is_empty() {
            tracing::info!(index, "key_config={key_config} raw={:?}", key_config.data);
        }
    }
    assert!(
        [VKAnalogKeyConfigMode::Rapid, VKAnalogKeyConfigMode::Regular].contains(&mode),
        "invalid global mode: {mode:?}"
    );
    assert_eq!(profile.key_count, profile.iter_key_config().count());

    for (index, okmc_config) in profile.iter_okmc_config().enumerate() {
        if !okmc_config.is_empty() {
            tracing::info!(
                index,
                "okmc_config={okmc_config} raw={:?}",
                okmc_config.data
            );
        }
    }
    if profile.iter_okmc_config().all(|c| c.is_empty()) {
        tracing::info!("no okmc configured");
    }

    for (index, socd_config) in profile.iter_socd_config().enumerate() {
        if !socd_config.is_empty() {
            tracing::info!(
                index,
                "socd_config={socd_config} raw={:?}",
                socd_config.data
            );
        }
    }
    if profile.iter_socd_config().all(|c| c.is_empty()) {
        tracing::info!("no socd configured");
    }

    Ok(())
}

#[test]
#[serial(keyboard)]
fn profile_name() -> ViaResult<()> {
    let kbd = get_keyboard(&HID)?;
    let proto = ViaKeychronProtocol::new(&kbd);
    let info = proto.get_analog_info()?.profile_info.clone();

    let mut profile = proto.get_analog_profile(0)?;
    assert_eq!(
        profile.data.len(),
        info.get_raw_profile_byte_size() as usize
    );

    let name = profile.get_name()?;
    tracing::info!("profile name: {name}");
    profile.set_name("pwet")?;
    assert_eq!(profile.get_name()?.as_str(), "pwet");
    profile.send_name(&proto)?;

    let mut profile = proto.get_analog_profile(0)?;
    assert_eq!(profile.get_name()?.as_str(), "pwet");
    profile.set_name(name)?;
    profile.send_name(&proto)?;

    Ok(())
}
