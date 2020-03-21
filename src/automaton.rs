// Copyright 2019 Andrew Thomas Christensen
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the
// MIT license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option. This file may not be copied,
// modified, or distributed except according to those terms.

use crate::Family;
use std::{
    convert::{AsRef, AsMut},
    borrow::{Borrow, BorrowMut},
    fmt,
};
use std::ops::{Deref, DerefMut};

/// Represents a state machine over a set of `Mode`s within the same `Family`.
/// 
/// The `Automaton` contains a single, active `Mode` that represents the current state of the state machine. The current
/// `Mode` is accessible via `borrow_mode()` and `borrow_mode_mut()` functions, which return an `F::Base` reference, or
/// via `Deref` coercion. The `Automaton` provides a `next()` function that should be called regularly in order to allow
/// the current state to swap in another `Mode` as active, if desired.
/// 
/// See [`Automaton::next()`](#method.next) for more details.
/// 
/// # Usage
/// ```
/// use mode::*;
/// #
/// # struct SomeFamily;
/// # impl Family for SomeFamily {
/// #     type Base = dyn MyBase;
/// #     type Mode = Box<dyn MyBase>;
/// # }
/// #
/// # trait MyBase : Mode<Family = SomeFamily> {
/// #     fn some_fn(&self);
/// #     fn some_mut_fn(&mut self);
/// #     fn some_transition_fn(self : Box<Self>) -> Box<dyn MyBase>;
/// # }
/// # 
/// # struct SomeMode;
/// # 
/// # impl MyBase for SomeMode {
/// #     fn some_fn(&self) { println!("some_fn was called"); }
/// #     fn some_mut_fn(&mut self) { println!("some_mut_fn was called"); }
/// #     fn some_transition_fn(self : Box<Self>) -> Box<dyn MyBase> { self }
/// # }
/// # 
/// # impl Mode for SomeMode {
/// #     type Family = SomeFamily;
/// # }
/// 
/// // Use with_mode() to create the Automaton with an initial state.
/// // NOTE: We could alternatively use SomeFamily::automaton_with_mode() here to shorten this.
/// let mut automaton = Automaton::<SomeFamily>::with_mode(Box::new(SomeMode));
/// 
/// // Functions can be called on the inner Mode through an Automaton reference via the Deref and DerefMut traits
/// automaton.some_fn();
/// automaton.some_mut_fn();
/// 
/// // If you want to be more explicit, use borrow_mode() or borrow_mode_mut();
/// automaton.borrow_mode().some_fn();
/// automaton.borrow_mode_mut().some_mut_fn();
/// 
/// // next() can be used to transition the Automaton to a different Mode, or, as in this case, to allow the current
/// // Mode to transition itself when ready.
/// Automaton::next(&mut automaton, |current_mode| current_mode.some_transition_fn());
/// ```
/// 
/// # The `F` parameter
/// 
/// One important thing to note about the `F` generic parameter it that it is **not** the base `Mode` type that will be
/// stored in the `Automaton`, itself. Rather, it is a separate, user-defined `struct` that implements the `Family`
/// trait, representing the group of all `Mode` types that are compatible with the `Automaton`. For example, an
/// `Automaton<SomeFamily>` will **only** be able to switch between states that implement `Mode<Family = SomeFamily>`.
/// 
/// # `F::Mode`, `F::Base`, and pointer types
/// 
/// Another important thing to understand is that the actual type stored in the `Automaton` will be `F::Mode`, **not**
/// `F::Base`. This has to be the case because, while `F::Base` can be an unsized type, e.g. a `dyn Trait`, `F::Mode` is
/// **required** to be a `Sized` type, e.g. a `struct` or a pointer type like `Box`. Since `F::Mode` is required to
/// implement `Mode`, there are several blanket `impl`s defined for various pointer types, e.g. `Box<T : Mode>`, so that
/// these types can be used to store the `Mode` in the `Automaton` by pointer, as opposed to in-place.
/// 
/// One advantage of having `F::Mode` be a pointer type is that the inner `Mode` can be a very large object that would
/// otherwise be slow to move into and out of `Automaton::next()` by value. Since the convention for keeping the
/// `Automaton` in the same state is to return the same `Mode` from `Automaton::next()`, moving the `Mode` into and out
/// of the function by value would result in needless and potentially expensive copy operations. (See example below.)
/// 
/// ```
/// use mode::*;
/// 
/// struct ReallyBigFamily;
/// impl Family for ReallyBigFamily {
///     type Base = ReallyBigMode;
///     type Mode = ReallyBigMode;
/// }
/// 
/// const DATA_SIZE : usize = 1024; // 1 KiB
/// 
/// struct ReallyBigMode {
///     data : [u8; DATA_SIZE],
/// }
/// 
/// impl Default for ReallyBigMode {
///     fn default() -> Self { Self { data : [0; DATA_SIZE] } }
/// }
/// 
/// impl Mode for ReallyBigMode {
///     type Family = ReallyBigFamily;
/// }
/// 
/// fn main() {
///     let mut automaton = ReallyBigFamily::automaton();
/// 
///     // This copies all 1 MiB of current_mode into the callback, and then right back out. Not very efficient.
///     Automaton::next(&mut automaton, |current_mode| current_mode);
/// }
/// ```
/// 
/// Having `F::Mode` be a pointer type allows the **pointer** itself to be moved in and out of the `swap()` function,
/// while still allowing the responsibility of swapping states to be delegated to the stored type itself, if desired.
/// (See example below.)
/// 
/// ```
/// use mode::*;
/// 
/// struct ReallyBigFamily;
/// impl Family for ReallyBigFamily {
///     type Base = ReallyBigMode;
///     type Mode = Box<ReallyBigMode>;
/// }
/// 
/// const DATA_SIZE : usize = 1024; // 1 KiB
/// 
/// struct ReallyBigMode {
///     data : [u8; DATA_SIZE],
/// }
/// 
/// impl Default for ReallyBigMode {
///     fn default() -> Self { Self { data : [0; DATA_SIZE] } }
/// }
/// 
/// impl Mode for ReallyBigMode {
///     type Family = ReallyBigFamily;
/// }
/// 
/// fn main() {
///     let mut automaton = ReallyBigFamily::automaton();
/// 
///     // This moves the Box back out of the function, not the ReallyBigMode object itself, which is *much* cheaper!
///     Automaton::next(&mut automaton, |current_mode| current_mode);
/// }
/// ```
/// 
/// For more on the `Base` and `Mode` parameters, see [`Family`](trait.Family.html).
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
    /// **NOTE:** If `F::Base` is a type that implements `Default`, [`new()`](struct.Automaton.html#method.new) can be
    /// used instead.
    /// 
    /// Since the `F` parameter cannot be determined automatically, using this function usually requires the use of the
    /// turbofish, e.g. `Automaton::<SomeFamily>::with_mode()`. To avoid that, `Family` provides an
    /// `automaton_with_mode()` associated function that can be used instead. See
    /// [`Family::automaton_with_mode()`](trait.Family.html#method.automaton_with_mode) for more details.
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
    /// impl Mode for SomeMode {
    ///     type Family = SomeFamily;
    /// }
    /// 
    /// // Create an Automaton with A as the initial Mode.
    /// // NOTE: We could alternatively use SomeFamily::automaton_with_mode() here to shorten this.
    /// let mut automaton = Automaton::<SomeFamily>::with_mode(SomeMode::A);
    /// ```
    /// 
    pub fn with_mode(mode : F::Mode) -> Self {
        Self {
            mode : Some(mode),
        }
    }

    /// Calls `transition_fn` on the current `Mode` to determine whether it should transition out, swapping in whatever
    /// `Mode` it returns as a result. Calling this function *may* change the current `Mode`, but not necessarily.
    /// 
    /// # Usage
    /// ```
    /// use mode::*;
    /// 
    /// struct SomeFamily;
    /// impl Family for SomeFamily {
    ///     type Base = State;
    ///     type Mode = State;
    /// }
    /// 
    /// #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    /// enum State { A, B, C }
    /// impl Mode for State { type Family = SomeFamily; }
    /// impl State {
    ///     fn next(self) -> Self {
    ///         match self {
    ///             State::A => State::B,
    ///             State::B => State::C,
    ///             State::C => State::C, // Don't transition.
    ///         }
    ///     }
    /// }
    /// 
    /// fn main() {
    ///     let mut automaton = SomeFamily::automaton_with_mode(State::A);
    ///     while *automaton != State::C {
    ///         Automaton::next(&mut automaton, |current_mode| current_mode.next());
    ///         println!("Now in state {:?}.", *automaton);
    ///     }
    /// }
    /// ```
    /// 
    pub fn next<T>(automaton : &mut Self, transition_fn : T)
        where T : FnOnce(F::Mode) -> F::Mode
    {
        Self::next_with_result(automaton, |mode| (transition_fn(mode), ()))
    }

    /// Calls `transition_fn` on the current `Mode` to determine whether it should transition out, swapping in whatever
    /// `Mode` it returns as a result. Calling this function *may* change the current `Mode`, but not necessarily.
    /// 
    /// Unlike [`next()`](struct.Automaton.html#method.next), the `transition_fn` returns a tuple containing the new
    /// `Mode` to transition in as well as a return value in the second parameter. The second parameter will be returned
    /// from this function after the new `Mode` is transitioned in. This is useful for things like error handling and
    /// allowing the calling code to sense transitions between states.
    /// 
    /// # Usage
    /// ```
    /// use mode::*;
    /// 
    /// struct SomeFamily;
    /// impl Family for SomeFamily {
    ///     type Base = State;
    ///     type Mode = State;
    /// }
    /// 
    /// #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    /// enum State { A, B, C }
    /// impl Mode for State { type Family = SomeFamily; }
    /// impl State {
    ///     fn next(self) -> (Self, Self) {
    ///         match self {
    ///             State::A => (State::B, self),
    ///             State::B => (State::C, self),
    ///             State::C => (State::C, self), // Don't transition.
    ///         }
    ///     }
    /// }
    /// 
    /// fn main() {
    ///     let mut automaton = SomeFamily::automaton_with_mode(State::A);
    ///     while *automaton != State::C {
    ///         let previous = Automaton::next_with_result(&mut automaton, |current_mode| current_mode.next());
    ///         if previous != *automaton {
    ///             println!("Switched from state {:?} to state {:?}.", previous, *automaton);
    ///         }
    ///         println!("Now in state {:?}.", *automaton);
    ///     }
    /// }
    /// ```
    /// 
    pub fn next_with_result<T, R>(automaton : &mut Self, transition_fn : T) -> R
        where T : FnOnce(F::Mode) -> (F::Mode, R)
    {
        let (next_mode, result) = transition_fn(
            automaton.mode.take().expect("Cannot swap out current Mode while another swap is taking place!"));
        automaton.mode = Some(next_mode);
        result
    }
}

