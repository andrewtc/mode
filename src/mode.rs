// Copyright 2019 Andrew Thomas Christensen
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the
// MIT license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option. This file may not be copied,
// modified, or distributed except according to those terms.

use crate::Family;

/// Trait that defines the transition behavior of a state within an `Automaton`.
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
///     type Input = ();
///     type Output = Box<dyn MyMode>;
/// }
/// 
/// trait MyMode : boxed::Mode<Family = MyFamily> {
///     // TODO: Define some common interface for ModeA and ModeB.
/// }
/// 
/// struct ModeA; // TODO: Add fields.
/// impl MyMode for ModeA { }
/// 
/// impl boxed::Mode for ModeA {
///     type Family = MyFamily;
///     fn swap(self : Box<Self>, _input : ()) -> Box<dyn MyMode> {
///         // Transition to ModeB. ModeA can swap to ModeB because both share the same Family.
///         Box::new(ModeB)
///     }
/// }
/// 
/// struct ModeB; // TODO: Add fields.
/// impl MyMode for ModeB { }
/// 
/// impl boxed::Mode for ModeB {
///     type Family = MyFamily;
///     fn swap(self : Box<Self>, _input : ()) -> Box<dyn MyMode> { self } // Returning self means don't transition.
/// }
/// ```
/// 
/// # Transitioning
/// `Mode`s can choose to transition to any other `Mode` with the same `Family` associated `type`. This is accomplished
/// by returning a new `Mode` from the `swap()` function, which will cause the parent `Automaton` to switch to this
/// `Mode` immediately. Since 
/// 
/// See [`Automaton::next()`](struct.Automaton.html#method.next) for more details.
/// 
/// # The `Family` parameter
/// You will notice from the [example](#usage) above that `ModeA` and `ModeB` implement `Mode` and `MyMode` separately,
/// but the `MyMode` trait itself does **not** extend `Mode`, i.e. is defined as `trait MyMode` as opposed to
/// `trait MyMode : Mode<Base = MyMode>`. We want to use `MyMode` as the `Base` type for `ModeA` and `ModeB`, but
/// unfortunately having `MyMode` extend `Mode<Base = MyMode>` would create a circular dependency between the two types,
/// and would cause a compile error. Hence, while it is possible to cast `ModeA` or `ModeB` to `MyMode` or `Mode`,
/// casting between `MyMode` and `Mode` is not allowed.
/// 
/// # Returning a value from `Mode::swap()`
/// It is possible to output a value in addition to the `Mode` that is returned from `swap()`. In order to do this, the
/// `Output` type of the `Family` for this `Mode` should be given a tuple containing `Family::Mode` as the first
/// parameter and some other type as the second, which will become the return type for `Mode::swap()`. The
/// `Automaton::next_with_output()` function will interpret the first parameter as the new `Mode` to switch in, and the
/// second parameter will be returned as a result.
/// 
/// **NOTE:** If you do this, you will be required to use `Automaton::next_with_output()` or
/// `Automaton::next_with_input_and_output()`, instead of `Automaton::next()` or `Automaton::next_with_input()`, due to
/// the `impl` bounds on these functions.
/// 
/// # Passing context into `Mode::swap()`
/// The `Mode::swap()` function takes a single `input` parameter that can be used in situations where some context is
/// necessary in order to allow the current `Mode` to swap itself. This parameter is of type `Family::Input`, and is
/// passed into the `swap()` function by value. If no context is necessary to switch `Mode`s, the value can effectively
/// be ignored by setting `Family::Input` to the empty tuple type, `()`.
/// 
/// **NOTE:** When a non-empty `Family::Input` type is used, you will be required to use `Automaton::next_with_input()`
/// or `Automaton::next_with_input_and_output()`, instead of `Automaton::next()` or `Automaton::next_with_output()`, due
/// to the `impl` bounds on these functions.
/// 
/// # Alternative `trait Mode`s for pointer types
/// When storing `Mode`s with a large amount of data or that should be accessed through some `dyn Trait` reference, it
/// is desirable to have the `Automaton` operate on a **pointer** to a `Mode`, as opposed to storing the current `Mode`
/// in place. This is possible by setting the `Family::Mode` type to a pointer type wrapping a `Family::Base`, e.g.
/// 
/// ```
/// use mode::Family;
/// # use mode::boxed::Mode;
/// #
/// # trait SomeTrait : Mode<Family = FamilyWithPointerMode> { }
/// 
/// struct FamilyWithPointerMode;
/// impl Family for FamilyWithPointerMode {
///     type Base = dyn SomeTrait;
///     type Mode = Box<dyn SomeTrait>; // All Modes in this Family will be stored as a Box<dyn SomeTrait> internally.
///     type Input = ();
///     type Output = Box<dyn SomeTrait>;
/// }
/// ```
/// 
/// However, when doing so, the responsibility for swapping in the next `Mode` needs to be delegated to the
/// type **stored** in the pointer, not the pointer itself.
/// 
/// Hence, this module defines a number of other `trait Mode`s that are meant to be extended **in place of**
/// `mode::Mode` when a `std` pointer type, e.g. `Box` or `Arc`, is being used. These are all stored in separate
/// submodules that rougly correspond to the path of the pointer type under `std`, e.g. `mode::boxed::Mode` wraps a
/// `std::boxed::Box`, and `mode::sync::Mode` wraps a `std::sync::Arc`. These define a slightly different `swap()`
/// function that accepts the **pointer** type as `self`, e.g. `self : Box<Self>`. There are multiple advantages to
/// this, but the main one is that the `Mode` implementation can return its own pointer from the `swap()` function when
/// it wants to remain active, instead of returning a new pointer wrapping itself. Moving a pointer into and out of the
/// `swap()` function can be **much** cheaper than moving the object itself around, especially for `Mode`s that store
/// large amounts of data.
/// 
/// When writing an `impl` for a `struct` in a `Family` that stores a pointer type, the corresponding `Mode`
/// implementation (e.g. `mode::boxed::Mode`) should be used **instead of** `mode::Mode` itself. The crate provides auto
/// `impl mode::Mode`s for each of these, allowing them to be used in the `Automaton`. (See example below.)
/// 
/// ```
/// use mode::{sync, Family};
/// use std::sync::Arc;
/// 
/// trait SomeTrait : sync::Mode<Family = FamilyWithArcMode> { }
/// 
/// struct FamilyWithArcMode;
/// impl Family for FamilyWithArcMode {
///     type Base = dyn SomeTrait;
///     type Mode = Arc<dyn SomeTrait>; // All Modes in this Family will be stored as an Arc<dyn SomeTrait> internally.
///     type Input = ();
///     type Output = Arc<dyn SomeTrait>;
/// }
/// 
/// struct SomeMode;
/// impl SomeTrait for SomeMode { } // TODO
/// 
/// // Note that we ONLY impl sync::Mode for SomeMode. There is an auto-impl of mode::Mode for Arc<T : sync::Mode>, so
/// // we don't need to implement mode::Mode ourselves.
/// //
/// impl sync::Mode for SomeMode {
///     type Family = FamilyWithArcMode;
///     fn swap(self : Arc<Self>, _input : ()) -> Arc<dyn SomeTrait> {
///         // TODO: Insert logic here to switch states by returning a different Arc.
///         self
///     }
/// }
/// ```
/// 
pub trait Mode {
    /// The `Family` type to which this `Mode` implementation belongs. `Mode` implementations are **only** allowed
    /// to return another `Mode`s from the `swap()` method if it has the exact same `Family` type as itself.
    /// Swapping between `Mode`s with different `Family` types is **not** allowed, even if the associated `type`s in
    /// two separate `Family` implementations are identical to each other. This is because reusing the same
    /// `Base` interface or `Mode` type between `Mode` implementations does not *necessarily* imply that both states
    /// are meant to represent states in the same state machine.
    /// 
    /// See [`Family`](trait.Family.html) for more details.
    /// 
    type Family : Family + ?Sized;

