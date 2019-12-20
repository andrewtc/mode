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
    type Input = u16;
    type Output = (State, u16);
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum State { A, B, C, D, E, H }

#[derive(Copy, Clone, Debug)]
enum PrintOp { Clear, Print }

#[derive(Copy, Clone, Debug)]
enum ShiftOp { Left,  Right }

impl<'a> Mode for State {
    type Family = StateFamily;
    fn swap(self, mut tape : u16) -> (Self, u16) {
        use State::*;
        use PrintOp::*;
        use ShiftOp::*;

        let bit = (tape & MASK) >> HEAD;

        let (next, op) =
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

        print!("{:016b} {:?} => {:?}, ", tape, self, next);

        if let Some(op) = op {
            println!("{:?}, {:?}", op.0, op.1);
        }
        else {
            println!("Halt");
        }

        if let Some((print_op, shift_op)) = op {
            match print_op {
                Print => { tape = tape |  (1 << HEAD) },
                Clear => { tape = tape & !(1 << HEAD) },
            }

            match shift_op {
                Left  => { tape = tape << 1 },
                Right => { tape = tape >> 1 },
            }
        }

        (next, tape)
    }
}

fn main() {
    let mut tape : u16 = 0b111 << HEAD;
    let mut automaton = StateFamily::automaton_with_mode(State::A);

    while *automaton != State::H {
        tape = Automaton::next_with_input_and_output(&mut automaton, tape);
    }
}