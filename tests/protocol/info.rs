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

use via_protocol_keychron::{VKFeatures, VKProtocolVersion, ViaKeychronProtocol, ViaResult};

#[test]
#[serial(keyboard)]
fn connect() -> ViaResult<()> {
    get_keyboard(&HID).and(Ok(()))
}

#[test]
#[serial(keyboard)]
fn info() -> ViaResult<()> {
    let kbd = get_keyboard(&HID)?;
    let proto = ViaKeychronProtocol::new(&kbd);

    let ret = VKProtocolVersion::load(&proto)?;
    tracing::info!(protocol_version = ?ret);

    let ret = proto.get_firmware_version()?;
    tracing::info!(firmware_version = ?ret);

    let support_features = VKFeatures::load(&proto)?;
    tracing::info!(?support_features);

    let ret = proto.get_default_layer()?;
    tracing::info!(default_layer = ?ret);
    Ok(())
}
