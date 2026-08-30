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

use via_protocol::ViaResult;

use crate::{VKCommandMaker, VKRgbCommandId, VKRgbTrait, ViaKeychronProtocol};

#[derive(Debug, Clone)]
/// gives a representation of the keyboard led layout
pub struct VKRgbLayout {
    pub matrix: Vec<Vec<Option<u8>>>,
}

impl VKRgbLayout {
    pub fn load(proto: &ViaKeychronProtocol) -> ViaResult<Self> {
        let led_count = proto.get_led_count()?;

        let mut matrix: Vec<Vec<Option<u8>>> = Vec::with_capacity(8);
        let cmd = &VKRgbCommandId::RgbGetLedIdx;
        let mut req = cmd.to_cmd();
        /* fills mask + unknown keys at end */
        req.payload_mut()[1..].fill(0xFF);
        let mut led_received = 0;

        for row in 0..255 {
            let payload = req.payload_mut();
            payload[0] = row;

            let resp = proto.raw_send(&req)?;
            match cmd.check_reply(&resp) {
                Ok(payload) => matrix.push(
                    payload
                        .iter()
                        .map(|&v| {
                            if v == 0xFF {
                                None
                            } else {
                                led_received += 1;
                                Some(v)
                            }
                        })
                        .collect(),
                ),
                Err(err) if matrix.is_empty() => return Err(err),
                _ => break,
            }
            if led_received >= led_count {
                break;
            }
        }
        /* reduce */
        let cols = matrix
            .iter()
            .map(|col| col.iter().rposition(|v| v.is_some()).unwrap_or(0))
            .max()
            .unwrap_or(0)
            + 1;
        matrix.iter_mut().for_each(|row| row.resize(cols, None));

        Ok(VKRgbLayout { matrix })
    }

    pub fn get_row_count(&self) -> usize {
        self.matrix.len()
    }

    pub fn get_col_count(&self) -> usize {
        self.matrix.first().map_or(0, |col| col.len())
    }
}
