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
    VKAnalogGamepadData, VKAnalogKeyConfig, VKAnalogKeyConfigAdvData, VKAnalogKeyConfigMode,
    VKAnalogProfile, VKAnalogTrait, ViaKeychronProtocol,
};

use super::profile::is_analog;
use crate::common::*;

#[test]
#[serial(keyboard)]
fn okmc_mode() -> ViaResult<()> {
    let kbd = get_keyboard(&HID)?;
    let proto = ViaKeychronProtocol::new(&kbd);
    if !is_analog(&proto)? {
        return Ok(());
    }

    let profile = proto.get_analog_profile(0)?;

    let mut key_config = profile.get_key_config(0)?;
    let backup = key_config.clone();
    key_config.set_mode(VKAnalogKeyConfigMode::DKS);
    key_config.set_adv_mode_info(0);
    /* use data converters */
    key_config.set_adv_mode_info(VKAnalogKeyConfigAdvData::Okmc { index: 0 });

    let okmc = profile.get_okmc_config(1)?;
    key_config
        .send(&proto, Some(&okmc))
        .expect_err("invalid okmc provided");
    key_config.send(&proto, None).expect_err("no okmc provided");

    let okmc = profile.get_okmc_config(0)?;
    key_config.send(&proto, Some(&okmc))?;

    let profile = proto.get_analog_profile(0)?;
    assert_eq!(key_config, profile.get_key_config(0)?);

    restore_key_config(&backup, &profile, &proto)?;
    let profile = proto.get_analog_profile(0)?;
    assert_eq!(backup, profile.get_key_config(0)?);
    Ok(())
}

#[test]
#[serial(keyboard)]
fn mode_switch() -> ViaResult<()> {
    let kbd = get_keyboard(&HID)?;
    let proto = ViaKeychronProtocol::new(&kbd);
    if !is_analog(&proto)? {
        return Ok(());
    }

    let profile = proto.get_analog_profile(0)?;

    let mut key_config = profile.get_key_config(0)?;
    let backup = key_config.clone();

    key_config.set_mode(VKAnalogKeyConfigMode::Gamepad);
    key_config.set_adv_mode_info(VKAnalogGamepadData::ButtonDown);
    key_config.send(&proto, None)?;
    let profile = proto.get_analog_profile(0)?;
    assert_eq!(key_config, profile.get_key_config(0)?);

    for mode in [
        VKAnalogKeyConfigMode::Toggle,
        VKAnalogKeyConfigMode::Global,
        VKAnalogKeyConfigMode::Rapid,
        VKAnalogKeyConfigMode::Regular,
    ] {
        let mut key_config = profile.get_key_config(0)?;
        key_config.set_mode(mode);
        key_config.set_adv_mode_info(0);
        key_config.send(&proto, None)?;
        let profile = proto.get_analog_profile(0)?;
        assert_eq!(key_config, profile.get_key_config(0)?);
    }

    restore_key_config(&backup, &profile, &proto)?;
    let profile = proto.get_analog_profile(0)?;
    assert_eq!(backup, profile.get_key_config(0)?);
    Ok(())
}

fn restore_key_config(
    backup: &VKAnalogKeyConfig,
    profile: &VKAnalogProfile,
    proto: &ViaKeychronProtocol,
) -> ViaResult<()> {
    let okmc = match backup.get_okmc_index() {
        Some(idx) => Some(profile.get_okmc_config(idx)?),
        None => None,
    };
    backup.send(proto, okmc.as_ref())?;

    Ok(())
}
