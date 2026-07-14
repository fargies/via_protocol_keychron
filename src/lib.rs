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

mod protocol;
pub use protocol::{KEYCHRON_VENDOR_ID, ViaKeychronProtocol, discover_keyboards};

mod command;
pub use command::{VKCommand, VKCommandId, VKCommandMaker};

mod version;
pub use version::VKProtocolVersion;

mod features;
pub use features::*;

mod misc;
pub use misc::*;

mod rgb;
pub use rgb::*;

mod data;
pub use data::{VKMiscFeatures, ViaReportData};

pub use via_protocol::{KeyboardDevice, ViaError, ViaResult};

#[cfg(test)]
mod tests {
    #[ctor::ctor(unsafe)]
    fn setup() {
        use tracing_subscriber::{
            EnvFilter, Registry, fmt, layer::SubscriberExt, util::SubscriberInitExt,
        };
        Registry::default()
            .with(EnvFilter::from_default_env())
            .with(fmt::layer().with_test_writer())
            .init();
    }
}
