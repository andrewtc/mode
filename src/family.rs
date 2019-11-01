// Copyright 2019 Andrew Thomas Christensen
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the
// MIT license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option. This file may not be copied,
// modified, or distributed except according to those terms.

use crate::Mode;

/// A meta-`trait` defining the common `Base` type and `Mode::swap()` return type used by a related group of `Mode`
/// implementations. `Mode`s can **only** transition to other `Mode`s within the same `Family`, i.e. where both `Mode`s
/// share the same `Family` associated `type`.
/// 
/// # The `Base` type
/// The `Base` associated type may be either a `dyn Trait` or a concrete type that represents how the current `Mode` can
/// be accessed from outside the `Automaton`.
/// 
/// If given some `dyn Trait`, **only** functions common to the `trait` interface will be callable on the current
/// `Mode`, since the `Automaton` will **only** allow it to be borrowed via a `trait` reference. However, the
/// `Automaton` will allow swapping between different concrete types `struct`s that implement this common interface,
/// provided that they are all in the same `Family`, i.e. all implement `Mode` with a common `Family` associated type.
/// 
/// If given a `struct`, **all** functions and members defined on the inner type will be accessible from outside the
/// `Automaton`. However, this also implies that **all** states in the `Automaton` will be represented by instances of
/// this same concrete type.
/// 
/// # Usage
/// To define a new `Family` of `Mode`s, simply define a new unit `struct` and `impl Family` for it. This will allow the
/// associated `type`s within `Family` to be defined for that specific `struct`, with the `struct` representing the
/// common usage pattern of all `Mode`s with a `Family` associated `type` equal to that `struct`. (See examples below.)
/// 
/// ## A `Family` where `Base` is a concrete type
/// ```
/// use mode::{boxed, Family, Mode};
/// 
/// enum SomeMode { A, B, C }
/// impl Mode for SomeMode {
///     type Family = SomeFamily;
///     fn swap(self) -> Self {
///         match self {
///             Self::A => Self::B,
///             Self::B => Self::C,
///             Self::C => Self::A,
///         }
///     }
/// }
/// 
/// struct SomeFamily;
/// 
/// impl Family for SomeFamily {
///     type Base = SomeMode; // All SomeFamily Modes will be visible as SomeMode from outside the Automaton.
///     type Mode = SomeMode; // The current Mode in the Automaton will be stored as a SomeMode in-place.
///     type Output = SomeMode; // All SomeFamily Modes will be able to swap in any new SomeMode.
/// }
/// ```
/// 
/// ## A `Family` where `Base = dyn Trait`
/// ```
/// use mode::{boxed, Family};
/// 
/// trait SomeTrait : boxed::Mode<Family = SomeFamily> {
///     // ...
/// }
/// 
/// struct SomeFamily;
/// 
/// impl Family for SomeFamily {
///     type Base = dyn SomeTrait; // All SomeFamily Modes will expose their SomeTrait interface via the Automaton.
///     type Mode = Box<dyn SomeTrait>; // The current Mode in the Automaton will be stored as a Box<dyn SomeTrait>.
///     type Output = Box<dyn SomeTrait>; // All SomeFamily Modes will be able to swap in any new Box<dyn SomeTrait>.
/// }
/// ```
/// 
pub trait Family {
    /// The user-facing interface for the `Mode` that will be exposed via the `Automaton::borrow_mode()` and
    /// `Automaton::borrow_mode_mut()` functions. This can be either a concrete type or a `dyn Trait`, depending on
    /// whether `Self::Mode` is a pointer type or not.
    /// 
    /// **NOTE:** This is **not** the actual type that will be stored in `Automaton` and passed into the `Mode::swap()`
    /// function. This is just the public interface for the current `Mode` that will be exposed by the `Automaton`.
    /// 
    type Base : ?Sized;

    /// The actual type that will be stored in `Automaton` and passed into the `Mode::swap()` function. For a `Family`
    /// where `Self::Base` is a **concrete** type, this should be set to the **same** type as `Self::Base`. On the other
    /// hand, if `Self::Base` is a `dyn Trait`, this should be set to some pointer type capable of storing `Self::Base`,
    /// e.g. `Box<Self::Base>` or `Rc<Self::Base>`.
    /// 
    type Mode : Mode<Family = Self>;

    /// The type of the value that will be returned from the `Mode::swap()` function. This should usually be set to the
    /// same value as `Self::Mode`. However, this can also be a `(Self::Mode, T)` tuple, where `T` is a type that will
    /// be returned from the `Automaton::next_with_output()` function. This second tuple parameter can be used to return
    /// information about what happened during the transition, e.g. a `bool` representing whether a new `Mode` was
    /// swapped in, or a `Result<(), SomeError>` indicating whether or not an error occurred while transitioning.
    /// 
    type Output;
}