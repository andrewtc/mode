// Copyright 2019 Andrew Thomas Christensen
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the
// MIT license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option. This file may not be copied,
// modified, or distributed except according to those terms.

use crate::{AnyModeWrapper, Mode, ModeWrapper};
use std::fmt;

/// Represents a state machine over a set of `Mode`s that can be referenced via some common interface `Base`.
/// 
/// The `Automaton` contains a single, active `Mode` that represents the current state of the state machine. The current
/// `Mode` is accessible via the `borrow_mode()` and `borrow_mode_mut()` functions, which return a `Base` reference. The
/// `Automaton` also provides a `perform_transitions()` function that should be called at some point in order to allow
/// the current `Mode` to transition another `Mode` in, if desired.
/// 
/// See [`Mode::get_transition()`](trait.Mode.html#tymethod.get_transition) for more details.
/// 
/// # The `Base` parameter
/// 
/// The `Base` parameter may be either a `trait` (e.g. `Automaton<dyn SomeTrait>`) or a concrete type
/// (e.g. `Automaton<SomeStructThatImplsMode>`). Given a `trait`, the `Automaton` will be able to swap between **any**
/// `Mode`s that implement the trait. However, this means that the `Automaton` will **only** allow the inner `Mode` to
/// be borrowed via a trait reference, implying that **only** functions defined on the trait will be callable.
/// 
/// By contrast, if given a `struct`, **all** functions defined on the inner type will be accessible from outside the
/// `Automaton`. However, this also implies that the `Automaton` will **only** be able to switch between states of the
/// same concrete type.
/// 
/// For more on the `Base` parameter, see [`Mode`](trait.Mode.html).
/// 
/// # Usage
/// ```
/// use mode::*;
/// 
/// # trait MyMode {
/// #     fn some_fn(&self);
/// #     fn some_mut_fn(&mut self);
/// # }
/// # 
/// # struct SomeMode;
/// # impl MyMode for SomeMode {
/// #     fn some_fn(&self) { println!("some_fn was called"); }
/// #     fn some_mut_fn(&mut self) { println!("some_mut_fn was called"); }
/// # }
/// # 
/// # impl Mode for SomeMode {
/// #     type Base = MyMode;
/// #     fn as_base(&self) -> &Self::Base { self }
/// #     fn as_base_mut(&mut self) -> &mut Self::Base { self }
/// #     fn get_transition(&mut self) -> Option<Box<Transition<Self>>> { None }
/// # }
/// # 
/// // Use with_initial_mode() to create the Automaton with an initial state.
/// let mut automaton = Automaton::with_initial_mode(SomeMode);
/// 
/// // To call functions on the inner Mode, use borrow_mode() or borrow_mode_mut();
/// automaton.borrow_mode().some_fn();
/// automaton.borrow_mode_mut().some_mut_fn();
/// 
/// // Let the Automaton handle transitions.
/// automaton.perform_transitions();
/// ```
/// 
pub struct Automaton<Base>
    where Base : ?Sized
{
    current_mode : Box<AnyModeWrapper<Base = Base>>,
}

impl<Base> Automaton<Base>
    where Base : ?Sized
{
    /// Creates a new `Automaton` with the specified `initial_mode`, which will be the active `Mode` for the `Automaton`
    /// that is returned.
    /// 
    pub fn with_initial_mode<M>(initial_mode : M) -> Self
        where M : Mode<Base = Base>
    {
        Self {
            current_mode : Box::new(ModeWrapper::new(initial_mode)),
        }
    }

    /// Calls `get_transition()` on the current `Mode` to determine whether it wants to transition out. If a
    /// `Transition` is returned, the `Transition` callback will be called on the current `Mode`, swapping in whichever
    /// `Mode` it returns as a result.
    /// 
    /// See [`Transition`](trait.Transition.html) and
    /// [`Mode::get_transition()`](trait.Mode.html#tymethod.get_transition) for more details.
    /// 
    pub fn perform_transitions(&mut self) {
        if let Some(mode) = self.current_mode.perform_transitions() {
            // If a transition was performed and a new `ModeWrapper` was returned, swap in the new `Mode`.
            self.current_mode = mode;
        }
    }

    /// Returns an immutable reference to the current `Mode` as a `&Self::Base`, allowing immutable functions to be
    /// called on the inner `Mode`.
    /// 
    pub fn borrow_mode(&self) -> &Base {
        self.current_mode.borrow_mode()
    }

    /// Returns a mutable reference to the current `Mode` as a `&mut Self::Base`, allowing mutable functions to be
    /// called on the inner `Mode`.
    /// 
    pub fn borrow_mode_mut(&mut self) -> &mut Base {
        self.current_mode.borrow_mode_mut()
    }
}

