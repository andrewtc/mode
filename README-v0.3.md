# What's new in version `0.3`?

With version `0.3`, this library has been completely rewritten. It is now much simpler, easier-to-use, and more flexible
than in previous versions. There's a lot that changed, so this guide will provide you with:

 - a high-level overview of the key differences between `mode` versions `^0.2` and `0.3`, and
 - a detailed explanation of why these changes were made and how they make `mode` a better library.

This document gets into a lot of the implementation details of `mode` version `0.3`. If you're already familiar with
these, or you are just interested in the steps necessary to upgrade your own project, feel free to skip over this file
and give [UPGRADING-v0.3.md](UPGRADING-v0.3.md) a read instead.

**Note:** Most of the following examples have been adapted from the [Activity](./examples/activity.rs) example. If you
diff that file between versions `0.2.4` and `0.3`, you will be able to see (roughly) the same list of changes detailed
below.

## The `Mode` trait has been simplified

Previously, implementing the `Mode` trait for a `struct` looked something like this:

```rust
impl<'a> Mode<'a> for Working {
    type Base = dyn Activity + 'a; // The Mode can ONLY be accessed from outside the Automaton as a dyn Activity.
    fn as_base(&self) -> &Self::Base { self }
    fn as_base_mut(&mut self) -> &mut Self::Base { self }
    fn get_transition(&mut self) -> Option<TransitionBox<'a, Self>> {
        if self.hours_worked == 4 || self.hours_worked >= 8 {
            Some(Box::new(|previous : Self| {
                println!("Time for {}!", if previous.hours_worked == 4 { "lunch" } else { "dinner" });
                Eating { hours_worked: previous.hours_worked, calories_consumed: 0 }
            }))
        }
        else { None } // None means don't transition.
    }
}
```

Here's what that same `impl` looks like in `0.3`:

```rust
impl boxed::Mode for Working {
    // This struct belongs to the "activity" Family of Modes, and can transition to any sibling in the same Family.
    type Family = ActivityFamily;
    fn swap(self : Box<Self>, _input : ()) -> Box<dyn Activity> {
        if self.hours_worked == 4 || self.hours_worked >= 8 {
            println!("Time for {}!", if self.hours_worked == 4 { "lunch" } else { "dinner" });
            Box::new(Eating { hours_worked: self.hours_worked, calories_consumed: 0 })
        }
        else { self } // Returning self means that this Mode should remain current.
    }
}
```

