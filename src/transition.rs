use crate::{AnyModeWrapper, Mode, ModeWrapper};
use std::marker::PhantomData;

/// Trait that defines the call signature for a function that can switch an `Automaton` from one `Mode` (`A`) to
/// another.
/// 
/// See [`Transition`](struct.Transition.html) for more details.
/// 
pub trait TransitionFrom<A>
    where A : Mode
{
    /// Calls the inner transition callback on the specified `Mode`, if it hasn't been called already, consuming `mode`
    /// and returning a new `ModeWrapper` wrapping the `Mode` to be swapped in. If the callback has been called once
    /// already, returns `None`.
    /// 
    /// **NOTE:** Ideally, this would return a raw `A::Base` object and not a boxed `AnyModeWrapper`, but this isn't
    /// possible in all situations because `A::Base` is not necessarily a sized type. This may change in the future when
    /// [unsized rvalues](https://github.com/rust-lang/rust/issues/48055) are stabilized.
    /// 
    fn try_invoke(&mut self, mode : A) -> Option<Box<AnyModeWrapper<Base = A::Base>>>;
}

/// Represents a function that can be called to switch an `Automaton` from one `Mode` (`A`) to another (`B`).
/// 
/// `Transition`s are functions that can only be called once, consuming the current `Mode` for an `Automaton` and
/// allowing it to pass large amounts of state directly to the new `Mode` to be switched in, if desired. `Transition`s
/// can be created using the `new()` and `boxed()` functions on any closure of the form `FnOnce(A) -> B`. Having the
/// inner closure consume the current `Mode` has the nice property of allowing `B` to steal pointers from `A` before
/// `A` is dropped, which can help reduce the spike in memory usage that would otherwise occur as a result of switching
/// between two `Mode`s that allocate large amounts of memory on the heap.
/// 
/// Another advantage of having a closure swap between `Mode`s is that closures created inside of `A` have full access
/// to `A`'s private functions, private data, and all state declared within the function that defined the callback.
/// This offers a lot of flexibility by allowing state from any one of these sources to be moved into the `Transition`
/// callback that is to be called later.
/// 
pub struct Transition<A, B, C>
    where
        A : Mode,
        B : Mode<Base = A::Base>,
        C : FnOnce(A) -> B
{
    phantom_a : PhantomData<A>,
    phantom_b : PhantomData<B>,
    callback : Option<C>,
}

impl<A, B, C> Transition<A, B, C>
    where
        A : Mode,
        B : Mode<Base = A::Base>,
        C : FnOnce(A) -> B
{
    /// Creates and returns a new `Transition` from a transition callback.
    /// 
    pub fn new(callback : C) -> Self {
        Self {
            phantom_a: Default::default(),
            phantom_b: Default::default(),
            callback: Some(callback),
        }
    }

    /// Creates and returns a new, boxed `Transition` from a transition callback.
    /// 
    pub fn boxed(callback : C) -> Box<Self> {
        Box::new(Self::new(callback))
    }
}

impl<A, B, C> TransitionFrom<A> for Transition<A, B, C>
    where
        A : Mode,
        B : Mode<Base = A::Base>,
        C : FnOnce(A) -> B
{
    fn try_invoke(&mut self, mode : A) -> Option<Box<AnyModeWrapper<Base = A::Base>>> {
        match self.callback.take() {
            None => None,
            Some(callback) => {
                // If the callback hasn't been called already, call it, passing in the Mode to be consumed and returning
                // a new ModeWrapper wrapping the Mode that was returned.
                Some(Box::new(ModeWrapper::new((callback)(mode))))
            }
        }
    }
}

impl<A, B, C> From<C> for Transition<A, B, C>
    where
        A : Mode,
        B : Mode<Base = A::Base>,
        C : FnOnce(A) -> B
{
    /// Converts a transition callback into a `Transition`.
    /// 
    fn from(callback : C) -> Self {
        Self::new(callback)
    }
}