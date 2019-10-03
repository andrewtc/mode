// Copyright 2019 Andrew Thomas Christensen
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the
// MIT license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option. This file may not be copied,
// modified, or distributed except according to those terms.

//! A simple and effective behavioral state machine library, written in in idiomatic, 100% safe, stable Rust code.
//! 
//! This library provides three main types, `Automaton`, `Mode`, and `Transition`, that facilitate the creation of
//! behavioral state machines. An `Automaton` can be used to quickly create a state machine over a set of `Mode`s that
//! implement some `Base` type. Each struct that implements `Mode` represents a distinct state in the state machine, and
//! the `Automaton` allows function calls to be dispatched to the current `Mode` by providing access to it as a `Base`
//! reference. A flexible `Transition` system provides a way for the current `Mode` to swap in a new state when it is
//! ready. The `Transition` system is designed such that the current `Mode` can move data from itself directly into the
//! `Mode` being created, which can help prevent spikes in memory usage while switching from one state to the next.
//! 
//! # Example
//! ```
//! use mode::*;
//! 
//! // This trait will be used as the Base type for the Automaton, defining a common interface
//! // for all states.
//! trait Activity {
//!     fn update(&mut self);
//! }
//! 
//! // Each state in the state machine implements both Activity (the Base type) and Mode.
//! struct Working {
//!     pub hours_worked : u32,
//! }
//! 
//! impl Activity for Working {
//!     fn update(&mut self) {
//!         println!("Work, work, work...");
//!         self.hours_worked += 1;
//!     }
//! }
//! 
//! impl<'a> Mode<'a> for Working {
//!     type Base = dyn Activity + 'a;
//!     fn as_base(&self) -> &Self::Base { self }
//!     fn as_base_mut(&mut self) -> &mut Self::Base { self }
//! 
//!     // This function allows the current Mode to swap to another Mode, when ready.
//!     fn get_transition(&mut self) -> Option<TransitionBox<'a, Self>> {
//!         if self.hours_worked == 4 || self.hours_worked >= 8 {
//!             // To swap to another Mode, a Transition function is returned, which will consume
//!             // the current Mode and return a new Mode to be swapped in as active.
//!             Some(Box::new(|previous : Self| {
//!                 println!("Time for {}!", if previous.hours_worked == 4 { "lunch" } else { "dinner" });
//!                 Eating { hours_worked: previous.hours_worked, calories_consumed: 0 }
//!             }))
//!         }
//!         else { None } // None means don't transition.
//!     }
//! }
//! 
//! struct Eating {
//!     pub hours_worked : u32,
//!     pub calories_consumed : u32,
//! }
//! 
//! impl Activity for Eating {
//!     fn update(&mut self) {
//!         println!("Yum!");
//!         self.calories_consumed += 100;
//!     }
//! }
//! 
//! impl<'a> Mode<'a> for Eating {
//!     type Base = dyn Activity + 'a;
//!     fn as_base(&self) -> &Self::Base { self }
//!     fn as_base_mut(&mut self) -> &mut Self::Base { self }
//!     fn get_transition(&mut self) -> Option<TransitionBox<'a, Self>> {
//!         if self.calories_consumed >= 500 {
//!             if self.hours_worked >= 8 {
//!                 println!("Time for bed!");
//!                 Some(Box::new(|_ : Self| { Sleeping { hours_rested: 0 } }))
//!             }
//!             else {
//!                 println!("Time to go back to work!");
//!                 Some(Box::new(|previous : Self| {
//!                     Working { hours_worked: previous.hours_worked }
//!                 }))
//!             }
//!         }
//!         else { None }
//!     }
//! }
//! 
//! struct Sleeping {
//!     pub hours_rested : u32,
//! }
//! 
//! impl Activity for Sleeping {
//!     fn update(&mut self) {
//!         println!("ZzZzZzZz...");
//!         self.hours_rested += 1;
//!     }
//! }
//! 
//! impl<'a> Mode<'a> for Sleeping {
//!     type Base = dyn Activity + 'a;
//!     fn as_base(&self) -> &Self::Base { self }
//!     fn as_base_mut(&mut self) -> &mut Self::Base { self }
//!     fn get_transition(&mut self) -> Option<TransitionBox<'a, Self>> {
//!         if self.hours_rested >= 8 {
//!             println!("Time for breakfast!");
//!             Some(Box::new(|_| { Eating { hours_worked: 0, calories_consumed: 0 } }))
//!         }
//!         else { None }
//!     }
//! }
//! 
//! fn main() {
//!     let mut person = Automaton::with_initial_mode(Working { hours_worked: 0 });
//!     
//!     for _age in 18..100 {
//!         // Update the current Mode for the Automaton.
//!         // NOTE: We can call update() on the inner Mode through the Automaton reference,
//!         // due to Deref coercion.
//!         person.update();
//! 
//!         // Allow the Automaton to switch Modes.
//!         person.perform_transitions();
//!     }
//! }
//! ```
//! 
//! # Getting started
//! A good place to start reading would be the [`Automaton`](struct.Automaton.html) documentation. For a full example
//! that demonstrates how to use `Automaton` and `Mode` to implement a simple state machine, see `examples/counter.rs`.
//! 
mod automaton;
mod mode;
mod mode_wrapper;
mod transition;

pub use self::automaton::*;
pub use self::mode::*;
pub use self::transition::*;

use self::mode_wrapper::*;