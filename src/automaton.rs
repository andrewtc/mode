// Copyright 2019 Andrew Thomas Christensen
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the
// MIT license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option. This file may not be copied,
// modified, or distributed except according to those terms.

use crate::{Family, Mode};
use std::{
    convert::{AsRef, AsMut},
    borrow::{Borrow, BorrowMut},
    fmt,
};
use std::ops::{Deref, DerefMut};

/// Represents a state machine over a set of `Mode`s within the same `Family`.
/// 
/// The `Automaton` contains a single, active `Mode` that represents the current state of the state machine. The current
/// `Mode` is accessible via `borrow_base()` and `borrow_base_mut()` functions, which return a `Base` reference, or via
/// `Deref` coercion. The `Automaton` provides a `next()` function that should be called regularly in order to allow the
/// current state to swap in another `Mode` as active, if desired.
/// 
/// See [`Mode::swap()`](trait.Mode.html#tymethod.swap) for more details.
/// 
/// # Usage
/// ```
/// use mode::*;
/// #
/// # struct SomeFamily;
/// # impl Family for SomeFamily {
/// #     type Base = dyn MyBase;
/// #     type Mode = Box<dyn MyBase>;
/// #     type Output = Box<dyn MyBase>;
/// # }
/// #
/// # trait MyBase : boxed::Mode<Family = SomeFamily> {
/// #     fn some_fn(&self);
/// #     fn some_mut_fn(&mut self);
/// # }
/// # 
/// # struct SomeMode;
/// # impl MyBase for SomeMode {
/// #     fn some_fn(&self) { println!("some_fn was called"); }
/// #     fn some_mut_fn(&mut self) { println!("some_mut_fn was called"); }
/// # }
/// # 
/// # impl boxed::Mode for SomeMode {
/// #     type Family = SomeFamily;
/// #     fn swap(self : Box<Self>) -> Box<dyn MyBase> { self }
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
/// // Let the Automaton handle transitions.
/// Automaton::next(&mut automaton);
/// ```
/// 
/// # The `F` parameter
/// 
/// One important thing to note about the `F` generic parameter it that it is **not** the base `Mode` type that will be
/// stored in the `Automaton`, itself. Rather, it is a separate `struct` that represents the group of all `Mode` types
/// that are compatible with the `Automaton`. For example, an `Automaton<SomeFamily>` will **only** be able to operate
/// on types that implement `Mode<Family = SomeFamily>`.
/// 
/// # `F::Mode`, `F::Base`, and pointer types
/// 
/// Another important thing to understand is that the actual type stored in the `Automaton` will be `F::Mode`, **not**
/// `F::Base`. This has to be the case because if `F::Base` is an unsized type, e.g. a `dyn Trait`, then `F::Mode` is
/// **required** be a `Sized` pointer type, e.g. a `Box` or an `Rc`. When this is the case, the `Automaton` will
/// actually call `Mode::swap()` on the **pointer** type wrapping the stored type. There are several blanket `impl`s
/// for various pointer types defined in the `mode` submodule that then delegate the responsibility of switching the
/// current `Mode` to some other interface, e.g. `impl<F> Mode for Box<boxed::Mode<Family = F>>`. Please note that
/// `boxed::Mode` is a completely **different** `trait` than `Mode` with a `swap()` method that operates on
/// `self : Box<Self>` instead of just `self`.
/// 
/// The reason for this is that when `F::Mode` is a pointer type, the inner `Mode` may be a very large object that would
/// be slow to move into and out of the `Mode::swap()` function. Since the convention for keeping the `Automaton` in the
/// same state is to return `self` from `Mode::swap()`, moving the inner `Mode` out of a pointer and back into another
/// one might result in two needless (and potentially computationally expensive) copy operations in order to move the
/// current state into the function and then back out into the `Automaton` again, even if the current `Mode` remained
/// current after the `swap()`. (See example below.)
/// 
/// ```
/// use mode::{Family, Mode};
/// 
/// struct ReallyBigFamily;
/// impl Family for ReallyBigFamily {
///     type Base = ReallyBigMode;
///     type Mode = ReallyBigMode;
///     type Output = ReallyBigMode;
/// }
/// 
/// const DATA_SIZE : usize = 1024 * 1024; // 1 MiB
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
///     fn swap(self) -> Self {
///         // This is silly, since we will never swap to another Mode in this scenario. However, even if we were fine
///         // never making another Mode current like this, each call to swap() would still (potentially) move 1 MiB of
///         // data into the function and then right back out! That's not very efficient, to say the least.
///         self
///     }
/// }
/// ```
/// 
/// Having a separate `swap()` interface that operates on the pointer type itself, e.g.
/// `fn swap(self : Box<Self>) -> <Self::Family as Family>::Output` in `boxed::Mode`, allows the **pointer** itself to
/// be moved in and out of the `swap()` function, while still delegating the responsibility of swapping states to the
/// stored type itself. (See example below.)
/// 
/// ```
/// use mode::{boxed, Family};
/// 
/// struct ReallyBigFamily;
/// impl Family for ReallyBigFamily {
///     type Base = ReallyBigMode;
///     type Mode = Box<ReallyBigMode>;
///     type Output = Box<ReallyBigMode>;
/// }
/// 
/// const DATA_SIZE : usize = 1024 * 1024; // 1 MiB
/// 
/// struct ReallyBigMode {
///     data : [u8; DATA_SIZE],
/// }
/// 
/// impl Default for ReallyBigMode {
///     fn default() -> Self { Self { data : [0; DATA_SIZE] } }
/// }
/// 
/// impl boxed::Mode for ReallyBigMode {
///     type Family = ReallyBigFamily;
///     fn swap(self : Box<Self>) -> Box<Self> {
///         // This moves the Box back out of the function, not the object itself, which is *much* cheaper!
///         self
///     }
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
    ///     type Output = SomeMode;
    /// }
    /// 
    /// enum SomeMode { A, B, C };
    /// impl Mode for SomeMode {
    ///     type Family = SomeFamily;
    ///     fn swap(mut self) -> Self {
    ///         // TODO: Logic for transitioning between states goes here.
    ///         self
    ///     }
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

