// Copyright 2019 Andrew Thomas Christensen
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the
// MIT license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option. This file may not be copied,
// modified, or distributed except according to those terms.

use mode::{Automaton, Family, Mode, ModeIn};
use std::marker::PhantomData;

// This meta-`struct` represents a group of all `Mode`s that can be used with the same Automaton. By implementing
// `Family`, we can specify common `Base` and `Output` types for all Modes in this Family. The important thing to note
// is that it will never be instantiated. It only exists to group a set of `Mode`s together.
// 
struct ActivityFamily<'a> { __ : PhantomData<&'a ()> }

impl<'a> Family for ActivityFamily<'a> {
    // This is the public interface that will be exposed by the Automaton for all Modes in this Family.
    type Base = dyn Activity;

    // This is the type that will be returned by Automaton::transition() for all Modes in this Family.
    type Output = Box<ModeIn<'a, Self>>;
}

// This trait will be used as the Base type for the Automaton, defining a common interface
// for all states.
trait Activity {
    fn update(&mut self);
}

// Each state in the state machine implements both Activity (the Base type) and Mode.
struct Working {
    pub hours_worked : u32,
}

impl Activity for Working {
    fn update(&mut self) {
        println!("Work, work, work...");
        self.hours_worked += 1;
    }
}

impl<'a> Mode<'a> for Working {
    type Family = ActivityFamily<'a>;

    fn as_base(&self) -> &<Self::Family as Family>::Base { self }
    fn as_base_mut(&mut self) -> &mut <Self::Family as Family>::Base { self }

    // This function allows the current Mode to swap to another Mode, when ready.
    fn transition(self : Box<Self>) -> Box<ModeIn<'a, ActivityFamily<'a>>> {
        if self.hours_worked == 4 || self.hours_worked >= 8 {
            // To swap to another Mode, a Transition function is returned, which will consume
            // the current Mode and return a new Mode to be swapped in as active.
            println!("Time for {}!", if self.hours_worked == 4 { "lunch" } else { "dinner" });
            Box::new(Eating { hours_worked: self.hours_worked, calories_consumed: 0 })
        }
        else { self } // None means don't transition.
    }
}

struct Eating {
    pub hours_worked : u32,
    pub calories_consumed : u32,
}

impl Activity for Eating {
    fn update(&mut self) {
        println!("Yum!");
        self.calories_consumed += 100;
    }
}

impl<'a> Mode<'a> for Eating {
    type Family = ActivityFamily<'a>;

    fn as_base(&self) -> &<Self::Family as Family>::Base { self }
    fn as_base_mut(&mut self) -> &mut <Self::Family as Family>::Base { self }

    fn transition(self : Box<Self>) -> Box<ModeIn<'a, ActivityFamily<'a>>> {
        if self.calories_consumed >= 500 {
            if self.hours_worked >= 8 {
                println!("Time for bed!");
                Box::new(Sleeping { hours_rested: 0 })
            }
            else {
                println!("Time to go back to work!");
                Box::new(Working { hours_worked: self.hours_worked })
            }
        }
        else { self }
    }
}

struct Sleeping {
    pub hours_rested : u32,
}

impl Activity for Sleeping {
    fn update(&mut self) {
        println!("ZzZzZzZz...");
        self.hours_rested += 1;
    }
}

impl<'a> Mode<'a> for Sleeping {
    type Family = ActivityFamily<'a>;

    fn as_base(&self) -> &<Self::Family as Family>::Base { self }
    fn as_base_mut(&mut self) -> &mut <Self::Family as Family>::Base { self }

    fn transition(self : Box<Self>) -> Box<ModeIn<'a, ActivityFamily<'a>>> {
        if self.hours_rested >= 8 {
            println!("Time for breakfast!");
            Box::new(Eating { hours_worked: 0, calories_consumed: 0 })
        }
        else { self }
    }
}

fn main() {
    let mut person = Automaton::with_initial_mode(Box::new(Working { hours_worked: 0 }));
    
    for _age in 18..100 {
        // Update the current Mode for the Automaton.
        // NOTE: We can call update() on the inner Mode through the Automaton reference,
        // due to Deref coercion.
        person.update();

        // Allow the Automaton to switch Modes.
        Automaton::transition(&mut person);
    }
}