// Copyright 2019 Andrew Thomas Christensen
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the
// MIT license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option. This file may not be copied,
// modified, or distributed except according to those terms.

use crate::{Family, Mode};
use std::{
    convert::{AsRef, AsMut},
    borrow::{Borrow, BorrowMut},
    fmt
};
use std::ops::{Deref, DerefMut};

/// Represents a state machine over a set of `Mode`s that can be referenced via some common interface `Base`.
/// 
/// The `Automaton` contains a single, active `Mode` that represents the current state of the state machine. The current
/// `Mode` is accessible via the `borrow_mode()` and `borrow_mode_mut()` functions, which return a `Base` reference.
/// Functions and members of the inner `Base` type can also be accessed directly via the `Deref` and `DerefMut` traits.
/// The `Automaton` provides a `perform_transitions()` function that should be called at some point in order to allow
/// the current `Mode` to transition another `Mode` in, if desired.
/// 
/// See [`Mode::get_transition()`](trait.Mode.html#tymethod.get_transition) for more details.
/// 
/// # The `'a` lifetime
/// Most types in this library include an explicit `'a` lifetime, which represents the lifetime of the `Automaton`
/// wrapping each `Mode`. In order for a `Mode` to be used with an `Automaton`, all references within the `Mode` must
/// outlive the parent `Automaton`. Having this lifetime allows for the creation of `Mode`s that store references to
/// objects that outlive the `Automaton` but are still declared on the stack. (See example below.)
/// 
/// ## Example
/// ```
/// use mode::*;
/// 
/// struct IncrementMode<'a> {
///     pub number : &'a mut u32,
///     pub step : u32,
/// }
/// 
/// impl<'a> IncrementMode<'a> {
///     fn update(&mut self) {
///         *self.number += self.step
///     }
/// 
///     fn get_step(&self) -> u32 { self.step }
/// }
/// 
/// impl<'a> Mode<'a> for IncrementMode<'a> {
///     type Base = Self;
///     fn as_base(&self) -> &Self::Base { self }
///     fn as_base_mut(&mut self) -> &mut Self::Base { self }
///     fn get_transition(&mut self) -> Option<TransitionBox<Self>> {
///         if self.step > 0 {
///             // Transition to another IncrementMode with a lower step amount.
///             Some(Box::new(|previous : Self| {
///                 IncrementMode { number: previous.number, step: previous.step - 1 }
///             }))
///         }
///         else { None } // None means don't transition
///     }
/// }
/// 
/// // Create a shared counter and pass it into the Mode.
/// let mut number : u32 = 0;
/// 
/// // NOTE: The Automaton can't outlive our shared counter.
/// {
///     let mut automaton =
///         Automaton::with_initial_mode(IncrementMode { number: &mut number, step: 10 });
///     
///     // NOTE: Automaton implements Deref so that all Base functions can be called
///     // through an Automaton reference.
///     while automaton.get_step() > 0 {
///         // Update the current Mode.
///         automaton.update();
///     
///         // Let the Automaton handle transitions.
///         automaton.perform_transitions();
///     }
/// }
/// 
/// // Make sure we got the right result.
/// assert_eq!(number, 55);
/// ```
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
/// # impl<'a> Mode<'a> for SomeMode {
/// #     type Base = MyMode;
/// #     fn as_base(&self) -> &Self::Base { self }
/// #     fn as_base_mut(&mut self) -> &mut Self::Base { self }
/// #     fn get_transition(&mut self) -> Option<TransitionBox<Self>> { None }
/// # }
/// # 
/// // Use with_initial_mode() to create the Automaton with an initial state.
/// let mut automaton = Automaton::with_initial_mode(SomeMode);
/// 
/// // Functions can be called on the inner Mode through an Automaton reference
/// // via the Deref and DerefMut traits
/// automaton.some_fn();
/// automaton.some_mut_fn();
/// 
/// // If you want to be more explicit, use borrow_mode() or borrow_mode_mut();
/// automaton.borrow_mode().some_fn();
/// automaton.borrow_mode_mut().some_mut_fn();
/// 
/// // Let the Automaton handle transitions.
/// automaton.perform_transitions();
/// ```
/// 
pub struct Automaton<F>
    where F : Family + ?Sized
{
    mode : Option<F::Mode>,
}

impl<F> Automaton<F>
    where F : Family + ?Sized
{
    /// Creates a new `Automaton` with the specified `mode`, which will be the initial active `Mode` for the `Automaton`
    /// that is returned.
    /// 
    pub fn with_mode<M>(mode : M) -> Self
        where M : Into<F::Mode>
    {
        Self {
            mode : Some(mode.into()),
        }
    }
}

impl<F> Automaton<F>
    where
        F : Family + ?Sized,
        F::Mode : Borrow<F::Base>,
{
    pub fn borrow_mode(&self) -> &F::Base {
        self.mode.as_ref()
            .expect("Cannot borrow current Mode because another swap is already taking place!")
            .borrow()
    }
}

impl<F> Automaton<F>
    where
        F : Family + ?Sized,
        F::Mode : BorrowMut<F::Base>,
{
    pub fn borrow_mode_mut(&mut self) -> &mut F::Base {
        self.mode.as_mut()
            .expect("Cannot borrow current Mode because another swap is already taking place!")
            .borrow_mut()
    }
}

