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
use via_protocol_keychron::{VKAnalogTrait, VKSocdKey, VKSocdType, ViaKeychronProtocol};

use super::profile::is_analog;
use crate::common::*;

#[test]
#[serial(keyboard)]
fn socd_update() -> ViaResult<()> {
    let kbd = get_keyboard(&HID)?;
    let proto = ViaKeychronProtocol::new(&kbd);
    if !is_analog(&proto)? {
        return Ok(());
    }

    let profile = proto.get_analog_profile(0)?;
    assert!(profile.socd_count > 0);

    let mut socd = profile.get_socd_config(0)?;
    let backup = socd.clone();
    socd.set_key(VKSocdKey::Key1, 0);
    socd.set_key(VKSocdKey::Key2, 3);
    socd.set_type(VKSocdType::DeeperTravelSingle);
    socd.send(&proto)?;
    let profile = proto.get_analog_profile(0)?;
    assert_eq!(socd, profile.get_socd_config(0)?);

    backup.send(&proto)?;
    let profile = proto.get_analog_profile(0)?;
    assert_eq!(backup, profile.get_socd_config(0)?);
    Ok(())
}
