// Copyright 2019 Andrew Thomas Christensen
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the
// MIT license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option. This file may not be copied,
// modified, or distributed except according to those terms.

use crate::Mode;

use std::marker::PhantomData;

/// Defines the `Automaton`-facing interface for a `ModeWrapper`.
/// 
pub trait AnyModeWrapper<'a> {
    /// Represents the user-facing interface for the `Mode` that will be exposed via the `Automaton`. See `Mode::Base`
    /// for more details.
    /// 
    type Base : ?Sized;

    /// Returns an immutable reference to the inner `Mode` as a `&Self::Base`.
    /// 
    fn borrow_mode(&self) -> &Self::Base;

    /// Returns a mutable reference to the inner `Mode` as a `&mut Self::Base`.
    /// 
    fn borrow_mode_mut(&mut self) -> &mut Self::Base;

    /// Calls `get_transition()` on the inner `Mode` to determine whether it wants another `Mode` to become active. If
    /// this yields a `Transition`, the `Transition` will be called on the inner `Mode` and a new `ModeWrapper` around
    /// the `Mode` to be swapped in will be returned.
    /// 
    fn perform_transitions(&mut self) -> Option<Box<dyn AnyModeWrapper<'a, Base = Self::Base> + 'a>>;
}

/// Wraps a specific instance of `Mode`, allowing the parent `Automaton` to handle `Transition`s between that instance
/// and other `Mode`s gracefully.
/// 
/// **NOTE:** This `struct` mainly exists to allow `Transition`s to be scheduled as `FnOnce(A) -> B` instead of
/// requiring each user-defined `Mode` to know more about the implementation details of the `Automaton`.
/// 
pub(crate) struct ModeWrapper<'a, T>
    where T : Mode<'a>,
{
    phantom : PhantomData<&'a T>,
    mode : Option<T>,
}

impl<'a, T> ModeWrapper<'a, T>
    where T : Mode<'a>,
{
    /// Creates and returns a new `ModeWrapper` around the specified `Mode`.
    /// 
    pub fn new(mode : T) -> Self {
        Self {
            phantom: PhantomData,
            mode: Some(mode),
        }
    }
}

impl<'a, T> AnyModeWrapper<'a> for ModeWrapper<'a, T>
    where T : Mode<'a>,
{
    type Base = T::Base;

    fn borrow_mode(&self) -> &Self::Base {
        self.mode.as_ref().unwrap().as_base()
    }

    fn borrow_mode_mut(&mut self) -> &mut Self::Base {
        self.mode.as_mut().unwrap().as_base_mut()
    }

    fn perform_transitions(&mut self) -> Option<Box<dyn AnyModeWrapper<'a, Base = Self::Base> + 'a>> {
        // Retrieve the desired transition, if any, from the inner Mode.
        match self.mode.as_mut().unwrap().get_transition() {
            None => None,
            Some(transition) => {
                // If a valid Transition was returned, call the Transition callback on the inner Mode and return a new
                // wrapper around the Mode that was produced.
                // NOTE: This will move the Mode into the callback, leaving this object empty.
                Some(transition.invoke(self.mode.take().unwrap()))
            }
        }
    }
}