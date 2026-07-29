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
    VKAnalogKeyConfigMode, VKAnalogProfile, VKAnalogProfileInfo, ViaKeychronProtocol,
};

use crate::common::*;

#[test]
#[serial(keyboard)]
fn load_info() -> ViaResult<()> {
    let kbd = get_keyboard(&HID)?;
    let proto = ViaKeychronProtocol::new(&kbd);

    let info = VKAnalogProfileInfo::load(&proto)?;
    tracing::info!(%info);
    Ok(())
}

#[test]
#[serial(keyboard)]
fn select() -> ViaResult<()> {
    let kbd = get_keyboard(&HID)?;
    let proto = ViaKeychronProtocol::new(&kbd);
    let info = VKAnalogProfileInfo::load(&proto)?;

    for profile in 0..info.get_profile_count() {
        VKAnalogProfile::select(&proto, profile)?;
        let info = VKAnalogProfileInfo::load(&proto)?;
        assert_eq!(info.get_current_profile(), profile, "failed to set profile");
    }

    VKAnalogProfile::select(&proto, info.get_current_profile())?;
    assert_eq!(
        info.get_current_profile(),
        VKAnalogProfileInfo::load(&proto)?.get_current_profile()
    );
    Ok(())
}

#[test]
#[serial(keyboard)]
fn load_profile() -> ViaResult<()> {
    let kbd = get_keyboard(&HID)?;
    let proto = ViaKeychronProtocol::new(&kbd);
    let info = VKAnalogProfileInfo::load(&proto)?;

    let profile = VKAnalogProfile::load(&proto, 0)?;
    assert_eq!(
        profile.data.len(),
        info.get_raw_profile_byte_size() as usize
    );

    let key_config = profile.get_global_key_config()?;
    tracing::info!("key_config={key_config} raw={:?}", key_config.data);
    let mode = key_config.get_mode()?;

    let key_config = profile.get_key_config(0)?;
    tracing::info!("key_config={key_config} raw={:?}", key_config.data);
    for i in 0..10 {
        let key_config = profile.get_key_config(i)?;
        tracing::info!(i, "key_config={key_config} raw={:?}", key_config.data);
        // tracing::info!(i, "adv_modekey_config={key_config} raw={:?}", key_config.data);
    }
    assert!(
        [VKAnalogKeyConfigMode::Rapid, VKAnalogKeyConfigMode::Regular].contains(&mode),
        "invalid global mode: {mode:?}"
    );
    Ok(())
}
