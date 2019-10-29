// Copyright 2019 Andrew Thomas Christensen
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the
// MIT license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option. This file may not be copied,
// modified, or distributed except according to those terms.

use mode::{Automaton, Family, Mode};

const HEAD : u16 = 8;
const MASK : u16 = 1 << HEAD;

struct StateFamily;
impl Family for StateFamily {
    type Base = State;
    type Mode = State;
    type Input = u16; // Needs to be a RefCell instead of a mut borrow because we can't elide the lifetime.
    type Output = (State, u16);
}

enum State { A, B, C, D, E, H }
enum PrintOp { Clear, Print }
enum ShiftOp { Left,  Right }

impl Mode for State {
    type Family = StateFamily;
    fn swap(self, mut tape : u16) -> (Self, u16) {
        use State::*;
        use PrintOp::*;
        use ShiftOp::*;

        let bit = (tape & MASK) >> HEAD;

        let (next, result) =
            match (self, bit) {
                (A, 0) => (H, None),
                (A, 1) => (B, Some((Clear, Right))),
                (B, 0) => (C, Some((Clear, Right))),
                (B, 1) => (B, Some((Print, Right))),
                (C, 0) => (D, Some((Print,  Left))),
                (C, 1) => (C, Some((Print, Right))),
                (D, 0) => (E, Some((Clear,  Left))),
                (D, 1) => (D, Some((Print,  Left))),
                (E, 0) => (A, Some((Print, Right))),
                (E, 1) => (E, Some((Print,  Left))),
                (H, _) => (H, None),
                (_, _) => unreachable!(),
            };

        if let Some((print_op, shift_op)) = result {
            match print_op {
                Print => { tape = tape |  (1 << HEAD); },
                Clear => { tape = tape & !(1 << HEAD); },
            }

            match shift_op {
                Left  => { tape = tape << 1; },
                Right => { tape = tape >> 1; },
            }
        }

        (next, tape)
    }
}

fn main() {
    let mut tape : u16 = 0b111 << HEAD;
    let mut automaton : Automaton<StateFamily> = Automaton::with_mode(State::A);

    loop {
        println!("{:#018b}", tape);
        tape = Automaton::next_with_input_output(&mut automaton, tape);
        if let &State::H = automaton.borrow_mode() { break; }
    }
}