impl<F> Automaton<F>
    where
        F : Family + ?Sized,
        F::Mode : Borrow<F::Base>,
{
    /// Returns an immutable reference to the current `Mode` as an `&F::Base`, allowing immutable functions to be called
    /// on the inner `Mode`.
    /// 
    /// **NOTE:** `Automaton` also implements `Deref<Target = F::Base>`, allowing all `Base` members to be accessed via
    /// a reference to the `Automaton`. Hence, you can usually leave the `borrow_mode()` out and simply treat the
    /// `Automaton` as if it were an object of type `Base`.
    /// 
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
    /// Returns a mutable reference to the current `Mode` as a `&mut F::Base`, allowing mutable functions to be called
    /// on the inner `Mode`.
    /// 
    /// **NOTE:** `Automaton` also implements `DerefMut<Target = Base>`, allowing all `Base` members to be accessed via
    /// a reference to the `Automaton`. Hence, you can usually leave the `borrow_mode_mut()` out and simply treat the
    /// `Automaton` as if it were an object of type `Base`.
    /// 
    pub fn borrow_mode_mut(&mut self) -> &mut F::Base {
        self.mode.as_mut()
            .expect("Cannot borrow current Mode because another swap is already taking place!")
            .borrow_mut()
    }
}

impl<F> AsRef<F::Base> for Automaton<F>
    where
        F : Family + ?Sized,
        F::Mode : Borrow<F::Base>,
{
    /// Returns an immutable reference to the current `Mode` as a `&F::Base`, allowing functions to be called on the
    /// inner `Mode`.
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
    /// Returns a mutable reference to the current `Mode` as a `&mut F::Base`, allowing functions to be called on the
    /// inner `Mode`.
    /// 
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

    /// Returns an immutable reference to the current `Mode` as a `&F::Base`, allowing functions to be called on the
    /// inner `Mode`.
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
    /// Returns a mutable reference to the current `Mode` as a `&mut F::Base`, allowing functions to be called on the
    /// inner `Mode`.
    /// 
    fn deref_mut(&mut self) -> &mut F::Base {
        self.borrow_mode_mut()
    }
}

