use crate::Mode;

/// Defines the `Automaton`-facing interface for a `ModeWrapper`.
/// 
pub trait AnyModeWrapper {
    /// Represents the user-facing interface for the `Mode` that will be exposed via the `Automaton`. See `Mode::Base`
    /// for more details.
    /// 
    type Base : ?Sized;

    /// Returns an immutable reference to the inner `Mode` as a `&Self::Base`.
    /// 
    fn borrow_mode(&self) -> &Self::Base;

    /// Returns a mutable reference to the inner `Mode` as a `&mut Self::Base`.
    /// 
    fn borrow_mode_mut(&mut self) -> &mut Self::Base;

    /// Calls `get_transition()` on the inner `Mode` to determine whether it wants another `Mode` to become active. If
    /// this yields a `Transition`, the `Transition` will be called on the inner `Mode` and a new `ModeWrapper` around
    /// the `Mode` to be swapped in will be returned.
    /// 
    fn perform_transitions(&mut self) -> Option<Box<AnyModeWrapper<Base = Self::Base>>>;
}

/// Wraps a specific instance of `Mode`, allowing the parent `Automaton` to handle `Transition`s between that instance
/// and other `Mode`s gracefully.
/// 
/// **NOTE:** This `struct` mainly exists to allow `Transition`s to be scheduled as `FnOnce(A) -> B` instead of
/// requiring each user-defined `Mode` to know more about the implementation details of the `Automaton`.
/// 
pub struct ModeWrapper<T>
    where T : Mode
{
    mode : Option<T>,
}

impl<T> ModeWrapper<T>
    where T : Mode
{
    /// Creates and returns a new `ModeWrapper` around the specified `Mode`.
    /// 
    pub fn new(mode : T) -> Self {
        Self {
            mode: Some(mode),
        }
    }
}

impl<T> AnyModeWrapper for ModeWrapper<T>
    where T : Mode
{
    type Base = T::Base;

    fn borrow_mode(&self) -> &Self::Base {
        self.mode.as_ref().unwrap().as_base()
    }

    fn borrow_mode_mut(&mut self) -> &mut Self::Base {
        self.mode.as_mut().unwrap().as_base_mut()
    }

    fn perform_transitions(&mut self) -> Option<Box<AnyModeWrapper<Base = Self::Base>>> {
        // Retrieve the desired transition, if any, from the inner Mode.
        match self.mode.as_mut().unwrap().get_transition() {
            None => None,
            Some(mut transition) => {
                // If a valid Transition was returned, call the Transition callback on the inner Mode and return a new
                // wrapper around the Mode that was produced.
                // NOTE: This will move the Mode into the callback, leaving this object empty.
                transition.try_invoke(self.mode.take().unwrap())
            }
        }
    }
}