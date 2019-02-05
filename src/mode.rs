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
/// # The `Base` parameter
/// You will notice from the [example](#usage) above that `ModeA` and `ModeB` implement `Mode` and `MyMode` separately,
/// but the `MyMode` trait itself does **not** extend `Mode`, i.e. is defined as `trait MyMode` as opposed to
/// `trait MyMode : Mode<Base = MyMode>`. We want to use `MyMode` as the `Base` type for `ModeA` and `ModeB`, but
/// unfortunately having `MyMode` extend `Mode<Base = MyMode>` would create a circular dependency between the two types,
/// and would cause a compile error. Hence, while it is possible to cast `ModeA` or `ModeB` to `MyMode` or `Mode`,
/// casting between `MyMode` and `Mode` is not allowed.
/// 
/// # `as_base()` and `as_base_mut()`
/// As mentioned above, a `Mode` reference **cannot** be cast to its `Base` type. What's more, the `Automaton` can only
/// require that the current `Mode` implements `Mode<Base = Base>`, and **cannot** enforce that it is also convertible
/// to `Base`. That's because `Base` is a type parameter and not a trait, and therefore **cannot** be used as a
/// constraint in `where` clauses, e.g. `where T : Base`.
/// 
/// Since the `Automaton` needs to be able to return a `Base` reference to the current `Mode`, each `Mode` is required
/// to implement `as_base()` and `as_base_mut()` functions that return `self` as a `&Base` and a `&mut Base`,
/// respectively. Unfortunately, these functions have to be defined manually for each struct that implements `Mode`.
/// 
/// **NOTE:** This may change when [`Unsize`](https://github.com/rust-lang/rfcs/blob/master/text/0982-dst-coercion.md)
/// becomes stable, since it will provide a way for `struct`s to express that a generic parameter must be convertible to
/// a particular type.
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