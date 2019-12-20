# mode

[![Build Status](https://travis-ci.com/andrewtc/mode.svg?branch=master)](https://travis-ci.com/andrewtc/mode)

A simple and effective behavioral state machine library, written in in idiomatic, 100% safe, stable Rust code.

## Features

This library provides three main types, `Automaton`, `Mode`, and `Family`, that facilitate the creation of behavioral
state machines. An `Automaton` can be used to quickly create a state machine over a `Family` of `struct`s that implement
the `Mode` trait, and allows function calls to be dispatched to the current `Mode` via `Deref` coercion. A flexible
transition system provides a way for the current `Mode` to swap the `Automaton` to a new state when it is ready, and is
designed such that the current `Mode` can freely move data from itself directly into the `Mode` being created, which can
help prevent spikes in memory usage when switching from one state to the next.

## Why use `mode`?

 - **Flexibility:** Create state machines that switch between `enum` values in-place, or organize each state into a
   separate `struct`. Create states with explicit lifetimes and references, or that copy around all the state they need.
   This library makes very few prescriptions about how you should write and organize your code.
 - **Well-documented code:** All public types have detailed documentation and examples, so getting up to speed is easy.
 - **Easy-to-digest internals:** Barring examples, the whole library clocks in at just over 1,000 lines of pure Rust, so
   digging into the internals to figure out how it works does not require a huge time investment. No macro magic, no
   auto-`impl`s. Just `trait`s, `struct`s, and generics.

## Releases

See the full list of releases on [GitHub](https://github.com/andrewtc/mode/releases).

## Upgrading to version `0.3`

With version `0.3`, the entire library has been rewritten to be much simpler, more intuitive, and more flexible. If
you're interested in upgrading your project from version `^0.2` to `0.3` of `mode`, please see
[README-v0.3.md](README-v0.3.md) for a full explanation of what has changed, as well as
[UPGRADING-v0.3.md](UPGRADING-v0.3.md) for a step-by-step guide on how to update your code.

## Documentation

Please see [docs.rs](https://docs.rs/mode) for detailed documentation.

## Example

```rust
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

    // This is the type that will be passed into swap() for all Modes in this Family.
    type Input = ();

    // This is the type that will be returned by swap() for all Modes in this Family.
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
    fn swap(self : Box<Self>, _input : ()) -> Box<dyn Activity> {
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

    fn swap(self : Box<Self>, _input : ()) -> Box<dyn Activity> {
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

    fn swap(self : Box<Self>, _input : ()) -> Box<dyn Activity> {
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
```

# License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](https://github.com/andrewtc/mode/blob/master/LICENSE-APACHE) or 
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](https://github.com/andrewtc/mode/blob/master/LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.

## Contributing

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as
defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

If you find bugs, please feel free to open an issue on [GitHub](https://github.com/andrewtc/mode/issues)! Otherwise, if
you would like to propose changes to this library, feel free to send me a pull request and I will handle them as best I
can. Thanks!