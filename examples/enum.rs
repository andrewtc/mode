// Copyright 2019 Andrew Thomas Christensen
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the
// MIT license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option. This file may not be copied,
// modified, or distributed except according to those terms.

use mode::{Automaton, Family, Mode};

struct ActivityFamily;

impl Family for ActivityFamily {
    type Base = Activity;
    type Mode = Activity;
    type Output = Activity;
}

#[derive(Copy, Clone)]
enum Activity {
    Working { hours_worked : u32 },
    Eating { hours_worked : u32, calories_consumed : u32 },
    Sleeping { hours_rested : u32 },
}

impl Activity {
    fn update(&mut self) {
        match self {
            Activity::Working{ hours_worked } => {
                println!("Work, work, work...");
                *hours_worked += 1;
            },
            Activity::Eating { calories_consumed, .. } => {
                println!("Yum!");
                *calories_consumed += 100;
            },
            Activity::Sleeping { hours_rested } => {
                println!("ZzZzZzZz...");
                *hours_rested += 1;
            },
        }
    }
}

impl Mode for Activity {
    type Family = ActivityFamily;

    fn swap(self) -> Self {
        match self {
            Activity::Working { hours_worked } => {
                if hours_worked == 4 || hours_worked >= 8 {
                    println!("Time for {}!", if hours_worked == 4 { "lunch" } else { "dinner" });
                    Activity::Eating { hours_worked, calories_consumed: 0 }
                }
                else { self }
            },
            Activity::Eating { hours_worked, calories_consumed } => {
                if calories_consumed >= 500 {
                    if hours_worked >= 8 {
                        println!("Time for bed!");
                        Activity::Sleeping { hours_rested: 0 }
                    }
                    else {
                        println!("Time to go back to work!");
                        Activity::Working { hours_worked }
                    }
                }
                else { self }
            },
            Activity::Sleeping { hours_rested } => {
                if hours_rested >= 8 {
                    println!("Time for breakfast!");
                    Activity::Eating { hours_worked: 0, calories_consumed: 0 }
                }
                else { self }
            },
        }
    }
}

fn main() {
    let mut person = Automaton::<ActivityFamily>::with_mode(Activity::Working { hours_worked: 0 });
    
    for _age in 18..100 {
        person.update();
        Automaton::next(&mut person);
    }
}