impl<F, M> Automaton<F>
    where
        F : Family<Mode = M, Output = M> + ?Sized,
        M : Mode<Family = F>,
{
    /// Calls `swap()` on the current `Mode` to determine whether it wants to transition out, swapping in whatever
    /// `Mode` it returns as a result. Calling this function *may* change the current `Mode`, but not necessarily.
    /// 
    /// See [`Mode::swap()`](trait.Mode.html#tymethod.swap) for more details.
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
    /// For `Mode` implementations that return a tuple with a `Mode` and some other parameter, calls `swap()` on the
    /// current `Mode` to determine whether it wants to transition out. Whatever `Mode` was returned as the first tuple
    /// parameter will be switched in as active, and the second tuple parameter will be returned from the function.
    /// Calling this function *may* change the current `Mode`, but not necessarily.
    /// 
    /// See [`Mode::swap()`](trait.Mode.html#tymethod.swap) for more details.
    /// 
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
    /// #     type Output = ModeWithDefault;
    /// # }
    /// 
    /// struct ModeWithDefault { count : u32 };
    /// 
    /// impl Mode for ModeWithDefault {
    ///     type Family = SomeFamily;
    ///     fn swap(mut self) -> ModeWithDefault {
    ///         // TODO: Logic for transitioning between states goes here.
    ///         self.count += 1;
    ///         self
    ///     }
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
    ///     Automaton::next(&mut automaton);
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
///     type Output = Box<dyn MyBase>;
/// }
/// 
/// trait MyBase : boxed::Mode<Family = MyFamily> + Debug { } // TODO: Add common interface.
/// 
/// #[derive(Debug)]
/// struct MyMode {
///     pub foo : i32,
///     pub bar : &'static str,
/// }
/// 
/// impl MyBase for MyMode { } // TODO: Implement common interface.
/// 
/// impl boxed::Mode for MyMode {
///     type Family = MyFamily;
///     fn swap(self : Box<Self>) -> Box<dyn MyBase> { self } // TODO
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
///     type Output = Box<dyn MyBase>;
/// }
/// 
/// trait MyBase : boxed::Mode<Family = MyFamily> + Display { } // TODO: Add common interface.
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
/// impl boxed::Mode for MyMode {
///     type Family = MyFamily;
///     fn swap(self : Box<Self>) -> Box<dyn MyBase> { self } // TODO
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