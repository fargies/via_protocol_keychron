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

mod debug;

pub use debug::DebugIter;

pub(crate) enum DebugAsDisplay<'a, T> {
    Borrowed(&'a T),
    Owned(T)
}

impl<'a, T> From<T> for DebugAsDisplay<'a, T> {
    fn from(value: T) -> Self {
        DebugAsDisplay::Owned(value)
    }
}

impl<'a, T> From<&'a T> for DebugAsDisplay<'a, T> {
    fn from(value: &'a T) -> Self {
        DebugAsDisplay::Borrowed(value)
    }
}

impl<'a, T> std::fmt::Debug for DebugAsDisplay<'a, T>
where
    T: std::fmt::Display,
{

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DebugAsDisplay::Borrowed(value) => value.fmt(f),
            DebugAsDisplay::Owned(value) => value.fmt(f),
        }
    }
}