impl<F> Automaton<F>
    where
        F : Family + ?Sized,
        F::Mode : Default,
{
    /// Creates a new `Automaton` with a default `Mode` instance as the active `Mode`.
    /// 
    /// **NOTE:** This only applies if `F::Base` is a **concrete** type that implements `Default`. If `F::Base` is a
    /// **trait** type, or you need to specify the initial mode of the created `Automaton`, use
    /// [`with_mode()`](struct.Automaton.html#method.with_mode) instead.
    /// 
    /// Since the `F` parameter cannot be determined automatically, using this function usually requires the use of the
    /// turbofish, e.g. `Automaton::<SomeFamily>::new()`. To avoid that, `Family` provides an `automaton()` associated
    /// function that can be used instead. See [`Family::automaton()`](trait.Family.html#method.automaton) for more
    /// details.
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
    /// impl ModeWithDefault {
    ///     fn update(mut self) -> Self {
    ///         // TODO: Logic for transitioning between states goes here.
    ///         self.count += 1;
    ///         self
    ///     }
    /// }
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
    /// // NOTE: We could alternatively use SomeFamily::automaton() here to shorten this.
    /// let mut automaton = Automaton::<SomeFamily>::new();
    /// 
    /// // NOTE: Deref coercion allows us to access the CounterMode's count variable through an Automaton reference.
    /// assert!(automaton.count == 0);
    /// 
    /// // Keep transitioning the current Mode out until we reach the target state
    /// // (i.e. a count of 10).
    /// while automaton.count < 10 {
    ///     Automaton::next(&mut automaton, |current_mode| current_mode.update());
    /// }
    /// ```
    /// 
    pub fn new() -> Self {
        Self {
            mode : Some(Default::default()),
        }
    }
}