impl<Base> Automaton<Base>
    where Base : Mode<Base = Base> + Default
{
    /// Creates a new `Automaton` with a default `Mode` instance as the active `Mode`.
    /// 
    /// **NOTE:** This only applies if `Base` is a **concrete** type (e.g. `Automaton<SomeStructThatImplsMode>`) that
    /// implements `Default`. If `Base` is a **trait** type (e.g. `Automaton<dyn SomeTraitThatExtendsMode>`) or you
    /// would otherwise like to specify the initial mode of the created `Automaton`, use
    /// [`with_initial_mode()`](struct.Automaton.html#method.with_initial_mode) instead.
    /// 
    /// ```
    /// use mode::*;
    /// 
    /// struct ConcreteMode { count : u32 };
    /// 
    /// impl Mode for ConcreteMode {
    ///     type Base = Self;
    ///     fn as_base(&self) -> &Self { self }
    ///     fn as_base_mut(&mut self) -> &mut Self { self }
    ///     fn get_transition(&mut self) -> Option<Box<Transition<Self>>> {
    ///         // TODO: Logic for transitioning between states goes here.
    ///         Some(Box::new(
    ///             |previous : Self| {
    ///                 ConcreteMode { count: previous.count + 1 }
    ///             }))
    ///     }
    /// }
    /// 
    /// impl Default for ConcreteMode {
    ///     fn default() -> Self {
    ///         ConcreteMode { count: 0 }
    ///     }
    /// }
    /// 
    /// // Create an Automaton with a default `Mode`.
    /// let mut automaton = Automaton::<ConcreteMode>::new();
    /// assert!(automaton.borrow_mode_mut().count == 0);
    /// 
    /// // Keep transitioning the current Mode out until we reach the target state (i.e. a count of 10).
    /// while automaton.borrow_mode_mut().count < 10 {
    ///     automaton.perform_transitions();
    /// }
    /// ```
    /// 
    pub fn new() -> Self {
        Self {
            current_mode : Box::new(ModeWrapper::<Base>::new(Default::default())),
        }
    }
}

impl<Base> Default for Automaton<Base>
    where Base : Mode<Base = Base> + Default
{
    /// Creates a new `Automaton` with the default `Mode` active. This is equivalent to calling `Automaton::new()`.
    /// 
    /// See note on [`new()`](struct.Automaton.html#method.new) for more on when this function can be used.
    /// 
    fn default() -> Self {
        Self::new()
    }
}

/// If `Base` implements `std::fmt::Debug`, `Automaton` also implements `Debug`, and will print the `current_mode`.
/// 
/// # Usage
/// ```
/// use mode::*;
/// use std::fmt;
/// 
/// trait MyBase : fmt::Debug { } // TODO: Add common interface.
/// 
/// #[derive(Debug)]
/// struct MyMode {
///     pub foo : i32,
///     pub bar : &'static str,
/// }
/// 
/// impl MyBase for MyMode { } // TODO: Implement common interface.
/// 
/// impl Mode for MyMode {
///     type Base = MyBase;
///     fn as_base(&self) -> &Self::Base { self }
///     fn as_base_mut(&mut self) -> &mut Self::Base { self }
///     fn get_transition(&mut self) -> Option<Box<Transition<Self>>> { None } // TODO
/// }
/// 
/// let automaton = Automaton::with_initial_mode(MyMode { foo: 3, bar: "Hello, World!" });
/// dbg!(automaton);
/// ```
/// 
impl<Base> fmt::Debug for Automaton<Base>
    where Base : fmt::Debug + ?Sized
{
    fn fmt(&self, formatter : &mut fmt::Formatter) -> fmt::Result {
        formatter.debug_struct("Automaton")
            .field("current_mode", &self.borrow_mode())
            .finish()
    }
}

impl<Base> fmt::Display for Automaton<Base>
    where Base : fmt::Display + ?Sized
{
    fn fmt(&self, formatter : &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", self.borrow_mode())
    }
}