// Copyright 2019 Andrew Thomas Christensen
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the
// MIT license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option. This file may not be copied,
// modified, or distributed except according to those terms.

use mode::{Automaton, Family};

// This meta-struct represents a group of all Modes that can be used with the same Automaton, i.e. all states in the
// same state machine. By implementing Family, we can specify the common interface that will be exposed for all states
// (type Base) and how the current state will be stored in the Automaton (type Mode). The important thing to note is
// that this struct will never be instantiated. It only exists to group a set of states (Modes) together.
// 
struct ActivityFamily;

impl Family for ActivityFamily {
    // This is the public interface that will be exposed by the Automaton for all Modes in this Family.
    type Base = dyn Activity;

    // This is the type that will be stored in the Automaton and passed into the Automaton::next() function.
    type Mode = Box<dyn Activity>;
}

// This trait defines a common interface for all Modes in ActivityFamily.
//
trait Activity {
    fn update(self : Box<Self>) -> Box<dyn Activity>;
}

// Each state in the state machine implements both Activity (the Base type) and Mode.
//
struct Working {
    pub hours_worked : u32,
}

impl Activity for Working {
    // This function updates the Mode and allows it to swap another one in as current, when ready.
    //
    fn update(mut self : Box<Self>) -> Box<dyn Activity> {
        println!("Work, work, work...");
        self.hours_worked += 1;

        if self.hours_worked == 4 || self.hours_worked >= 8 {
            // To swap to another Mode, we can return a new, boxed Mode with the same signature as this one. Note that
            // because this function consumes the input Box<Self>, we can freely move state out of this Mode and into
            // the new one that will be swapped in.
            println!("Time for {}!", if self.hours_worked == 4 { "lunch" } else { "dinner" });
            Box::new(Eating { hours_worked: self.hours_worked, calories_consumed: 0 })
        }
        else { self } // Returning self means that this Mode should remain current.
    }
}

struct Eating {
    pub hours_worked : u32,
    pub calories_consumed : u32,
}

impl Activity for Eating {
    fn update(mut self : Box<Self>) -> Box<dyn Activity> {
        println!("Yum!");
        self.calories_consumed += 100;

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
    fn update(mut self : Box<Self>) -> Box<dyn Activity> {
        println!("ZzZzZzZz...");
        self.hours_rested += 1;

        if self.hours_rested >= 8 {
            println!("Time for breakfast!");
            Box::new(Eating { hours_worked: 0, calories_consumed: 0 })
        }
        else { self }
    }
}

fn main() {
    let mut person = ActivityFamily::automaton_with_mode(Box::new(Working { hours_worked: 0 }));
    
    for _age in 18..100 {
        // Update the current Mode and/or transition to another Mode, when the current Mode requests it.
        Automaton::next(&mut person, |current_mode| current_mode.update());
    }
}