impl<F> Default for Automaton<F>
    where
        F : Family + ?Sized,
        F::Mode : Default,
{
    /// Creates a new `Automaton` with the default `Mode` active. This is equivalent to calling `Automaton::new()`.
    /// 
    /// See note on [`new()`](struct.Automaton.html#method.new) for more on when this function can be used.
    /// 
    fn default() -> Self {
        Self::new()
    }
}

/// If `Base` implements `std::fmt::Debug`, `Automaton` also implements `Debug`, and will print its current `mode`.
/// 
/// # Usage
/// ```
/// use mode::*;
/// use std::fmt::Debug;
/// 
/// struct MyFamily;
/// impl Family for MyFamily {
///     type Base = dyn MyBase;
///     type Mode = Box<dyn MyBase>;
/// }
/// 
/// trait MyBase : Mode<Family = MyFamily> + Debug { } // TODO: Add common interface.
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
///     type Family = MyFamily;
/// }
/// 
/// let automaton = MyFamily::automaton_with_mode(Box::new(MyMode { foo: 3, bar: "Hello, World!" }));
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

/// If `Base` implements `std::fmt::Display`, `Automaton` also implements `Display`, and will print its current `mode`.
/// 
/// # Usage
/// ```
/// use mode::*;
/// use std::fmt::{Display, Formatter, Result};
/// 
/// struct MyFamily;
/// impl Family for MyFamily {
///     type Base = dyn MyBase;
///     type Mode = Box<dyn MyBase>;
/// }
/// 
/// trait MyBase : Mode<Family = MyFamily> + Display { } // TODO: Add common interface.
/// 
/// struct MyMode {
///     pub foo : i32,
///     pub bar : &'static str,
/// }
/// 
/// impl Display for MyMode {
///     fn fmt(&self, f : &mut Formatter<'_>) -> Result {
///         write!(f, "Foo is {}, and bar is \"{}\".", self.foo, self.bar)
///     }
/// }
/// 
/// impl MyBase for MyMode { } // TODO: Implement common interface.
/// 
/// impl Mode for MyMode {
///     type Family = MyFamily;
/// }
/// 
/// let automaton = MyFamily::automaton_with_mode(Box::new(MyMode { foo: 3, bar: "Hello, World!" }));
/// println!("{}", automaton);
/// ```
/// 
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