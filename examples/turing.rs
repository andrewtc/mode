// Copyright 2019 Andrew Thomas Christensen
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the
// MIT license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option. This file may not be copied,
// modified, or distributed except according to those terms.

use mode::{Automaton, Family, Mode};
use std::marker::PhantomData;

const HEAD : u16 = 8;
const MASK : u16 = 1 << HEAD;

struct StateFamily<'a> { __ : PhantomData<&'a ()> }
impl<'a> Family for StateFamily<'a> {
    type Base = StateWithContext<'a>;
    type Mode = StateWithContext<'a>;
    type Output = (StateWithContext<'a>, bool);
}

enum State { A, B, C, D, E, H }
enum PrintOp { Clear, Print }
enum ShiftOp { Left,  Right }

struct StateWithContext<'a> {
    state : State,
    tape : &'a mut u16,
}

impl<'a> Mode for StateWithContext<'a> {
    type Family = StateFamily<'a>;
    fn swap(mut self) -> (Self, bool) {
        use PrintOp::*;
        use ShiftOp::*;

        println!("{:#018b}", self.tape);
        let bit = (*self.tape & MASK) >> HEAD;

        let (next, result) =
            match (self.state, bit) {
                (State::A, 0) => (State::H, None),
                (State::A, 1) => (State::B, Some((Clear, Right))),
                (State::B, 0) => (State::C, Some((Clear, Right))),
                (State::B, 1) => (State::B, Some((Print, Right))),
                (State::C, 0) => (State::D, Some((Print,  Left))),
                (State::C, 1) => (State::C, Some((Print, Right))),
                (State::D, 0) => (State::E, Some((Clear,  Left))),
                (State::D, 1) => (State::D, Some((Print,  Left))),
                (State::E, 0) => (State::A, Some((Print, Right))),
                (State::E, 1) => (State::E, Some((Print,  Left))),
                (State::H, _) => (State::H, None),
                            _ => unreachable!(),
            };
        
        let halt = result.is_none();

        if let Some((print_op, shift_op)) = result {
            match print_op {
                Print => { *self.tape = *self.tape |  (1 << HEAD); },
                Clear => { *self.tape = *self.tape & !(1 << HEAD); },
            }

            match shift_op {
                Left  => { *self.tape = *self.tape << 1; },
                Right => { *self.tape = *self.tape >> 1; },
            }
        }

        self.state = next;
        (self, !halt)
    }
}

fn main() {
    let mut tape : u16 = 0b111 << HEAD;
    let mut automaton : Automaton<StateFamily> =
        Automaton::with_mode(
            StateWithContext {
                state: State::A,
                tape:  &mut tape,
            });

    while Automaton::next_with_result(&mut automaton) { }
}