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
    VKFeatures, VKHsv, VKRgbPerKeyConfig, VKRgbPerKeyType, VKRgbTrait, ViaKeychronProtocol,
};

use crate::common::*;

#[test]
#[serial(keyboard)]
fn config() -> ViaResult<()> {
    let kbd = get_keyboard(&HID)?;
    let proto = ViaKeychronProtocol::new(&kbd);

    let support_features = proto.get_support_features()?;
    tracing::info!(?support_features);

    if !support_features.contains(VKFeatures::KEYCHRON_RGB) {
        tracing::warn!("not running PerKey test: not supported by keyboard");
        return Ok(());
    }

    let led_count = proto.get_led_count()?;
    tracing::info!(led_count);

    let per_key_type = VKRgbPerKeyType::load(&proto)?;
    tracing::info!(?per_key_type);
    VKRgbPerKeyType::Breathing.send(&proto)?;
    per_key_type.send(&proto)?;

    let config = VKRgbPerKeyConfig::load(&proto)?;
    assert_eq!(config.config.len(), led_count as usize);
    assert_eq!(0, config.start);

    let mut new_config = VKRgbPerKeyConfig::load_part(&proto, 0, 1)?;
    assert_eq!(new_config.config.len(), 1);
    assert_eq!(0, new_config.start);
    new_config.config[0] = VKHsv {
        hue: 255,
        saturation: 100,
        value: 200,
    };
    new_config.send(&proto)?;
    config.send(&proto)?;

    Ok(())
}
