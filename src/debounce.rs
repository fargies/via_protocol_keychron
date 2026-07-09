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

use via_protocol::ViaError;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[repr(u8)]
pub enum VKDebounceType {
    SymDeferGlobal = 0,
    SymDeferPerRow = 1,
    SymDeferPerKey = 2,
    SymEagerPerRow = 3,
    SymEagerPerKey = 4,
    AsymEagerDeferPerKey = 5,
    None = 6,
    Max = 7,
}

impl TryFrom<u8> for VKDebounceType {
    type Error = ViaError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value <= VKDebounceType::Max as u8 {
            Ok(unsafe { std::mem::transmute::<u8, VKDebounceType>(value) })
        } else {
            Err(ViaError::Protocol(format!(
                "invalid VKDebounceType: {value}"
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use via_protocol::ViaResult;

    use super::*;

    #[test]
    fn parse() -> ViaResult<()> {
        assert_eq!(VKDebounceType::try_from(2)?, VKDebounceType::SymDeferPerKey);
        VKDebounceType::try_from(42).expect_err("should fail to convert");
        Ok(())
    }
}
