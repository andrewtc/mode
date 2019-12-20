# How to upgrade from `mode` version `^0.2` to `0.3`

Because of the rewrite, upgrading from `^0.2` to `0.3` will involve refactoring your code a bit. This guide will walk
you through all of the necessary steps in order to make that process as simple and straightforward as possible.

**Note:** Most of the following examples have been adapted from the Activity example. If you diff `examples/activity.rs`
between versions `0.2.4` and `0.3`, you will be able to see (roughly) the same list of changes detailed below.

## 0. Bump the `mode` version number in `Cargo.toml`

Before doing anything else, you should bump the `mode` version number in your project's `Cargo.toml`, like this:

```toml
mode = "^0.3"
```

After that, run the following command from your project's root folder to update the dependency:

```cmd
cargo update --package mode
```

After doing this, you should be able to continue on with upgrading your code.

## 1. Define a new `Family` type for each separate `Automaton`

In order to upgrade to `0.3`, you will need to define a separate `Family` struct for each distinct group of `Mode`s
that are part of the same state machine, with:

1. A `Base` associated `type` equal to the old `Mode::Base` type, e.g. `dyn Activity` in the Activity example.
2. A `Mode` associated `type` that is *either*:
    - the same type as `Base`, if `Base` is a *concrete* type, e.g. an `enum` or a `struct`, or
    - a pointer type (e.g. `Box` or `Arc`) holding the `Base` type, if `Base` is a `dyn Trait`, e.g. `Box<dyn Activity>`
      in the Activity example.
