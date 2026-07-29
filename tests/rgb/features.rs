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

use palette::{IntoColor, named};
use serial_test::serial;
use via_protocol::{ViaProtocol, ViaResult};

use crate::common::*;

use via_protocol_keychron::{VKFeatures, VKRgbTrait, ViaKeychronProtocol};

#[test]
#[serial(keyboard)]
fn rgb() -> ViaResult<()> {
    let kbd = get_keyboard(&HID)?;
    let proto = ViaKeychronProtocol::new(&kbd);

    let support_features = proto.get_support_features()?;
    tracing::info!(?support_features);

    if support_features.contains(VKFeatures::KEYCHRON_RGB) {
        let rgb_info = proto.get_rgb_info()?;
        tracing::info!(?rgb_info);

        proto.save_rgb()?;
    } else {
        proto.get_rgb_info().expect_err("should fail");
        return Ok(());
    }

    let mut indicators = proto.get_indicators()?;
    tracing::info!(%indicators);
    let initial = indicators.get_color();
    indicators.set_color(&named::RED.into_format::<f32>().into_color());
    proto.set_indicators(&indicators)?;
    indicators.set_color(&initial);
    proto.set_indicators(&indicators)?;

    let led_count = proto.get_led_count()?;
    tracing::info!(led_count);
    Ok(())
}

#[test]
#[ignore = "via test"]
#[serial(keyboard)]
fn lighting_protocol() -> ViaResult<()> {
    let kbd = get_keyboard(&HID)?;

    let via = ViaProtocol::new(&kbd);
    let proto = via
        .detect_lighting_protocol()
        .expect("keychron keyboards must support lighting-protocol");
    tracing::info!(?proto);

    let mut light = via.read_lighting_values(&proto)?;
    tracing::info!(?light);
    light.effect_id = 24;
    via.write_lighting_values(&proto, &light)?;
    Ok(())
}
