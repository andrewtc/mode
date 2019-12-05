// Copyright 2019 Andrew Thomas Christensen
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the
// MIT license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option. This file may not be copied,
// modified, or distributed except according to those terms.

use mode::{Automaton, boxed, Family};

// This meta-struct represents a group of all Modes that can be used with the same Automaton. By implementing Family,
// we can specify common Base and Output types for all Modes in this Family. The important thing to note is that this
// struct will never be instantiated. It only exists to group a set of Modes together.
// 
struct ActivityFamily;

impl Family for ActivityFamily {
    // This is the public interface that will be exposed by the Automaton for all Modes in this Family.
    type Base = dyn Activity;

    // This is the type that will be stored in the Automaton and passed into the Mode::swap() function.
    type Mode = Box<dyn Activity>;

    // This is the type that will be returned by Automaton::next() for all Modes in this Family.
    type Output = Box<dyn Activity>;
}

// This trait defines a common interface for all Modes in ActivityFamily.
//
trait Activity : boxed::Mode<Family = ActivityFamily> {
    fn update(&mut self);
}

// Each Mode in the state machine implements both Activity (the Base type) and boxed::Mode.
//
struct Working {
    pub hours_worked : u32,
}

impl Activity for Working {
    fn update(&mut self) {
        println!("Work, work, work...");
        self.hours_worked += 1;
    }
}

impl boxed::Mode for Working {
    type Family = ActivityFamily;

    // This function allows the current Mode to swap to another Mode, when ready.
    //
    fn swap(self : Box<Self>) -> Box<dyn Activity> {
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
    fn update(&mut self) {
        println!("Yum!");
        self.calories_consumed += 100;
    }
}

impl boxed::Mode for Eating {
    type Family = ActivityFamily;

    fn swap(self : Box<Self>) -> Box<dyn Activity> {
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

impl boxed::Mode for Sleeping {
    type Family = ActivityFamily;

    fn swap(self : Box<Self>) -> Box<dyn Activity> {
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
        // Update the current Mode for the Automaton.
        // NOTE: Using Deref coercion, we can call Activity::update() on the inner Mode through the Automaton itself.
        person.update();

        // Allow the Automaton to switch Modes.
        Automaton::next(&mut person);
    }
}