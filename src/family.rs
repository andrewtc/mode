use crate::Swap;

pub trait Family {
    /// Represents the user-facing interface for the `Mode` that will be exposed via the `Automaton`. In order to be
    /// used with an `Automaton`, the `Base` type of the `Mode` **must** match the `Base` type of the `Automaton`. This
    /// is so that the `Automaton` can provide `borrow_mode()` and `borrow_mode_mut()` functions that return a reference
    /// to the `Mode` as the `Base` type.
    /// 
    type Base : ?Sized;

    type Mode : Swap<Family = Self>;

    type Output;
}