use crate::Transition;

/// Trait that represents a state within an `Automaton`.
/// 
/// Every `Automaton` contains a single `Mode` instance that represents the active state of the state machine. An
/// `Automaton<Base>` can **only** switch between implementations of `Mode` of the same `Base` type. The `Automaton`
/// only allows its active `Mode` to be accessed as a `Base` reference, so only functions exposed on the `Base` type are
/// callable on the `Mode` from outside the `Automaton`.
/// 
/// See [`Automaton`](struct.Automaton.html) for more details.
/// 
/// # Transitions
/// `Mode`s can choose to transition to any other `Mode` with the same `Base` type. This is accomplished by returning a
/// `Transition` function from the `get_transition()` function, which will cause the parent `Automaton` to switch to
/// another `Mode` the next time `perform_transitions()` is called.
/// 
/// See [`Transition`](trait.Transition.html) and
/// [`Automaton::perform_transitions()`](struct.Automaton.html#method.perform_transitions) for more details.
/// 
/// # Usage
/// ```
/// use mode::*;
/// 
/// trait MyMode {
///     // TODO: Define some common interface for ModeA and ModeB.
/// }
/// 
/// struct ModeA; // TODO: Add fields.
/// impl MyMode for ModeA { }
/// 
/// impl Mode for ModeA {
///     type Base = MyMode;
///     fn as_base(&self) -> &Self::Base { self }
///     fn as_base_mut(&mut self) -> &mut Self::Base { self }
///     fn get_transition(&mut self) -> Option<Box<Transition<Self>>> {
///         // Transition to ModeB. ModeA can swap to ModeB because both share the same Base.
///         Some(Box::new(|previous : Self| { ModeB }))
///     }
/// }
/// 
/// struct ModeB; // TODO: Add fields.
/// impl MyMode for ModeB { }
/// 
/// impl Mode for ModeB {
///     type Base = MyMode;
///     fn as_base(&self) -> &Self::Base { self }
///     fn as_base_mut(&mut self) -> &mut Self::Base { self }
///     fn get_transition(&mut self) -> Option<Box<Transition<Self>>> { None } // None means don't transition.
/// }
/// ```
/// 
pub trait Mode : 'static {
    /// Represents the user-facing interface for the `Mode` that will be exposed via the `Automaton`. In order to be
    /// used with an `Automaton`, the `Base` type of the `Mode` **must** match the `Base` type of the `Automaton`. This
    /// is so that the `Automaton` can provide `get_mode()` and `get_mode_mut()` functions that return a reference to
    /// the `Mode` as the `Base` type.
    /// 
    type Base : ?Sized;

    /// Returns an immutable reference to this `Mode` as a `&Self::Base`.
    /// 
    fn as_base(&self) -> &Self::Base;

    /// Returns a mutable reference to this `Mode` as a `&mut Self::Base`.
    /// 
    fn as_base_mut(&mut self) -> &mut Self::Base;

    /// Every time `perform_transitions()` is called on an `Automaton`, This function will be called on the current
    /// `Mode` to determine whether it wants another `Mode` to become active. If this function returns `None`, the
    /// current `Mode` will remain active. If it returns a valid `Transition` function, however, the `Automaton` will
    /// call the function on the active `Mode`, consuming it and swapping in whichever `Mode` is produced as a result.
    /// 
    /// See [`Transition`](trait.Transition.html) for more details.
    /// 
    fn get_transition(&mut self) -> Option<Box<Transition<Self>>>;
}