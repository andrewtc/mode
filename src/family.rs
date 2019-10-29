// Copyright 2019 Andrew Thomas Christensen
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the
// MIT license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option. This file may not be copied,
// modified, or distributed except according to those terms.

use crate::Mode;

pub trait Family {
    /// Represents the user-facing interface for the `Mode` that will be exposed via the `Automaton`. In order to be
    /// used with an `Automaton`, the `Base` type of the `Mode` **must** match the `Base` type of the `Automaton`. This
    /// is so that the `Automaton` can provide `borrow_mode()` and `borrow_mode_mut()` functions that return a reference
    /// to the `Mode` as the `Base` type.
    /// 
    type Base : ?Sized;

    type Mode : Mode<Family = Self>;

    type Input;

    type Output;
}