// Copyright 2019 Andrew Thomas Christensen
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the
// MIT license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option. This file may not be copied,
// modified, or distributed except according to those terms.

//! A simple and effective behavioral state machine library, written in in idiomatic, 100% safe, stable Rust code.
//! 
//! This library provides three main types, `Automaton`, `Mode`, and `Family`, which facilitate the creation of
//! behavioral state machines. An `Automaton` can be used to quickly create a state machine over some set of concrete
//! types that implement the `Mode` trait, called a `Family`. This can contain either:
//! 
//! - a single `Mode` implementation representing all states in the state machine, e.g. a `struct` or an `enum`, or
//! - one or more separate `Mode` implementations that implement a specific `dyn Trait`, with each type representing a
//!   distinct state in the state machine.
//! 
//! The `Automaton` keeps track of a current instance of `Mode`, and provides external access to any members that are
//! common to all `Mode`s in the `Family`. A flexible transition system provides a way for the current `Mode` to swap in
//! a new `Mode` in the same `Family` when it is ready, and is designed such that the current `Mode` can move large
//! amounts of its own state directly into the new `Mode` being created. This can help prevent spikes in memory and CPU
//! usage when switching between `Mode`s.
//! 
//! # Examples
//! For an in-depth demonstration of how to use this library to implement a simple state machine over a single, concrete
//! type, please see `examples/turing.rs`. For a more advanced example demonstrating a state machine over several types
//! in the same `Family`, please see `examples/activity.rs`.
//! 
//! You can run the examples using the following Cargo commands:
//! ```shell
//! cargo run --example turing
//! cargo run --example activity
//! ```
//! 
//! # Getting started
//! A good place to start reading would be the [`Automaton`](struct.Automaton.html) documentation, followed by
//! [`Mode`](trait.Mode.html) and then [`Family`](trait.Family.html).
//! 
mod automaton;
mod family;
mod mode;

pub use self::automaton::*;
pub use self::family::*;
pub use self::mode::*;