3. An `Input` associated `type` equal to the empty tuple type, `()`. You can ignore this, for now.
4. An `Output` associated `type` equal to *either*:
    - the same type as `Self::Mode`, if no return value is desired when switching `Mode`s, or
    - a tuple of the form `(M, R)`, where `M` is the same type as `Self::Mode` and `R` is the desired return type of the
      `Automaton::next_with_output()` function. (For more, please see the
      [`Automaton` section](#4-update-usages-of-automaton), below.)

**Note:** If you were previously making use of the return value of `Automaton::perform_transitions()` in `^0.2`, you
probably want `Output` to be `(M, bool)`. More on that to come.

Since the `Family` trait is *only* used to define associated types, the `Family` struct you define can simply be a unit
struct, i.e. a `struct` with no fields. Once you're done, it should look something like this:

```rust
struct ActivityFamily;
impl Family for ActivityFamily {
    type Base = dyn Activity;
    type Mode = Box<dyn Activity>;
    type Input = ();
    type Output = Box<dyn Activity>;
}
```

## 2. Update each `Mode` implementation

If you were previously using an `Automaton` with a `dyn Trait` as the base type, your `Mode` implementations probably
look something like this:

```rust
impl<'a> Mode<'a> for Working {
    type Base = dyn Activity + 'a;
    fn as_base(&self) -> &Self::Base { self }
    fn as_base_mut(&mut self) -> &mut Self::Base { self }
    fn get_transition(&mut self) -> Option<TransitionBox<'a, Self>> {
        // ...
    }
}
```

To upgrade this code, you will need to:

1. Remove all references to the `'a` lifetime from the `impl` block. These are no longer necessary.
2. Replace the old `Base` associated type with a new `Family` associated type equal to the `Family` struct you created
   in the [previous section](#1-define-a-new-family-type-for-each-separate-automaton).
3. If `Base` was previously a `dyn Trait` type, as in the example above, you will need to change the `impl` from `Mode`
   to **one** of the following:
   - `boxed::Mode`, if the `Family` you defined has a `Box<dyn Trait>` for its `Mode` type.
   - `rc::Mode`, if the `Family` you defined has an `Rc<dyn Trait>` for its `Mode` type.
   - `sync::Mode`, if the `Family` you defined has an `Arc<dyn Trait>` for its `Mode` type.
4. Remove the `as_base()` and `as_base_mut()` functions from the `impl` block. These are no longer necessary.
5. Replace the `get_transition()` function implementation with a `swap()` function. (See the
   [next section](#replacing-get_transition-with-swap) for more details.)

Once you're done, your `Mode` implementation should look something like this:

```rust
impl boxed::Mode for Working {
    type Family = ActivityFamily;
    fn swap(self : Box<Self>, _input : ()) -> Box<dyn Activity> {
        // ...
    }
}
```

### Replacing `get_transition()` with `swap()`

This is the most involved part of the upgrade process. Previously, each `Mode` would return a boxed closure from this
function to indicate that it wanted to switch the `Automaton` to another state. (See the snippet below, taken from the
Activity example.)

```rust
fn get_transition(&mut self) -> Option<TransitionBox<'a, Self>> {
    if self.hours_worked == 4 || self.hours_worked >= 8 {
        Some(Box::new(|previous : Self| {
            println!("Time for {}!", if previous.hours_worked == 4 { "lunch" } else { "dinner" });
            Eating { hours_worked: previous.hours_worked, calories_consumed: 0 }
        }))
    }
    else { None } // None means don't transition.
}
```

The `0.3` equivalent of this code is *much* simpler and more readable:

```rust
fn swap(self : Box<Self>, _input : ()) -> Box<dyn Activity> {
    if self.hours_worked == 4 || self.hours_worked >= 8 {
        println!("Time for {}!", if self.hours_worked == 4 { "lunch" } else { "dinner" });
        Box::new(Eating { hours_worked: self.hours_worked, calories_consumed: 0 })
    }
    else { self } // Returning self means that this Mode should remain current.
}
```

Some key differences:

 - Instead of returning a `TransitionBox`, you can simply return a new `Mode` implementation from the function when you
   want to transition.
   - **Note:** The actual return type of The `swap()` function will be different depending on the specific `Mode` trait
     you chose to use in the [`Mode` section](#2-update-each-mode-implementation). In this case, it's
     `Box<dyn Activity>` because `boxed::Mode` is being used.
 - Since `swap()` always moves `self` into the function, you can return `self` whenever the current `Mode` should stay
   active, i.e. you should return `self` wherever you were previously returning `None` from the old `get_transition()`
   function.
   - **Note:** The `self` type *also* varies based on the `Mode` trait being used. In this case, since we're using
     `boxed::Mode`, it's `Box<Self>`.

## 3. Have any `dyn Trait`s used as a `Base` extend `Mode`

One new constraint of the `Family::Base` type is that it *must* `impl Mode<Family = Self>`. `Self` here means the
*concrete* `Family` implementation for which we are defining `Base`, e.g. `ActivityFamily`. Hence, any `dyn Trait` that
is used as a `Base` type for a `Family` *must* extend `Mode` so that it can be specified as the `Base` type, with a
`Family` type that matches the `Family` struct that will make use of it.

Due to this new constraint, you will need to find any `dyn Trait` being used as a `Base` type, e.g.

```rust
trait Activity {
    fn update(&mut self);
}
```

and modify it to extend `Mode` with the proper `Family` type, like so:

```rust
trait Activity : boxed::Mode<Family = ActivityFamily> {
    fn update(&mut self);
}
```

## 4. Update usages of `Automaton`

Once you've made it through all of the previous upgrade steps, you will need to upgrade all usages of `Automaton` to
match the new `0.3` conventions. Right now, each usage of `Automaton` in your code probably looks something like this:

```rust
// Construct the Automaton.
let mut person = Automaton::with_initial_mode(Working { hours_worked: 0 });

// TODO: Call functions on the Automaton.

// Let the Automaton transition.
person.perform_transitions();
```

In order to upgrade this code, you will need to:

1. Replace calls to constructor functions with their `0.3` equivalents.
   - Any calls to `Automaton::new()` should be replaced with `Family::automaton()`.
   - Any calls to `Automaton::with_initial_mode()` should be replaced with `Family::automaton_with_mode()`.
2. Replace each call to `Automaton::perform_transitions()` with *one* of the following:
   - If the `Output` type of the `Family` for the `Automaton` is the same as the `Mode` associated type, you will need
     to use `Automaton::next()`.
   - If the `Output` type for the `Family` is a tuple, e.g. `(Box<dyn Activity>, bool)`, you will need to use
     `Automaton::next_with_output()` instead.
   - **Note:** Both of these are associated functions that take a `&mut Automaton` as the first argument.

After making these changes, your code should look something like this:

```rust
// Construct the Automaton.
let mut person = ActivityFamily::automaton_with_mode(Box::new(Working { hours_worked: 0 }));

// TODO: Call functions on the Automaton.

// Let the Automaton transition.
Automaton::next(&mut person);
```

# Troubleshooting

That's it! After you make these changes, your project should compile with `mode` version `0.3`. If you have any
difficulty getting your code to compile, please compare your code with the snippets in the `examples` folder. The
examples there cover a few different use cases of the library, and should be helpful in determining any additional steps
you may need to take in order to get up and running.

Enjoy!