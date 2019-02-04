extern crate mode;

use mode::*;

/// Defines the public interface of all `Mode`s below.
trait CounterMode {
    /// Tells the `CounterMode` to update once.
    fn update(&mut self);

    /// Returns an `i32` if the program is finished and a final result has been returned.
    fn get_result(&self) -> Option<i32> { None }
}

/// `CounterMode` that increments a counter value until it reaches the target value.
struct UpMode {
    pub counter : i32,
    pub target : i32,
}

impl CounterMode for UpMode {
    fn update(&mut self) {
        // Increment the counter until it reaches the target value.
        self.counter += 1;
        println!("+ {}", self.counter);
    }
}

impl Mode for UpMode {
    type Base = CounterMode;
    fn as_base(&self) -> &Self::Base { self }
    fn as_base_mut(&mut self) -> &mut Self::Base { self }
    fn get_transition(&mut self) -> Option<Box<Transition<Self>>> {
        if self.counter == self.target {
            // If we've reached the target value, start counting down to (roughly) the median value.
            Some(Box::new(
                |previous : Self| {
                    DownMode {
                        counter: previous.counter,
                        target: (previous.counter / 2) + 1,
                    }
                }))
        }
        else { None }
    }
}

/// `CounterMode` that decrements a counter value until it reaches the target value.
struct DownMode {
    pub counter : i32,
    pub target : i32,
}

impl CounterMode for DownMode {
    fn update(&mut self) {
        // Decrement the counter until it reaches the target value.
        self.counter -= 1;
        println!("- {}", self.counter);
    }
}

impl Mode for DownMode {
    type Base = CounterMode;
    fn as_base(&self) -> &Self::Base { self }
    fn as_base_mut(&mut self) -> &mut Self::Base { self }
    fn get_transition(&mut self) -> Option<Box<Transition<Self>>> {
        const GOAL : i32 = 10;
        if self.counter == GOAL {
            // When we finally count down to the goal value, end the program by swapping in a "finished" state.
            Some(Box::new(
                |previous : Self| {
                    FinishedMode {
                        result: previous.counter,
                    }
                }))
        }
        else if self.counter == self.target {
            // If we've reached the target value, start counting up to double the counter value.
            Some(Box::new(
                |previous : Self| {
                    UpMode {
                        counter: previous.counter,
                        target: previous.counter * 2,
                    }
                }))
        }
        else { None }
    }
}

/// Represents that we've finished counting and have a final result.
struct FinishedMode {
    result : i32,
}

impl CounterMode for FinishedMode {
    fn update(&mut self) { } // We're finished. Do nothing.
    fn get_result(&self) -> Option<i32> { Some(self.result) }
}

impl Mode for FinishedMode {
    type Base = CounterMode;
    fn as_base(&self) -> &Self::Base { self }
    fn as_base_mut(&mut self) -> &mut Self::Base { self }
    fn get_transition(&mut self) -> Option<Box<Transition<Self>>> {
        // We're finished calculating, so we never want to transition.
        None
    }
}

fn main() {
    // Create a new Automaton with an initial CounterMode.
    let mut automaton =
        Automaton::with_initial_mode(
            UpMode {
                counter: 0,
                target: 3,
            });

    loop {
        // Update the inner Mode.
        {
            let mode = automaton.borrow_mode_mut();

            if let Some(result) = mode.get_result() {
                // If the current mode returns a result, print it and exit the program.
                println!("Result: {}", result);
                break;
            }
            else {
                // Keep updating the current mode until it wants to transition or we get a result.
                mode.update();
            }
        }

        // Allow the Automaton to switch to another Mode after updating the current one, if desired.
        automaton.perform_transitions();
    }
}