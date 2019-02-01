use crate::{AnyModeWrapper, Mode, ModeWrapper};

/// Represents a state machine over a set of `Mode`s that implement some common interface `Base`.
/// 
/// The `Automaton` contains a single, active `Mode` that represents the current state of the state machine. The current
/// `Mode` is accessible via the `borrow_mode()` and `borrow_mode_mut()` functions, which return a `Base` reference. The
/// `Automaton` also provides a `perform_transitions()` function that should be called at some point in order to allow
/// it to transition to another active `Mode`, if desired.
/// 
/// See the [`Mode`](trait.Mode.html) trait and [`get_transition()`](trait.Mode.html#tymethod.get_transition) for more
/// details.
/// 
/// # The `Base` parameter
/// 
/// The `Base` parameter may be either a `trait` (e.g. `Automaton<dyn SomeTraitThatExtendsMode>`) or a concrete type
/// (e.g. `Automaton<SomeStructThatImplsMode>`). Given a `trait`, the `Automaton` will be able to swap between **any**
/// `Mode`s that implement the trait. However, this means that the `Automaton` will **only** allow the inner `Mode` to
/// be borrowed via a trait reference, implying that **only** functions defined on the trait will be callable.
/// 
/// By contrast, if given a `struct`, **all** functions defined on the inner type will be accessible from outside the
/// `Automaton`. However, this also implies that the `Automaton` will **only** be able to switch between states of the
/// same concrete type.
/// 
/// # Usage
/// ```
/// use mode::*;
/// 
/// trait MyBase {
///     fn some_fn(&mut self);
/// }
/// 
/// struct MyMode; // TODO
/// 
/// impl MyBase for MyMode {
///     fn some_fn(&mut self) { } // TODO
/// }
/// 
/// impl Mode for MyMode {
///     type Base = MyBase;
///     fn as_base(&self) -> &Self::Base { self }
///     fn as_base_mut(&mut self) -> &mut Self::Base { self }
///     fn get_transition(&mut self) -> Option<Box<TransitionFrom<Self>>> { None } // TODO
/// }
/// 
/// // TODO: Add more Modes that implement MyBase.
/// 
/// fn main() {
///     let mut automaton = Automaton::with_initial_mode(MyMode);
/// 
///     loop {
///         // Call some update method on the Mode.
///         automaton.borrow_mode_mut().some_fn();
/// 
///         // Allow the Automaton to switch Modes, if desired.
///         automaton.perform_transitions();
///         # break;
///     }
/// }
/// ```
/// 
pub struct Automaton<Base>
    where Base : ?Sized
{
    current_mode : Box<AnyModeWrapper<Base = Base>>,
}

impl<Base> Automaton<Base>
    where Base : ?Sized
{
    /// Creates a new `Automaton` with the specified `initial_mode`, which will be the active `Mode` for the `Automaton`
    /// that is returned.
    /// 
    pub fn with_initial_mode<M>(initial_mode : M) -> Self
        where M : Mode<Base = Base>
    {
        Self {
            current_mode : Box::new(ModeWrapper::new(initial_mode)),
        }
    }

    /// Calls `get_transition()` on the current `Mode` to determine whether it wants to transition out. If a
    /// `Transition` is returned, the `Transition` callback will be called on the current `Mode`, swapping in whichever
    /// `Mode` it returns as a result.
    /// 
    /// See [`Transition`](struct.Transition.html) for more details.
    /// 
    pub fn perform_transitions(&mut self) {
        if let Some(mode) = self.current_mode.perform_transitions() {
            // If a transition was performed and a new `ModeWrapper` was returned, swap in the new `Mode`.
            self.current_mode = mode;
        }
    }

    /// Returns an immutable reference to the current `Mode` as a `&Self::Base`, allowing immutable functions to be
    /// called on the inner `Mode`.
    /// 
    pub fn borrow_mode(&self) -> &Base {
        self.current_mode.borrow_mode()
    }

    /// Returns a mutable reference to the current `Mode` as a `&mut Self::Base`, allowing mutable functions to be
    /// called on the inner `Mode`.
    /// 
    pub fn borrow_mode_mut(&mut self) -> &mut Base {
        self.current_mode.borrow_mode_mut()
    }
}

impl<Base> Automaton<Base>
    where Base : Mode<Base = Base> + Default
{
    /// Creates a new `Automaton` with the default `Mode` active.
    /// 
    /// **NOTE:** This only applies if `Base` is a **concrete** type (e.g. `Automaton<SomeStructThatImplsMode>`) that
    /// implements `Default`. If `Base` is a **trait** type (e.g. `Automaton<dyn SomeTraitThatExtendsMode>`) or you
    /// would otherwise like to specify the initial mode of the created `Automaton`, use
    /// [`with_initial_mode()`](struct.Automaton.html#method.with_initial_mode) instead.
    /// 
    pub fn new() -> Self {
        Self {
            current_mode : Box::new(ModeWrapper::<Base>::new(Default::default())),
        }
    }
}

impl<Base> Default for Automaton<Base>
    where Base : Mode<Base = Base> + Default
{
    /// Creates a new `Automaton` with the default `Mode` active. This is equivalent to calling `Automaton::new()`.
    /// 
    /// See note on `new()` for when this can be used.
    /// 
    fn default() -> Self {
        Self::new()
    }
}