impl<F, M> Automaton<F>
    where
        F : Family<Mode = M, Output = M> + ?Sized,
        M : Mode<Family = F>,
{
    /// Calls `get_transition()` on the current `Mode` to determine whether it wants to transition out. If a
    /// `Transition` is returned, the `Transition` callback will be called on the current `Mode`, swapping in whichever
    /// `Mode` it returns as a result.
    /// 
    /// For convenience, this function returns a `bool` representing whether a `Transition` was performed or not. A
    /// result of `true` indicates that the `Automaton` transitioned to another `Mode`. If no `Transition` was performed
    /// and the previous `Mode` is still active, returns `false`.
    /// 
    /// See [`Transition`](trait.Transition.html) and
    /// [`Mode::get_transition()`](trait.Mode.html#tymethod.get_transition) for more details.
    /// 
    pub fn next(this : &mut Self) {
        let next =
            this.mode.take()
                .expect("Cannot swap to next Mode because another swap is already taking place!")
                .swap();
        this.mode = Some(next);
    }
}

impl<F, M, Output> Automaton<F>
    where
        F : Family<Mode = M, Output = (M, Output)> + ?Sized,
        M : Mode<Family = F>,
{
    pub fn next_with_output(this : &mut Self) -> Output {
        let (next, result) =
            this.mode.take()
                .expect("Cannot swap to next Mode because another swap is already taking place!")
                .swap();
        this.mode = Some(next);
        result
    }
}

impl<F> AsRef<F::Base> for Automaton<F>
    where
        F : Family + ?Sized,
        F::Mode : Borrow<F::Base>,
{
    /// Returns an immutable reference to the current `Mode` as a `&Self::Base`, allowing immutable functions to be
    /// called on the inner `Mode`.
    /// 
    fn as_ref(&self) -> &F::Base {
        self.borrow_mode()
    }
}

impl<F> AsMut<F::Base> for Automaton<F>
    where
        F : Family + ?Sized,
        F::Mode : BorrowMut<F::Base>,
{
    fn as_mut(&mut self) -> &mut <F as Family>::Base {
        self.borrow_mode_mut()
    }
}

impl<F> Deref for Automaton<F>
    where
        F : Family + ?Sized,
        F::Mode : Borrow<F::Base>,
{
    type Target = F::Base;

    /// Returns an immutable reference to the current `Mode` as a `&Self::Base`, allowing immutable functions to be
    /// called on the inner `Mode`.
    /// 
    fn deref(&self) -> &F::Base {
        self.borrow_mode()
    }
}

impl<F> DerefMut for Automaton<F>
    where
        F : Family + ?Sized,
        F::Mode : Borrow<F::Base> + BorrowMut<F::Base>,
{
    /// Returns a mutable reference to the current `Mode` as a `&mut Self::Base`, allowing mutable functions to be
    /// called on the inner `Mode`.
    /// 
    fn deref_mut(&mut self) -> &mut F::Base {
        self.borrow_mode_mut()
    }
}

impl<F> Automaton<F>
    where F : Family + ?Sized + Default
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
    /// impl<'a> Mode<'a> for ConcreteMode {
    ///     type Base = Self;
    ///     fn as_base(&self) -> &Self { self }
    ///     fn as_base_mut(&mut self) -> &mut Self { self }
    ///     fn get_transition(&mut self) -> Option<TransitionBox<Self>> {
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
    /// // Create an Automaton with a default Mode.
    /// // NOTE: Deref coercion allows us to access the CounterMode's count variable
    /// // through an Automaton reference.
    /// let mut automaton = Automaton::<ConcreteMode>::new();
    /// assert!(automaton.count == 0);
    /// 
    /// // Keep transitioning the current Mode out until we reach the target state
    /// // (i.e. a count of 10).
    /// while automaton.count < 10 {
    ///     automaton.perform_transitions();
    /// }
    /// ```
    /// 
    pub fn new() -> Self {
        Self {
            mode : Default::default(),
        }
    }
}

impl<F> Default for Automaton<F>
    where F : Family + ?Sized + Default
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
/// impl<'a> Mode<'a> for MyMode {
///     type Base = MyBase;
///     fn as_base(&self) -> &Self::Base { self }
///     fn as_base_mut(&mut self) -> &mut Self::Base { self }
///     fn get_transition(&mut self) -> Option<TransitionBox<Self>> { None } // TODO
/// }
/// 
/// let automaton = Automaton::with_initial_mode(MyMode { foo: 3, bar: "Hello, World!" });
/// dbg!(automaton);
/// ```
/// 
impl<F> fmt::Debug for Automaton<F>
    where
        F : Family + ?Sized,
        F::Mode : Borrow<F::Base>,
        F::Base : fmt::Debug,
{
    fn fmt(&self, formatter : &mut fmt::Formatter) -> fmt::Result {
        formatter.debug_struct("Automaton")
            .field("mode", &self.borrow_mode())
            .finish()
    }
}

impl<F> fmt::Display for Automaton<F>
    where
        F : Family + ?Sized,
        F::Mode : Borrow<F::Base>,
        F::Base : fmt::Display,
{
    fn fmt(&self, formatter : &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", self.borrow_mode())
    }
}