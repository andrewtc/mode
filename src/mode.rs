// Copyright 2019 Andrew Thomas Christensen
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the
// MIT license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option. This file may not be copied,
// modified, or distributed except according to those terms.

use crate::Family;
use std::{rc::Rc, sync::Arc};

/// Trait that defines a state within some `Family`, and can be made active in an `Automaton`.
/// 
/// Every `Automaton` contains a single `Mode` instance that represents the active state of the state machine. An
/// `Automaton<F>` can **only** switch between `Mode`s with the same `Family` type `F`. The `Automaton` only allows the
/// active `Mode` to be accessed as a `F::Base` reference, so only functions exposed on the `Base` type are callable on
/// the `Mode` from outside the `Automaton`.
/// 
/// See [`Automaton`](struct.Automaton.html) for more details.
/// 
/// # Usage
/// ```
/// use mode::*;
/// 
/// struct MyFamily;
/// impl Family for MyFamily {
///     type Base = dyn MyMode;
///     type Mode = Box<dyn MyMode>;
/// }
/// 
/// trait MyMode : Mode<Family = MyFamily> {
///     // TODO: Define some common interface for ModeA and ModeB.
///     fn is_mode_a(&self) -> bool;
///     fn is_mode_b(&self) -> bool;
/// 
///     // This function will be used to delegate the responsibility for swapping to the active Mode in the Automaton.
///     fn swap(self : Box<Self>) -> Box<dyn MyMode>;
/// }
/// 
/// struct ModeA; // TODO: Add fields.
/// impl Mode for ModeA { type Family = MyFamily; }
/// impl MyMode for ModeA {
///     fn is_mode_a(&self) -> bool { true }
///     fn is_mode_b(&self) -> bool { false }
///     fn swap(self : Box<Self>) -> Box<dyn MyMode> {
///         // Transition to ModeB. ModeA can swap to ModeB because both share the same Family.
///         Box::new(ModeB)
///     }
/// }
/// 
/// struct ModeB; // TODO: Add fields.
/// impl Mode for ModeB { type Family = MyFamily; }
/// impl MyMode for ModeB {
///     fn is_mode_a(&self) -> bool { false }
///     fn is_mode_b(&self) -> bool { true }
///     fn swap(self : Box<Self>) -> Box<dyn MyMode> { self } // Returning self means don't transition.
/// }
/// 
/// fn main() {
///     // Create an Automaton, starting in ModeA.
///     let mut automaton = MyFamily::automaton_with_mode(Box::new(ModeA));
/// 
///     // Allow Modes to swap between each other. (Call this whenever Modes should be allowed to transition.)
///     Automaton::next(&mut automaton, |current_mode| current_mode.swap());
/// 
///     // MyMode functions can be called on the Automaton to dispatch them to the current Mode, via Deref coercion.
///     assert!(!automaton.is_mode_a() && automaton.is_mode_b());
/// }
/// ```
/// 
/// # Tying `Mode`s together with the `Family` parameter
/// In the [example](#usage) above, `ModeA` and `ModeB` both implement `Mode` with a `Family` type of `MyFamily`.
/// Conceptually, this means that both `ModeA` and `ModeB` are part of the same state machine, and therefore an
/// `Automaton` of type `Automaton<MyFamily>` is **only** allowed to switch between `Mode`s of the same `Family` type
/// when `Automaton::next()` is called. See [`Automaton::next()`](struct.Automaton.html#method.next) for more details.
/// 
/// # Blanket `Mode` implementations for pointer types (e.g. `Box`, `Rc`, `Arc`)
/// When storing `Mode`s with a large amount of data or that should be accessed through some `dyn Trait` reference, it
/// is desirable to have the `Automaton` store a **pointer** to a `Mode`, as opposed to storing the current `Mode` in
/// place. This is possible by setting the `Family::Mode` type to a pointer type wrapping a `Family::Base`, e.g.
/// 
/// ```
/// use mode::{Family, Mode};
/// #
/// # trait SomeTrait : Mode<Family = FamilyWithPointerMode> { }
/// 
/// struct FamilyWithPointerMode;
/// impl Family for FamilyWithPointerMode {
///     type Base = dyn SomeTrait;
///     type Mode = Box<dyn SomeTrait>; // All Modes in this Family will be stored as a Box<dyn SomeTrait> internally.
/// }
/// ```
/// 
/// When doing so, it's usually a good idea to delegate the responsibility for swapping in the next `Mode` to the type
/// **stored** in the pointer, not the pointer itself. Hence, this module defines some blanket `Mode` implementations
/// for various pointer types wrapping `T`, where `T` implements `Mode`. These are given the same `Family` type as `T`,
/// and allow the pointer type, e.g. `Box<T>`, to be used as the `Mode` type for the `Family`. The main advantage of
/// doing this is that the callback passed into the `Automaton::next()` function will accept the current `Mode` by
/// pointer instead of by value, and it can simply return a pointer to itself if it wishes to remain active. Moving a
/// pointer into and out of the function like this can be **much** cheaper than moving around the current `Mode` by
/// value, particularly for `Mode`s that store large amounts of data. (See example below.)
/// 
/// ```
/// use mode::{Family, Mode};
/// use std::sync::Arc;
/// 
/// trait SomeTrait : Mode<Family = FamilyWithArcMode> {
///     // NOTE: This function will delegate the responsibility for swapping in a new Arc<dyn SomeTrait> to the current,
///     // concrete Mode type. This is advantageous because it allows the current Mode to move large amounts of state
///     // out of itself into the new Mode being swapped in, if it wants to.
///     fn swap(self : Arc<Self>) -> Arc<dyn SomeTrait>;
/// }
/// 
/// struct FamilyWithArcMode;
/// impl Family for FamilyWithArcMode {
///     type Base = dyn SomeTrait;
///     type Mode = Arc<dyn SomeTrait>; // All Modes in this Family will be stored as an Arc<dyn SomeTrait> internally.
/// }
/// 
/// struct SomeMode;
/// impl SomeTrait for SomeMode {
///     // TODO: Switch Modes by returning a different Arc<dyn SomeTrait> from this function.
///     fn swap(self : Arc<Self>) -> Arc<dyn SomeTrait> { self }
/// }
/// 
/// // NOTE: We ONLY impl Mode for SomeMode. There is an auto-impl of Mode for Arc<T : Mode>, so we don't need to
/// // implement Mode for Arc<dyn SomeTrait>.
/// //
/// impl Mode for SomeMode {
///     type Family = FamilyWithArcMode;
/// }
/// ```
/// 
pub trait Mode {
    /// The `Family` type to which this `Mode` implementation belongs. An `Automaton` can **only** switch between
    /// `Mode`s of the exact same `Family` type, matching the `Family` type of the `Automaton`. Swapping between `Mode`s
    /// with different `Family` types is **not** allowed, even if the `Base` and `Mode` associated `type`s of both
    /// `Family` implementations are identical. This is because `Mode`s with the same `Family` type represent finite
    /// states that are part of the same state machine.
    /// 
    /// See [`Family`](trait.Family.html) for more details.
    /// 
    type Family : Family + ?Sized;
}

/// Blanket `impl` that allows a `Box<T : Mode>` to be used as the `Mode` associated `type` for a `Family`.
/// 
impl<T, F> Mode for Box<T>
    where
        F : Family + ?Sized,
        T : Mode<Family = F> + ?Sized,
{
    type Family = F;
}

/// Blanket `impl` that allows an `Rc<T : Mode>` to be used as the `Mode` associated `type` for a `Family`.
/// 
impl<T, F> Mode for Rc<T>
    where
        F : Family + ?Sized,
        T : Mode<Family = F> + ?Sized,
{
    type Family = F;
}

/// Blanket `impl` that allows an `Arc<T : Mode>` to be used as the `Mode` associated `type` for a `Family`.
/// 
impl<T, F> Mode for Arc<T>
    where
        F : Family + ?Sized,
        T : Mode<Family = F> + ?Sized,
{
    type Family = F;
}