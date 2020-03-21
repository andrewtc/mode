# How to upgrade from `mode` version `0.3` to `0.4`

This guide will help you upgrade your code from version `0.3` to `0.4` of `mode` in just a few easy steps!

**Note:** Most of the following examples have been adapted from the Activity example. If you diff `examples/activity.rs`
between versions `0.3` and `0.4`, you will be able to see (roughly) the same list of changes detailed below.

## 0. Bump the `mode` version number in `Cargo.toml`

Before doing anything else, you should bump the `mode` version number in your project's `Cargo.toml`, like this:

```toml
mode = "^0.4"
```

After that, run the following command from your project's root folder to update the dependency:

```cmd
cargo update --package mode
```

With that, you should be ready to start refactoring.

## A quick note on the `Mode::swap()` function

Previously, there was a required `swap()` function for each `Mode` implementation. This function was called on the
active `Mode` whenever one of the `Automaton::next()` family of functions was invoked, in order to allow it to make
another `Mode` in as active, if desired:

```rust
fn swap(self : Box<Self>, _input : ()) -> Box<dyn Activity> {
    if self.hours_worked == 4 || self.hours_worked >= 8 {
        println!("Time for {}!", if self.hours_worked == 4 { "lunch" } else { "dinner" });
        Box::new(Eating { hours_worked: self.hours_worked, calories_consumed: 0 })
    }
    else { self } // Returning self means that this Mode should remain current.
}
```

In `0.4`, this function has been eliminated entirely. Instead, `Automaton::next()` takes a closure that is called on the
current `Mode` in order to transition it. The idea is that the callback can capture any necessary state from the calling
function and use it during the transition process. In the Activity example, that looks like this:

```rust
// Update the current Mode and/or transition to another Mode, when the current Mode requests it.
Automaton::next(&mut person, |current_mode| current_mode.update());
```

The great thing about this new system is that it provides a lot more flexibility when it comes to the transition
process. It doesn't matter how transitioning is accomplished, so long as the callback returns a new `Mode` to be swapped
in at the end. The callback can be a closure, a free function, or any other `FnOnce` that consumes a `Family::Mode` as
the first argument and returns a new one to be swapped in.

You'll notice that in the new Activity example, the code in the `Mode::swap()` function for each state has been moved
into `Activity::update()`, which now takes a `self : Box<Self>` and returns a `Box<Activity>`, as `swap()` once did:

```rust
impl Activity for Working {
    fn update(mut self : Box<Self>) -> Box<dyn Activity> {
        println!("Work, work, work...");
        self.hours_worked += 1;

        if self.hours_worked == 4 || self.hours_worked >= 8 {
            println!("Time for {}!", if self.hours_worked == 4 { "lunch" } else { "dinner" });
            Box::new(Eating { hours_worked: self.hours_worked, calories_consumed: 0 })
        }
        else { self }
    }
}
```

This accomplishes the same thing that `boxed::Mode::swap()` did in the previous example: delegating the responsibility
for transitioning to the current `Mode` in the `Automaton`, except that we can now call it whatever we want and have
total control over the function signature. `Activity::update()` is just a normal `trait` function, which we can call on
`current_mode` because is moved into the closure as a `Box<Activity>`.

Obviously, you can solve this transition problem however you want. However, if you just want to get your code compiling
again quickly, the easiest way to upgrade your code is outlined in the sections below.

## 1. Remove the `Input` and `Output` associated `type`s from each `Family` implementation

In version `0.3` of `mode`, each `Family` struct was required to define several associated types, like this:

```rust
struct ActivityFamily;
impl Family for ActivityFamily {
    type Base = dyn Activity;
    type Mode = Box<dyn Activity>;
    type Input = ();
    type Output = Box<dyn Activity>;
}
```

With version `0.4`, the `Input` and `Output` associated `type`s are no longer necessary. Hence, you can remove those
associated `type`s from the `impl` entirely. Once you're done, it should look something like this:

```rust
struct ActivityFamily;
impl Family for ActivityFamily {
    type Base = dyn Activity;
    type Mode = Box<dyn Activity>;
}
```

## 2. Replace any `impl`s for `boxed::Mode`, `rc::Mode`, and `sync::Mode` with `impl`s for `Mode`

Previously, in order to have an `Automaton` store the current `Mode` by pointer type, it was necessary to define a
`swap()` function for each `Mode` that took the pointer type as the `self` parameter. This was accomplished by
implementing one of various `Mode` traits corresponding to the pointer type being stored, e.g. `boxed::Mode`:

```rust
impl boxed::Mode for Working {
    type Family = ActivityFamily;
    fn swap(self : Box<Self>, _input : ()) -> Box<dyn Activity> {
        // ...
    }
}
```

In `0.4`, these separate traits have disappeared entirely (along with the `swap()` function itself). Now, all that is
necessary to implement `Mode` for a type is to specify the `Family` to which the type belongs, like so:

```rust
impl Mode for Working {
    type Family = ActivityFamily;
}
```

Hence, you can replace any replace any references to the separate pointer-specific `trait`s, `boxed::Mode`, `rc::Mode`,
and `sync::Mode`, with the root-level `Mode`. The code in the `swap()` function will also need to move, as `Mode` no
longer defines a `swap()` function.

## 3. Move the `swap()` function to the `Base` type for each `Family`

Since `trait Mode` no longer defines a `swap()` function, we need to find a new home for the `swap()` function on each
`Mode` implementation. Since the callback in `Automaton::next()` consumes an `F::Base`, we can call any function that is
defined on that type. Hence, if we define a `swap()` function on the `Base` type with the same signature as the old
`Mode::swap()` implementation, we can call it through the `Base` type when we call `Automaton::next()`:

