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
use via_protocol_keychron::{
    VKFeatures, VKRgbMixedEffectList, VKRgbMixedRegions, VKRgbTrait, ViaKeychronProtocol, ViaResult,
};

use crate::common::*;

#[test]
#[serial(keyboard)]
fn regions() -> ViaResult<()> {
    let kbd = get_keyboard(&HID)?;
    let proto = ViaKeychronProtocol::new(&kbd);

    let support_features = proto.get_support_features()?;
    tracing::info!(?support_features);

    if !support_features.contains(VKFeatures::KEYCHRON_RGB) {
        tracing::warn!("not running PerKey test: not supported by keyboard");
        return Ok(());
    }

    let mixed_info = proto.get_rgb_info()?.mixed.clone();
    tracing::info!(%mixed_info);

    let led_count = proto.get_led_count()?;
    tracing::info!(led_count);

    // backup all regions
    let regions = proto.get_mixed_regions()?;
    assert_eq!(0, regions.start);
    assert_eq!(led_count, regions.regions.len());
    tracing::info!(?regions);

    // set half of the keyboard to region 1
    let mut req = VKRgbMixedRegions {
        start: 0,
        regions: vec![1; led_count / 2],
    };
    proto.set_mixed_regions(&req)?;

    // load new regions map
    let new_regions = proto.get_mixed_regions()?;
    tracing::info!(?new_regions);

    // restore changes
    req.regions
        .copy_from_slice(&regions.regions[0..led_count / 2]);
    proto.set_mixed_regions(&req)?;

    // compare new config with backup for equality
    let new_regions = proto.get_mixed_regions()?;
    assert_eq!(new_regions, regions);

    Ok(())
}

#[test]
#[serial(keyboard)]
fn effects() -> ViaResult<()> {
    let kbd = get_keyboard(&HID)?;
    let proto = ViaKeychronProtocol::new(&kbd);

    let support_features = proto.get_support_features()?;
    tracing::info!(?support_features);

    if !support_features.contains(VKFeatures::KEYCHRON_RGB) {
        tracing::warn!("not running PerKey test: not supported by keyboard");
        return Ok(());
    }

    let info = proto.get_mixed_info()?;

    for region in 0..info.get_region_count() {
        let effects = proto.get_mixed_effects(region)?;
        tracing::info!(?effects);
        assert_eq!(effects.region, region);
        assert_eq!(effects.start, 0);
        assert_eq!(
            effects.effects.len(),
            info.get_effects_per_region() as usize
        );

        let single = VKRgbMixedEffectList::load_part(&proto, region, 1, 1)?;
        assert_eq!(single.region, region);
        assert_eq!(single.start, 1);
        assert_eq!(1, single.effects.len());
        assert_eq!(effects.effects[1], single.effects[0]);
    }

    let effects = VKRgbMixedEffectList::load(&proto, 0)?;
    let mut new_effects = effects.clone();
    let mut hue = 0;
    for e in new_effects.effects.iter_mut() {
        e.effect = 2;
        e.time = 2000;
        e.speed = 100;
        e.saturation = 255;
        e.hue = hue;
        hue = hue.overflowing_add(30).0;
    }
    new_effects.send(&proto)?;
    // restore
    effects.send(&proto)?;
    Ok(())
}