    /// Every time one of the `Automaton::next*()` functions is called, the `Automaton` will call this function on the
    /// current `Mode` to determine whether it wants another `Mode` to become active. If this function returns `self`,
    /// the current `Mode` will remain active. However, if it returns another object implementing `Mode` with the same
    /// `Family` type, the `Automaton` will make the `Mode` that was returned active immediately after the `swap()`
    /// function returns, consuming the `Mode` that was previously active. Since the original `Mode` is consumed, it is
    /// possible for the current `Mode` to move state out of itself and into the new `Mode` being created.
    /// 
    /// This function returns `Self::Family::Output`, which can either be the `Self::Family::Mode` type or some
    /// `(mode, result)` tuple, where `mode` represents the new `Self::Family::Mode` to switch in as active and
    /// `result` represents some value that should be returned to the caller. Regardless of the `Self::Family::Output`
    /// type, the `Mode` **must** return a `Self::Family::Mode` type to transition in. If `self` is returned from this
    /// function, the current `Mode` will remain active.
    /// 
    /// See [`Automaton::next()`](struct.Automaton.html#method.next) and
    /// [`Automaton::next_with_output()`](struct.Automaton.html#method.next_with_output) for more details.
    /// 
    fn swap(self, input : <Self::Family as Family>::Input) -> <Self::Family as Family>::Output;
}

