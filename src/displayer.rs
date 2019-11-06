// Copyright 2019 DFINITY
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::fmt;
use std::marker::PhantomData;

/// This trait provides display capabilities for the type it
/// parameterized with, `T`.
///
/// NOTE: If you ended up reading this doc, consider using regular
/// [newtype
/// idiom](https://doc.rust-lang.org/rust-by-example/generics/new_types.html)
/// instead of relying on `phantom_newtype`.
pub trait DisplayerOf<T> {
    fn display(value: &T, f: &mut fmt::Formatter<'_>) -> fmt::Result;
}

/// An object `DisplayProxy`, when is asked to display itself,
/// displays `T` using the specified `Displayer` instead.
pub struct DisplayProxy<'a, T, Displayer>
where
    Displayer: DisplayerOf<T>,
{
    value: &'a T,
    displayer_tag: PhantomData<Displayer>,
}

impl<'a, T, Displayer> DisplayProxy<'a, T, Displayer>
where
    Displayer: DisplayerOf<T>,
{
    pub fn new(value: &'a T) -> Self {
        Self {
            value,
            displayer_tag: PhantomData,
        }
    }
}

impl<'a, T, Displayer> fmt::Display for DisplayProxy<'a, T, Displayer>
where
    Displayer: DisplayerOf<T>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Displayer::display(&self.value, f)
    }
}
