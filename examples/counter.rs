// Copyright 2019 Andrew Thomas Christensen
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the
// MIT license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option. This file may not be copied,
// modified, or distributed except according to those terms.

use mode::{Automaton, Mode};
use std::fmt::Debug;

// Defines the public interface of all Modes below.
trait Counter : Debug {
    // Tells the Counter to update once.
    fn update(&mut self);

    // Returns an i32 if the program is finished and a final result has been returned.
    fn get_result(&self) -> Option<i32> { None }

    // Returns true if the current Counter has the final result, false otherwise.
    fn has_result(&self) -> bool { self.get_result().is_some() }
}

type CounterMode<'a> = dyn Mode<'a, Base = dyn Counter + 'a, Output = bool> + 'a;

// Counter that increments a counter value until it reaches the target value.
#[derive(Debug)]
struct UpMode {
    pub counter : i32,
    pub target : i32,
}

impl Counter for UpMode {
    fn update(&mut self) {
        // Increment the counter until it reaches the target value.
        self.counter += 1;
        print!(" {}", self.counter);
    }
}

impl<'a> Mode<'a> for UpMode {
    type Base = dyn Counter + 'a;
    type Output = bool;

    fn as_base(&self) -> &Self::Base { self }
    fn as_base_mut(&mut self) -> &mut Self::Base { self }

    fn transition(self : Box<Self>) -> (Box<CounterMode<'a>>, bool) {
        if self.counter == self.target {
            // If we've reached the target value, start counting down to (roughly) the median value.
            let next = Box::new(
                DownMode {
                    counter: self.counter,
                    target: (self.counter / 2) + 1,
                });
            (next, true)
        }
        else { (self, false) }
    }
}

// Counter that decrements a counter value until it reaches the target value.
#[derive(Debug)]
struct DownMode {
    pub counter : i32,
    pub target : i32,
}

impl Counter for DownMode {
    fn update(&mut self) {
        // Decrement the counter until it reaches the target value.
        self.counter -= 1;
        print!(" {}", self.counter);
    }
}

impl<'a> Mode<'a> for DownMode {
    type Base = dyn Counter + 'a;
    type Output = bool;

    fn as_base(&self) -> &Self::Base { self }
    fn as_base_mut(&mut self) -> &mut Self::Base { self }

    fn transition(self : Box<Self>) -> (Box<CounterMode<'a>>, bool) {
        const GOAL : i32 = 10;
        if self.counter == GOAL {
            // When we finally count down to the goal value, end the program by swapping in a "finished" state.
            let next = Box::new(
                FinishedMode {
                    result: self.counter,
                });
            (next, true)
        }
        else if self.counter == self.target {
            // If we've reached the target value, start counting up to double the counter value.
            let next = Box::new(
                UpMode {
                    counter: self.counter,
                    target: self.counter * 2,
                });
            (next, true)
        }
        else { (self, false) }
    }
}

// Represents that we've finished counting and have a final result.
#[derive(Debug)]
struct FinishedMode {
    result : i32,
}

impl Counter for FinishedMode {
    fn update(&mut self) { } // We're finished. Do nothing.
    fn get_result(&self) -> Option<i32> { Some(self.result) }
}

impl<'a> Mode<'a> for FinishedMode {
    type Base = dyn Counter + 'a;
    type Output = bool;

    fn as_base(&self) -> &Self::Base { self }
    fn as_base_mut(&mut self) -> &mut Self::Base { self }

    fn transition(self : Box<Self>) -> (Box<CounterMode<'a>>, bool) {
        // We're finished calculating, so we never want to transition.
        (self, false)
    }
}

fn main() {
    // Create a new Automaton with an initial Counter.
    let mut automaton =
        Automaton::with_initial_mode(Box::new(
            UpMode {
                counter: 0,
                target: 3,
            }));

    println!("Starting in {:?}", automaton.as_ref());

    while !automaton.has_result() {
        // Keep updating the current mode until it wants to transition or we get a result.
        automaton.update();

        // Allow the Automaton to switch to another Mode after updating the current one, if desired.
        if Automaton::transition(&mut automaton) {
            println!();
            println!("Switched to {:?}", automaton.as_ref());
        }
    }

    println!("FINISHED! Result: {}", automaton.get_result().unwrap());
}