/// Defines types that can be used to set up an `Automaton` that stores a `Box<Mode>` instead of a `Mode` in place.
/// 
pub mod boxed {
    use crate::Family;

    /// Alternate `trait Mode` that takes a `Box<Mode>` as the `self` parameter instead of `Mode`.
    /// 
    /// For more on how to use this `trait`, see `mode::Mode`.
    /// 
    pub trait Mode {
        /// The `Family` type to which this `Mode` implementation belongs. In order to use the `boxed::Mode` trait, this
        /// `Family` should be a `Box<T>` where `T : boxed::Mode`.
        /// 
        /// See `mode::Mode` for more details.
        /// 
        type Family : Family + ?Sized;

        /// Will be called on the current `Mode` by `Automaton::next()` or `Automaton::next_with_output()` in order to
        /// determine whether it wants the `Automaton` to transition to another `Mode`. Note that this `trait`'s
        /// `swap()` function takes a `Box<Self>` instead of just `self`.
        /// 
        /// See `mode::Mode` for more details.
        /// 
        fn swap(self : Box<Self>, input : <Self::Family as Family>::Input) -> <Self::Family as Family>::Output;
    }

    impl<T, F> crate::Mode for Box<T>
        where
            F : Family + ?Sized,
            T : self::Mode<Family = F> + ?Sized,
    {
        type Family = F;

        fn swap(self, input : <Self::Family as Family>::Input) -> <Self::Family as Family>::Output {
            self.swap(input)
        }
    }
}

/// Defines types that can be used to set up an `Automaton` that stores an `Rc<Mode>` instead of a `Mode` in place.
/// 
pub mod rc {
    use crate::Family;
    use std::rc::Rc;

    /// Alternate `trait Mode` that takes an `Rc<Mode>` as the `self` parameter instead of `Mode`.
    /// 
    /// For more on how to use this `trait`, see `mode::Mode`.
    /// 
    pub trait Mode {
        /// The `Family` type to which this `Mode` implementation belongs. In order to use the `rc::Mode` trait, this
        /// `Family` should be an `Rc<T>` where `T : rc::Mode`.
        /// 
        /// See `mode::Mode` for more details.
        /// 
        type Family : Family + ?Sized;

        /// Will be called on the current `Mode` by `Automaton::next()` or `Automaton::next_with_output()` in order to
        /// determine whether it wants the `Automaton` to transition to another `Mode`. Note that this `trait`'s
        /// `swap()` function takes an `Rc<Self>` instead of just `self`.
        /// 
        /// See `mode::Mode` for more details.
        /// 
        fn swap(self : Rc<Self>, input : <Self::Family as Family>::Input) -> <Self::Family as Family>::Output;
    }

    impl<T, F> crate::Mode for Rc<T>
        where
            F : Family + ?Sized,
            T : self::Mode<Family = F> + ?Sized,
    {
        type Family = F;

        fn swap(self, input : <Self::Family as Family>::Input) -> <Self::Family as Family>::Output {
            self.swap(input)
        }
    }
}

/// Defines types that can be used to set up an `Automaton` that stores an `Arc<Mode>` instead of a `Mode` in place.
/// 
pub mod sync {
    use crate::Family;
    use std::sync::Arc;

    /// Alternate `trait Mode` that takes an `Arc<Mode>` as the `self` parameter instead of `Mode`.
    /// 
    /// For more on how to use this `trait`, see `mode::Mode`.
    /// 
    pub trait Mode {
        /// The `Family` type to which this `Mode` implementation belongs. In order to use the `sync::Mode` trait, this
        /// `Family` should be an `Arc<T>` where `T : sync::Mode`.
        /// 
        /// See `mode::Mode` for more details.
        /// 
        type Family : Family + ?Sized;

        /// Will be called on the current `Mode` by `Automaton::next()` or `Automaton::next_with_output()` in order to
        /// determine whether it wants the `Automaton` to transition to another `Mode`. Note that this `trait`'s
        /// `swap()` function takes an `Arc<Self>` instead of just `self`.
        /// 
        /// See `mode::Mode` for more details.
        /// 
        fn swap(self : Arc<Self>, input : <Self::Family as Family>::Input) -> <Self::Family as Family>::Output;
    }

    impl<T, F> crate::Mode for Arc<T>
        where
            F : Family + ?Sized,
            T : self::Mode<Family = F> + ?Sized,
    {
        type Family = F;

        fn swap(self, input : <Self::Family as Family>::Input) -> <Self::Family as Family>::Output {
            self.swap(input)
        }
    }
}