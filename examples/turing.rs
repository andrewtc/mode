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
    type Base = State<'a>;
    type Mode = State<'a>;
    type Output = (State<'a>, bool);
}

#[derive(Copy, Clone, Debug)]
enum Name { A, B, C, D, E, H }

#[derive(Copy, Clone, Debug)]
enum PrintOp { Clear, Print }

#[derive(Copy, Clone, Debug)]
enum ShiftOp { Left,  Right }

#[derive(Debug)]
struct State<'a> {
    name : Name,
    tape : &'a mut u16,
}

impl<'a> Mode for State<'a> {
    type Family = StateFamily<'a>;
    fn swap(mut self) -> (Self, bool) {
        use Name::*;
        use PrintOp::*;
        use ShiftOp::*;

        let bit = (*self.tape & MASK) >> HEAD;

        let (next, op) =
            match (self.name, bit) {
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

        print!(
            "{:016b} {:?} => {:?}, ",
            self.tape,
            self.name,
            next);

        if let Some(op) = op {
            println!("{:?}, {:?}", op.0, op.1);
        }
        else {
            println!("Halt");
        }

        let halt = op.is_none();

        if let Some((print_op, shift_op)) = op {
            match print_op {
                Print => { *self.tape = *self.tape |  (1 << HEAD); },
                Clear => { *self.tape = *self.tape & !(1 << HEAD); },
            }

            match shift_op {
                Left  => { *self.tape = *self.tape << 1; },
                Right => { *self.tape = *self.tape >> 1; },
            }
        }

        self.name = next;
        (self, !halt)
    }
}

fn main() {
    let mut tape : u16 = 0b111 << HEAD;
    let mut automaton : Automaton<StateFamily> =
        Automaton::with_mode(
            State{
                name: Name::A,
                tape: &mut tape,
            });

    while Automaton::next_with_output(&mut automaton) { }
}