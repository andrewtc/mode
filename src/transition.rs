// Copyright 2019 Andrew Thomas Christensen
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the
// MIT license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option. This file may not be copied,
// modified, or distributed except according to those terms.

use crate::{AnyModeWrapper, Mode, ModeWrapper};

/// Trait that defines the call signature for a function that can switch an `Automaton` from one `Mode` (`A`) to
/// another.
/// 
/// State machines, by definition, are systems that possess multiple states that can be switched in or out in order to
/// change the behavior of the system. `Automaton`s are no exception. Eventually, the `Automaton` will need to switch
/// `Mode`s in order to change its behavior.
/// 
/// Rather than requiring the owner of the `Automaton` to check the status of the current `Mode` periodically and swap
/// in a new state from outside the object when appropriate, the `Automaton` instead delegates this responsibility to
/// the current `Mode` via the `perform_transitions()` function. Each time this function is called, the `Automaton`
/// calls `get_transition()` on the current `Mode` to determine whether it is ready to transition to another state. To
/// do this, the `Mode` may return a callback function of the form `FnOnce(A) -> B`, where `A` is the type of the
/// currently active `Mode` and `B` is another implementation of `Mode` with the same `Base` type. The `Transition`
/// trait is automatically implemented on any closure of this form, so that `get_transitions()` can return a boxed
/// closure instead of some wrapper type.
/// 
/// **NOTE:** Although there is rarely a need to do this, it is perfectly valid for a `Transition` function to return
/// the input `Mode` as a result. This will result in the currently active `Mode` remaining active, even after the
/// `Transition` function has been called.
/// 
/// # Usage
/// ```
/// use mode::*;
/// 
/// # trait MyMode { }
/// #
/// # struct SomeMode;
/// # impl MyMode for SomeMode { }
/// # 
/// impl Mode for SomeMode {
/// #   type Base = MyMode;
/// #   fn as_base(&self) -> &Self::Base { self }
/// #   fn as_base_mut(&mut self) -> &mut Self::Base { self }
///     // ...
///     fn get_transition(&mut self) -> Option<Box<dyn Transition<Self>>> {
/// #       let some_condition = true;
/// #       let some_other_condition = true;
///         // ...
///         if some_condition {
///             // Returning a Transition function will cause the Automaton to switch the
///             // current Mode to whatever new Mode is produced by the callback.
///             Some(Box::new(|previous : Self| { SomeOtherMode }))
///         }
///         else if some_other_condition {
///             // NOTE: The Transition trait allows this function to return closures with
///             // completely different return types, so long as the parameter types match.
///             Some(Box::new(|previous : Self| { YetAnotherMode }))
///         }
///         else { None } // Returning None will keep the current Mode active.
///     }
/// }
/// #
/// # struct SomeOtherMode;
/// # impl MyMode for SomeOtherMode { }
/// #
/// # impl Mode for SomeOtherMode {
/// #     type Base = MyMode;
/// #     fn as_base(&self) -> &Self::Base { self }
/// #     fn as_base_mut(&mut self) -> &mut Self::Base { self }
/// #     fn get_transition(&mut self) -> Option<Box<dyn Transition<Self>>> { None }
/// # }
/// #
/// # struct YetAnotherMode;
/// # impl MyMode for YetAnotherMode { }
/// #
/// # impl Mode for YetAnotherMode {
/// #     type Base = MyMode;
/// #     fn as_base(&self) -> &Self::Base { self }
/// #     fn as_base_mut(&mut self) -> &mut Self::Base { self }
/// #     fn get_transition(&mut self) -> Option<Box<dyn Transition<Self>>> { None }
/// # }
/// ```
/// 
/// # Why `Transition` functions?
/// One of the most powerful features of the `Transition` system is that, while transitioning, the current `Mode` is
/// moved into the `Transition` callback. This allows `Mode` `B`, being produced, to steal pointers and other state from
/// `Mode` `A`, as opposed to allocating more memory for `B`, copying over state from `A`, and then deallocating `A`.
/// This won't make much of a difference for small `Mode`s, but for `Mode`s that contain a large amount of state (on the
/// order of several megabytes or even gigabytes, as is common in UI applications), this can make a huge difference for
/// both performance and heap memory usage.
/// 
/// Instead of having the `get_transition()` function return a callback, the transition system could have been
/// implemented as a `fn do_transition(self) -> B` function on `Mode`. (This is a little bit of an oversimplification.)
/// However, having a callback allows `Transition`s to be scheduled in advance and performed later, capturing state from
/// the calling code, as necessary. This is especially convenient in cases where a `Mode` needs to stay active for a
/// little longer after a `Transition` is scheduled, e.g. allowing a `Mode`-driven UI system to finish playing a
/// transition animation or sound effect before swapping in a new `Mode`.
/// 
/// ## Example
/// ```
/// use mode::*;
/// 
/// # trait MyMode {
/// #     fn update(&mut self) { }
/// # }
/// #
/// struct SomeMode {
///     queued_transition : Option<Box<dyn Transition<Self>>>
/// }
/// 
/// impl MyMode for SomeMode {
///     fn update(&mut self) {
/// #       let some_condition = true;
///         // ...
///         if (some_condition) {
///             // Queue up a transition to be performed later.
///             self.queued_transition = Some(Box::new(|previous| { SomeOtherMode }))
///         }
/// 
///         // TODO: Continue updating, animating, etc.
///     }
/// }
/// 
/// impl Mode for SomeMode {
/// #   type Base = MyMode;
/// #   fn as_base(&self) -> &Self::Base { self }
/// #   fn as_base_mut(&mut self) -> &mut Self::Base { self }
///     // ...
///     fn get_transition(&mut self) -> Option<Box<dyn Transition<Self>>> {
/// #       let ready_to_transition = true;
///         // ...
///         if ready_to_transition && self.queued_transition.is_some() {
///             // When we're finally finished updating, return the queued transition.
///             self.queued_transition.take()
///         }
///         else { None } // Returning None will keep the current Mode active.
///     }
/// }
/// #
/// # struct SomeOtherMode;
/// # impl MyMode for SomeOtherMode { }
/// #
/// # impl Mode for SomeOtherMode {
/// #     type Base = MyMode;
/// #     fn as_base(&self) -> &Self::Base { self }
/// #     fn as_base_mut(&mut self) -> &mut Self::Base { self }
/// #     fn get_transition(&mut self) -> Option<Box<dyn Transition<Self>>> { None }
/// # }
/// ```
/// 
pub trait Transition<A>
    where A : Mode + ?Sized
{
    /// Calls the `Transition` function on the specified `Mode`, consuming the `Transition` and the `mode` and returning
    /// a wrapper around the new `Mode` to be swapped in as active.
    /// 
    /// **NOTE:** You should not attempt to implement this function yourself, as the return type makes use of a private
    /// trait (`AnyModeWrapper`). Ideally, this function would return a `Box<Mode>` in order to abstract away all of the
    /// implementation details of the `Automaton`. Unfortunately, that isn't possible, in this case, because the `Mode`
    /// trait cannot be made into an object, and therefore cannot be boxed.
    /// 
    fn invoke(self : Box<Self>, mode : A) -> Box<dyn AnyModeWrapper<Base = A::Base>>;
}

impl<T, A, B> Transition<A> for T
    where
        T : FnOnce(A) -> B,
        A : Mode,
        B : Mode<Base = A::Base>,
{
    fn invoke(self : Box<Self>, mode : A) -> Box<dyn AnyModeWrapper<Base = A::Base>> {
        // Call the transition function and wrap the result with a ModeWrapper.
        Box::new(ModeWrapper::<B>::new((self)(mode)))
    }
}