You'll notice that quite a few things have changed:

 - Instead of implementing `Mode`, the `struct` in the example (`Working`) now implements `boxed::Mode`.
 - The `'a` lifetime has been completely removed.
 - The `Base` associated type has been replaced with a `Family` associated type. (See the
   [`Family` section](#mode::base-has-been-replaced-by-the-new-family-associated-type) for more details.)
 - No more `as_base()` or `as_base_mut()`.
 - The `get_transition()` function has been replaced by `swap()`, which in this example takes `self : Box<Self>`.
 - Instead of returning an `Option<TransitionBox<'a, Self>>` like the old `get_transition()` function, `swap()` simply
   returns a `Box<dyn Activity>`.

Some of the reasons for these changes are outlined in the following sections.

## `Automaton` now stores `Mode`s in place 

In version `^0.2`, the `Mode` trait was complicated by a number of factors:

 - Boxing of individual `Mode`s was handled behind the scenes by `Automaton`, so an `'a` lifetime was necessary
   everywhere to allow `Mode`s to reference data outside the `Automaton`.
 - Each `Mode` needed to have a `Base` associated type, so that it could be limited to swapping in *only* other `Mode`s
   with the same `Base`.
 - However, the connection between `Mode` and its `Base` type was *very* weak. Each `Mode` implementation was required
   to provide boilerplate `as_base()` and `as_base_mut()` functions capable of converting the `Mode` to a `Base`
   reference.

All of this complexity has been obviated by redesigning `Automaton` such that it *always* stores `Mode`s in-place. In
situations where the `Automaton` needs to switch between different `struct`s that implement some common `trait`, it can
be made to store a `Box<dyn Trait>`. As long as the `trait` in question extends `boxed::Mode`, things will just work. In
fact, `Automaton` now supports a couple of pointer types other than `Box`, specifically `std::rc::Rc` and
`std::sync::Arc`, with each of these having a corresponding `trait` (`rc::Mode` and `sync::Mode`, respectively.) With
this new design, any necessary lifetimes can be added at the `Family` level by the implementor, making an explicit `'a`
lifetime on `Automaton` no longer necessary.

## `Transition` is no more

The `Transition` system added some substantial complexity to `Automaton`. Swapping between `Mode`s required a boxed
closure (a.k.a. `TransitionBox`) to be returned from `Mode::get_transition()`, like so:

```rust
fn get_transition(&mut self) -> Option<TransitionBox<'a, Self>> {
    if self.hours_rested >= 8 {
        println!("Time for breakfast!");
        Some(Box::new(|_| { Eating { hours_worked: 0, calories_consumed: 0 } })) // Closure that swaps to a new state.
    }
    else { None } // None means stay in the current state.
}
```

The closure itself was responsible for returning a new `Mode` to be swapped in after it was invoked. The only reason
that this was necessary was so that the closure could consume the current `Mode`, if it wanted to, after
`get_transition()` was done borrowing it. Not only did this system require an allocation for the `TransitionBox` each
time the current `Mode` transitioned, but it also placed a cognitive burden on anyone trying to consume the library in
their own project. Users needed to understand that returning `None` from `Mode::get_transition()` meant "don't
transition," and that even when a `Transition` closure *was* returned, it was perfectly legal for the closure to swap in
the same `Mode` that was passed to it. Hence, a `true` result from `Automaton::perform_transitions()` did not
*necessarily* indicate that a `Transition` to a new `Mode` had occurred, which could be confusing.

All of this has been greatly simplified in version `0.3`. Instead of returning a closure, `swap()` *always* consumes the
current `Mode`, and returns another `Mode` with the same `Family` type in its place. If the current `Mode` wants to
remain active, it simply returns `self` from the `swap()` function. In cases where moving `self` around would be too
wasteful, `swap()` can be given a pointer `self` type instead, e.g. `self : Box<Self>`, in order to make transitioning
more efficient. All of this is *much* more intuitive than the closure-based `Transition` system of `^0.2`.

For `Mode` implementations that only need to swap to other `Mode`s of the same type, the `Mode::swap()` function now
looks something like this. (See the example below, taken from `examples/enum.rs`.)

```rust
impl Mode for Activity {
    type Family = ActivityFamily;
    fn swap(self, _input : ()) -> Self {
        match self {
            // ...
            Activity::Sleeping { hours_rested } => {
                if hours_rested >= 8 {
                    println!("Time for breakfast!");
                    Activity::Eating { hours_worked: 0, calories_consumed: 0 } // Swap to a new state.
                }
                else { self } // Stay in the current state.
            },
        }
    }
}
```

For `Mode` implementations that are stored via some `dyn Trait` pointer and can swap other `struct`s that impl the same
trait, the `Mode::swap()` function now looks something like this, depending on which pointer type is used. (See example
below.)

```rust
impl boxed::Mode for Sleeping {
    type Family = ActivityFamily;
    fn swap(self : Box<Self>, _input : ()) -> Box<dyn Activity> {
        if self.hours_rested >= 8 {
            println!("Time for breakfast!");
            Box::new(Eating { hours_worked: 0, calories_consumed: 0 }) // Swap to a new state.
        }
        else { self } // Stay in the current state.
    }
}
```

## `Mode::Base` has been replaced by the new `Family` associated type

As was briefly mentioned earlier, the `Mode::Base` associated type has been replaced by a new one called `Family`:

```rust
impl boxed::Mode for Working {
    type Family = ActivityFamily; // ...
}
```

If you look at the definition of `ActivityFamily` in the Activity example, you will see something like this:

```rust
struct ActivityFamily;

impl Family for ActivityFamily {
    type Base = dyn Activity;
    type Mode = Box<dyn Activity>;
    type Input = ();
    type Output = Box<dyn Activity>;
}
```

As you can see, the `Family` trait has no functions. All it does is define associated types for some set of `Mode`s that
can all be used together in the same `Automaton`.

In the example above, `ActivityFamily` is a unit `struct` that will never be constructed. Its only purpose is to group
together a set of states that will be used in the same `Automaton`. This actually fixes a design flaw in earlier
versions of `mode`. In `^0.2`, a `Mode` in one state machine could technically switch to a `Mode` in an unrelated state
machine, so long as both `Mode`s had the same `Base` type. In `0.3`, two `Mode`s with the same `Base` can be given
separate `Family` types if they are conceptually unrelated, and the compiler will ensure that each `Mode`'s
implementation of `swap()` is *only* able to return a `Mode` in the same `Family`.

### `Base`

The `Base` type defines the common interface for all `Mode`s in the same `Family`. In this way, it is equivalent to the
`Base` associated type of `^0.2`, except that it is defined for a whole `Family` of `Mode`s, rather than separately for
each `Mode` implementation. `Base` can either be a concrete type, such as a `struct` or an `enum`, or a `dyn Trait`.
When a concrete type is used, e.g.

```rust
type Base = Activity; // where Activity is a Sized type
```

all states in the `Family` must be of this *same* concrete type. Hence, any `Automaton` over this `Family` of states
will *only* be allowed to switch between states of this exact same type.

By contrast, if a `dyn Trait` is specified for `Base`, e.g.

```rust
type Base = dyn Activity;
```

each state in the `Family` is allowed to be of a *different* type, so long as each state implements the common `Base`
trait (`dyn Activity`, in this case.) Please note that, in order to do this, the trait itself *must* extend one of the
pointer-based `Mode` traits included with this library, e.g.

```rust
trait Activity : boxed::Mode<Family = ActivityFamily> {
    // ...
}
```

This is because the trait object is unsized, and therefore *must* be wrapped with a pointer object of some kind. Hence,
`swap()` must operate on a *pointer* to the current `Mode`, as opposed to the unsized type. (See the sections below for
more details.)

### `Mode`

The `Mode` type specifies how the current state will be *stored* in the `Automaton`. In the Activity example, for
instance, we want the `Automaton<ActivityFamily>` to be able to switch between different `struct`s that implement the
`Activity` trait. Since `Activity` is an unsized type, we need the current state in the `Automaton` to be stored by
pointer. Hence, we specify `Box<dyn Activity>` for `ActivityFamily::Mode`.

```rust
type Mode = Box<dyn Activity>;
```

There is a blanket `Mode` implementation for `Box<M>` where `M : boxed::Mode`, so specifying a `Box` type for
`ActivityFamily::Mode` like this is legal, so long as the inner type implements `boxed::Mode`. The only real difference
between the standard `Mode` trait and `boxed::Mode` is that the `swap()` function for `boxed::Mode` takes
`self : Box<Self>`, allowing the whole `Box` to be moved into and out of the function.

As mentioned earlier, other pointer types are supported, as well, such as `std::rc::Rc` and `std::sync::Arc`. Each of
these requires a different `Mode` trait to be implemented by the inner type, with a slightly different `swap()` function
signature, e.g. `rc::Mode::swap()` takes `self : Rc<Self>`, while `sync::Mode::swap()` takes `self : Arc<Self>`.

### `Input`

One new feature of `0.3` is that the new `swap()` function takes an `input` parameter that can optionally be used to
pass some context into the function, when necessary, to allow the current `Mode` to transition itself. (See example
below.)

```rust
fn swap(self : Box<Self>, input : i32) -> Box<dyn SomeTrait> {
    if input > 0 {
        Box::new(SomeMode::new()) // Swap in a new Mode.
    }
    else { self }
}
```

This wasn't possible with `get_transition()` before. If you'd like to see how the `input` parameter can be used, please
see the [Turing](./examples/turing.rs) example.

### `Output`

The `Output` type defines the return type of the `swap()` function.

If `Activity` was some concrete type, like an `enum`, `Family` would look something like this:

```rust
enum Activity { Working, Eating, Sleeping } // TODO: impl Mode

struct ActivityFamily;

impl Family for ActivityFamily {
    type Base = Activity; // Expose the entire enum via the Automaton.
    type Mode = Activity; // Store the enum in the Automaton in-place.
    type Input = (); // Don't pass any context into the swap() function.
    type Output = Activity; // Return a new Activity from the swap() function.
}
```

Setting `Output` to `Activity` like this means that the `swap()` function for all `Mode`s in this `Family` will return
an `Activity` to be swapped in whenever it is called. However, what if we wanted to pass some information back to the
caller of `Automaton::next()` to notify them of what happened as a result of the `swap()`? We can do this by specifying
a *tuple* type for `Output`, e.g.

```rust
type Output = (Activity, bool); // Return a new Activity from the swap() function, along with a bool return type.
```

When this convention is used, the caller must use `Automaton::next_with_output()` or
`Automaton::next_with_input_and_output()` to swap the current `Mode`. The first tuple element is assumed to be the new
`Mode` to swap in, and therefore *must* be an object of type `<Self as Family>::Mode`. The second element can be any
`Sized` type, which will be returned from `Automaton::next_with_output()` or `Automaton::next_with_input_and_output()`.
In the `(Activity, bool)` example above, we could have the `bool` represent that the current `Mode` changed as a result
of the `swap()`, conveniently allowing us to write code like this:

```rust
if Automaton::next_with_output(&mut person) {
    println!("Activity changed!");
}
```

Alternatively, we could return a `Result` from `swap()`, e.g.

```rust
type Output = (Activity, Result<(), SomeError>);
```

allowing us to catch any `Error` that occured while transitioning:

```rust
if let Err(error) = Automaton::next_with_output(&mut person) {
    println!("Error: {:?}", error);
}
```

**Note:** These are just two common use cases. The return value is very flexible, and can be used in a variety of other
ways, depending on the convention that is desired.

## `Automaton` now provides more flexibility when swapping the current `Mode`

Previously, using an `Automaton` looked something like this:

```rust
fn main() {
    let mut person = Automaton::with_initial_mode(Working { hours_worked: 0 });
    
    for _age in 18..100 {
        person.update();
        person.perform_transitions();
    }
}
```

With `0.3`, this changes to the following:

```rust
fn main() {
    let mut person = ActivityFamily::automaton_with_mode(Box::new(Working { hours_worked: 0 }));
    
    for _age in 18..100 {
        person.update();
        Automaton::next(&mut person);
    }
}
```

In `^0.2`, the `Automaton::perform_transitions()` function took a `&mut self`, since the function name was obscure
enough that it was unlikely to conflict with any functions on the inner `Family::Mode` type exposed via `Deref`
coercion. This function has been replaced by several associated functions that take `&mut Automaton` instead:

|Name                                     |Description                                                                |
|-----------------------------------------|---------------------------------------------------------------------------|
|`Automaton::next()`                      |Swaps the current `Mode`.                                                  |
|`Automaton::next_with_output()`          |Swaps the current `Mode` and returns a value.                              |
|`Automaton::next_with_input()`           |Takes an `input` parameter and swaps the current `Mode`.                   |
|`Automaton::next_with_input_and_output()`|Takes an `input` parameter and swaps the current `Mode`, returning a value.|

As hinted at in the [previous section](#output), the correct function **must** be used to swap the current `Mode`,
depending on the values of `Family::Input` and `Family::Output` being used. Having these variants allows different
`input` and return types to be used with the `Mode::swap()` function, without relying on macro magic or typecasting to
accomplish this.

# Conclusion

There's a lot more that could be said about the new version in this guide, but at this point it's probably best to start
reading over the documentation and looking at the examples, to get a feel for the library. If you're looking to update
your own project, also give [UPGRADING-v0.3.md](UPGRADING-v0.3.md) a read, if you haven't already. Hopefully, you will
find version `0.3` of `mode` to be more intuitive and easier to work with, compared to previous versions of the library.

Enjoy!