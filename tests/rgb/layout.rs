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
use via_protocol_keychron::{VKFeatures, VKRgbTrait, ViaKeychronProtocol};

use crate::common::*;

#[test]
#[serial(keyboard)]
fn layout() -> ViaResult<()> {
    let kbd = get_keyboard(&HID)?;
    let proto = ViaKeychronProtocol::new(&kbd);

    let support_features = proto.get_support_features()?;
    tracing::info!(?support_features);

    if !support_features.contains(VKFeatures::KEYCHRON_RGB) {
        tracing::warn!("not running Layout test: not supported by keyboard");
        return Ok(());
    }

    let rgb_layout = proto.get_rgb_layout()?;
    assert!(!rgb_layout.matrix.is_empty());
    assert!(rgb_layout.get_row_count() > 0);
    assert!(rgb_layout.get_col_count() > 0);

    for row in rgb_layout.matrix.iter() {
        for col in row.iter() {
            match col {
                Some(idx) => print!("{:^3} ", idx),
                None => print!(" -  "),
            }
        }
        println!();
    }

    Ok(())
}
