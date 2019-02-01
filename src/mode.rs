use crate::TransitionFrom;

/// Represents a state within an `Automaton`.
/// 
/// All `Mode`s have an associated type called `Base` that represents the common interface of a certain subset of
/// `Mode`s. `Mode`s can **only** be used with an `Automaton` that has the same `Base` type, i.e. an `Automaton<Base>`.
/// 
/// # Transitions
/// `Mode`s can choose to transition to any other `Mode` with the same `Base` type. This is accomplished by returning a
/// `Transition` from the `get_transition_mut()` function, which will cause the parent `Automaton` to switch to another
/// `Mode` the next time `perform_transitions()` is called.
/// 
/// See [`Transition`](struct.Transition.html) for more details and
/// [`Automaton::perform_transitions()`](struct.Automaton.html#method.perform_transitions) for more details.
/// 
pub trait Mode : 'static {
    /// Represents the user-facing interface for the `Mode` that will be exposed via the `Automaton`. The `as_base()`
    /// and `as_base_mut()` functions of this trait return a `Self::Base` reference to the `Mode`, so that the parent
    /// `Automaton` can allow access to a common subset of functions on the object.
    /// 
    type Base : ?Sized;

    /// Returns an immutable reference to this `Mode` as a `&Self::Base`.
    /// 
    fn as_base(&self) -> &Self::Base;

    /// Returns a mutable reference to this `Mode` as a `&mut Self::Base`.
    /// 
    fn as_base_mut(&mut self) -> &mut Self::Base;

    /// Every time `perform_transitions()` is called on an `Automaton`, This function will
    /// be called on the current `Mode` to determine whether it wants another `Mode` to
    /// become active. If this function returns `None`, the current `Mode` will remain
    /// active. If it returns a valid `Transition`, however, the `Automaton` will call
    /// the `Transition` callback on the current `Mode`, consuming it and swapping in
    /// whatever `Mode` is produced as a result.
    /// 
    /// See [`Transition`](struct.Transition.html) for more details.
    /// 
    fn get_transition(&mut self) -> Option<Box<dyn TransitionFrom<Self>>>;
}