If we were to update the old Activity example in this way, it would look something like this:

```rust
trait Activity : Mode<Family = ActivityFamily> {
    fn update(&mut self);
    fn swap(self :Box<Self>, input : ()) -> Box<dyn Activity>; // TODO: Remove the unnecessary input parameter.
}
```

In this case, after adding a `swap()` function to `Activity`, we're required to implement it for each `Mode` in
`ActivityFamily`. Since the signature is the same as the old `Mode::swap()` function, we could simply move the `swap()`
implementation for each `impl Mode` into the `impl` for `Activity`

```rust
impl Mode for Working {
    type Family = ActivityFamily;

    // NOTE: No more fn swap() in here!
}

impl Activity for Working {
    fn update(&mut self) {
        println!("Work, work, work...");
        self.hours_worked += 1;
    }

    fn swap(self : Box<Self>, _input : ()) -> Box<dyn Activity> { // This is now defined when implementing Activity.
        if self.hours_worked == 4 || self.hours_worked >= 8 {
            println!("Time for {}!", if self.hours_worked == 4 { "lunch" } else { "dinner" });
            Box::new(Eating { hours_worked: self.hours_worked, calories_consumed: 0 })
        }
        else { self }
    }
}
```

## 4. Update all calls to `Automaton::next*()`

In `mode` version `0.4`, there is no more `Automaton::next_with_input()`, `Automaton::next_with_output()`, etc. Instead,
this `next*()` family of functions has been reduced down to two:

1. ```rust
   pub fn next<T>(automaton : &mut Self, transition_fn : T)
       where T : FnOnce(F::Mode) -> F::Mode
   ```
   Takes a `&mut Automaton` and a `FnOnce(F::Mode) -> F::Mode` callback. When called, the callback
   is called on the current `Mode`, consuming it and producing another `Mode` to swap in as active. As in version `0.3`,
   if the callback returns the input `Mode`, the current `Mode` will remain active. This replaces `Automaton::next()`
   and `Automaton::next_with_input()` from `0.3`.
2. ```rust
   pub fn next_with_result<T, R>(automaton : &mut Self, transition_fn : T) -> R
       where T : FnOnce(F::Mode) -> (F::Mode, R)
   ```
   Same as the function above, except that the input callback has a signature of `FnOnce(F::Mode) -> (F::Mode, R)`,
   where `R` is an arbitrary return type that will be returned from the `next_with_result()` function when it exits.
   This replaces `Automaton::next_with_output()` and `Automaton::next_with_input_output()` from `0.3`.

Right now, all calls to `Automaton::next*()` in your own code will look something like one of these four use cases:

```rust
// CASE #1: Call swap() on the current Mode and transition to whatever Mode is returned.
Automaton::next(&mut automaton);

// CASE #2: Call swap() on the current Mode, passing in some input.
Automaton::next_with_input(&mut automaton, input);

// CASE #3: Call swap() on the current Mode, returning some result.
let result = Automaton::next_with_output(&mut automaton);

// CASE #4: Call swap() on the current Mode, passing in some input and returning some result.
let result = Automaton::next_with_input_output(&mut automaton, input);
```

Since we haven't changed anything about the `swap()` function, except that we moved to our `Base` implementation, it's
pretty easy for us to replace the old `Automaton::next*()` calls with their `0.4` equivalents:

```rust
// CASE #1: Call swap() on the current Mode and transition to whatever Mode is returned.
Automaton::next(&mut automaton, |current_mode| current_mode.swap(()));

// CASE #2: Call swap() on the current Mode, passing in some input.
Automaton::next(&mut automaton, |current_mode| current_mode.swap(input));

// CASE #3: Call swap() on the current Mode, returning some result.
let result = Automaton::next_with_result(&mut automaton, |current_mode| current_mode.swap(()));

// CASE #4: Call swap() on the current Mode, passing in some input and returning some result.
let result = Automaton::next_with_result(&mut automaton, |current_mode| current_mode.swap(input));
```

Here's what changed:

 - The `next()` function replaced `next_with_input()`, and `next_with_result()` replaced both `next_with_output()` and
   `next_with_input_output()`.
 - All of these new function calls take in a closure in the second parameter that calls `swap()` on the current `Mode`.
 - Instead of passing an input parameter into `next_with_input()` or `next_with_input_output()`, we can just use
   `next()` and `next_with_result()` and pass the input parameter directly into `swap()` from inside the closure.

### A quick note on the `swap()` function

In the `Activity` example, you'll notice that the `input` parameter is always an empty tuple, and the value is never
used in the `swap()` implementations of each `Mode` in `ActivityFamily`. Now that we can control the signature of
`swap()`, we could easily remove this parameter entirely, which would allow us to do this:

```rust
Automaton::next(&mut automaton, |current_mode| current_mode.swap()); // No more input parameter to swap()!
```

There's also nothing that says we couldn't pass *more* input parameters to `swap()`, if we needed them:

```rust
Automaton::next(&mut automaton, |current_mode| current_mode.swap(foo, bar, baz)); // Three input parameters!
```

As you can see, you have a lot more control now over how your `Automaton` transitions between `Mode`s, which is a very
good thing.

# Troubleshooting

After making these changes, everything should compile and you should be good to go! If you have any difficulty with
this, try diffing the code in the `examples` folder, to catch any changes that you may have missed. If you continue to
have trouble, feel free to join the `mode`
[Gitter channel](https://gitter.im/mode-rs/community?utm_source=share-link&utm_medium=link&utm_campaign=share-link) and
ask questions!

Enjoy!