// Copyright 2019 Andrew Thomas Christensen
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the
// MIT license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option. This file may not be copied,
// modified, or distributed except according to those terms.

use crate::{Automaton, Mode};

/// A meta-`trait` defining the common `Base` type and `Mode` storage conventions used by a related group of `Mode`
/// implementations. `Mode`s can **only** transition to other `Mode`s within the same `Family`, i.e. where both `Mode`s
/// share the same `Family` associated `type`.
/// 
/// # The `Base` type
/// The `Base` associated type may be either a `dyn Trait` or a concrete type that represents how the current `Mode` can
/// be accessed from outside the `Automaton`.
/// 
/// If given some `dyn Trait`, **only** functions common to the `trait` interface will be callable on the current
/// `Mode`, as the `Automaton` will **only** allow it to be borrowed via a `trait` reference. However, the `Automaton`
/// will allow swapping between different concrete implementations of this common interface, provided that the `Mode`
/// associated `type` is a pointer of some kind (e.g. `Box`) and they share the same `Family` associated `type`.
/// 
/// If given a concrete type, e.g. an `enum` or `struct`, **all** functions and members defined on the inner type will
/// be accessible from outside the `Automaton`. However, this also implies that **all** states in the `Automaton` will
/// be represented by instances of this same concrete type.
/// 
/// # Usage
/// To define a new `Family` of `Mode`s, simply define a new unit `struct` and `impl Family` for it. This will allow the
/// associated `type`s within `Family` to be defined for that specific `struct`, with the `struct` representing the
/// common usage pattern of all `Mode`s with a `Family` associated `type` equal to that `struct`. (See examples below.)
/// 
/// ## A `Family` where `Base` is a concrete type
/// ```
/// use mode::{Family, Mode};
/// 
/// enum SomeMode { A, B, C }
/// 
/// impl Mode for SomeMode {
///     type Family = SomeFamily;
/// }
/// 
/// impl SomeMode {
///     pub fn swap(self) -> Self {
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
/// }
/// ```
/// 
/// ## A `Family` where `Base` is a `dyn Trait`
/// ```
/// use mode::{Mode, Family};
/// 
/// trait SomeTrait : Mode<Family = SomeFamily> {
///     // ...
/// }
/// 
/// struct SomeFamily;
/// 
/// impl Family for SomeFamily {
///     type Base = dyn SomeTrait; // All SomeFamily Modes will expose their SomeTrait interface via the Automaton.
///     type Mode = Box<dyn SomeTrait>; // The current Mode in the Automaton will be stored as a Box<dyn SomeTrait>.
/// }
/// ```
/// 
pub trait Family {
    /// The user-facing interface for the `Mode` that will be exposed via the `Automaton::borrow_mode()` and
    /// `Automaton::borrow_mode_mut()` functions. This can be either a concrete type or a `dyn Trait`, depending on
    /// whether `Self::Mode` is a pointer type or not.
    /// 
    /// **NOTE:** This is **not** the actual type that will be stored in `Automaton`. This is just the public interface
    /// for the current `Mode` that will be exposed by the `Automaton`.
    /// 
    type Base : ?Sized;

    /// The actual type that will be stored in `Automaton` and moved into the `Automaton::next()` function. For a
    /// `Family` where `Self::Base` is a **concrete** type, this should be set to the **same** type as `Self::Base`. On
    /// the other hand, if `Self::Base` is a `dyn Trait`, this should usually be set to some pointer type capable of
    /// storing `Self::Base`, e.g. `Box<Self::Base>` or `Rc<Self::Base>`.
    /// 
    type Mode : Mode<Family = Self>;

    /// Convenience function allowing an `Automaton` to be created for this `Family` type. Note that this is shorthand
    /// for `Automaton::new()`, and therefore `Self::Mode` *must* implement `Default`. See
    /// [`Automaton::new()`](struct.Automaton.html#method.new) for more details.
    /// 
    /// # Usage
    /// ```
    /// use mode::*;
    /// # 
    /// # struct SomeFamily;
    /// # impl Family for SomeFamily {
    /// #     type Base = ModeWithDefault;
    /// #     type Mode = ModeWithDefault;
    /// # }
    /// 
    /// struct ModeWithDefault { count : u32 };
    /// 
    /// impl Mode for ModeWithDefault {
    ///     type Family = SomeFamily;
    /// }
    /// 
    /// impl Default for ModeWithDefault {
    ///     fn default() -> Self {
    ///         ModeWithDefault { count: 0 }
    ///     }
    /// }
    /// 
    /// // Create an Automaton with a default Mode.
    /// let mut automaton = SomeFamily::automaton();
    /// ```
    /// 
    fn automaton() -> Automaton<Self>
        where Self::Mode : Default
    {
        Automaton::new()
    }

    /// Convenience function that returns a new `Automaton` for this `Family` type with the specified `mode` as current.
    /// Note that this is shorthand for `Automaton::with_mode()`. See
    /// [`Automaton::with_mode()`](struct.Automaton.html#method.with_mode) for more details.
    /// 
    /// # Usage
    /// ```
    /// use mode::*;
    /// 
    /// struct SomeFamily;
    /// impl Family for SomeFamily {
    ///     type Base = SomeMode;
    ///     type Mode = SomeMode;
    /// }
    /// 
    /// enum SomeMode { A, B, C };
    /// 
    /// impl Mode for SomeMode {
    ///     type Family = SomeFamily;
    /// }
    /// 
    /// // Create an Automaton with A as the initial Mode.
    /// let mut automaton = SomeFamily::automaton_with_mode(SomeMode::A);
    /// ```
    /// 
    fn automaton_with_mode(mode : Self::Mode) -> Automaton<Self> {
        Automaton::with_mode(mode)
    }
}