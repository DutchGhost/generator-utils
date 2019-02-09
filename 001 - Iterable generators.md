# Iterable Generators

Working with generators is something you don't hear alot about in the world of Rust.
However, generators are an important feature,
they are used in the internals for Future's for example.
Another corner where generators might prove themselves usefull is Iterators.

## Generators in a nutshell
A generator is iterable - that is, a generator can _yield_ items.
Yielding can be seen as calling an Iterators `.next()` method.
In order to advance a generator towards the next yield point, `.resume()` is called.
This method returns a `GeneratorState` enum:
```Rust
pub enum GeneratorState<Y, R> {
    Yielded(Y),
    Complete(R),
}
```

The `Yielded` variant is returned when a _yield_ statement is found,
and the `Complete` variant is returned when the generator itself is done, and returns.

You might wonder, "How do we know the types of `Y` and `R`"?
The answer is in the generator trait itself, let's take a look:

```Rust
pub trait Generator {
    type Yield;
    type Return;
    fn resume(self: Pin<&mut Self>) -> GeneratorState<Self::Yield, Self::Return>;
}
```
We see that a Generator has 2 associated types, `Yield` and `Return`, which
are mapped to `Y` and `R` from the GeneratorState enum.

We can also see that in order to advance, we need a `Pin<&mut Self>`. More on that later.

## Iterating over a generator
Now we know a bit more how a generator works, let's try making it Iterable.
The basic idea is that when the generator yields, the iterable can return `Some(Generator::Yield)`,
and when it's complete, `None` is returned.
The implementation would look something like this:

```Rust
impl <G: Generator> Iterator for G {
  type Item = G::Yield;

  fn next(&mut self) -> Option<Self::Item> {
    match self.resume() {
      GeneratorState::Yielded(y) => Some(y),
      // we don't really care to bind the return,
      GeneratorState::Complete(_) => None,
    }
  }
}
```

## Pinning & Panic
We've sketched how the iterator implementation for a generator should look like,
however there are 2 issues with it:
1. The resume function takes a `Pin<&mut Self>`
2. Resume is unconditional

#### Pinning
We know that a generator takes a `Pin<&mut Self>`. "Why?" you might wonder.
There are 2 types of generators, self-referential, and non-self-referential.
A self-referential generator can hold references to local variables across yield points,
while a non-self-referential can not.
This also means, that a self-referential generator can not be moved after the first resume call.
Moving it would invalidate the references.
Here comes pinning into the story. If we can prove we won't move the self-referential generator,
everything is still safe. Once we have a `Pin<&mut Self>`, we can not move Self, unless `Self: Unpin`. `Unpin` is only implemented for types that are non-self-referntial.

#### Panic
Another problem with the sketch above, is that `.resume()` is unconditionally called.
The documentation of the resume function states: "This function may panic if it is called after the `Complete` variant has been returned previously.".
This means, if someone would call our `.next()` method after it has returned `None` once, we'd panic as well.
We can do better than that.

## Back to the drawing board
Now we know we have to deal with pinning, and we dont want to panic.
Lets try this again.
First, we'll create a wrapper type:

```Rust
struct GenIter<G>(Option<G>);
```

We have a struct, that inside it holds an `Option<G>`. The idea being we set the option to None once the generator completes.
This way, we can check if we still hold `Some(G)`, or None, and never call `.resume()` again if the generator completed.

#### Implementing Iterator
Now, the implementation of Iterator would look like this:
```Rust
impl <G: Generator> Iterator for GenIter<G> {
  type Item = G::Yield;

  fn next(&mut self) -> Option<Self::Item> {
    match self
      .0
      .as_mut()
      .map(|generator| unsafe { Pin::new_unchecked(generator) }.resume())
      {
        Some(GeneratorState::Yielded(y)) => Some(y),
        Some(GeneratorState::Complete(_)) => {
          self.0 = None;
          None
        },
        None => None,
      }
  }
}
```

Except we are using `unsafe`, and this implementation is in fact unsafe.
"How did one call to `Pin::new_unchecked` make the whole implementation unsafe?", you might wonder.
The answer lies in the fact that with this implementation, the `.next()` method can be called with an invalid Generator.
Imagine the following steps:
- `.next()` is called,
- the GenIter struct is moved in memory,
- `.next()` is called again.

The second call would be invalid, the struct was moved, and all references are invalidated.
In fact, because we implemented Iterator for the GenIter struct directly, this is possible.
To move, or not to move, thats the question.

#### Implementing Iterator v2
Okey, we can't implement Iterator for our GenIter struct directly, because we can't ensure that the struct isn't moved.
Well...what does ensure you dont move?... A pin!
Lets implement Iterator for `Pin<&mut GenIter<G>>`:

```Rust
impl<G: Generator> Iterator for Pin<&mut GenIter<G>> {
  type Item = G::Yield;

  fn next(&mut self) -> Option<Self::Item> {
      let this: Pin<&mut GenIter<G>> = self.as_mut();

      let gen: Pin<&mut Option<G>> = unsafe { this.map_unchecked_mut(|geniter| &mut geniter.0) };

      let gen: Option<Pin<&mut G>> = Option::as_pin_mut(gen);

      match gen.map(Generator::resume) {
        Some(GeneratorState::Yielded(y)) => Some(y),
        Some(GeneratorState::Complete(_)) => {
            self.set(GenIter(None));
            None
        }
        None => None,
      }
    }
}
```

With this implementation, the caller has to first create a Pin that contains a mutable reference to our GenIter struct, and it's their responsibility to do so correctly.

#### Extending the implementation
One downside of the last implementation is that even if it is safe to create a Pin<&mut GenIter<G>>, it still has to be done by the user.
Here comes `Unpin` into play. If we know that `G: Unpin`, our GenIter struct also implements Unpin.
We could create a safe Iterator implementation on `GenIter<G> where G: Unpin`.

```Rust
impl <G: Generator + Unpin> Iterator for GenIter<G> {
  type Item = G::Yield;

  fn next(&mut self) -> Option<Self::Item> {
    Pin::new(self).next()
  }
}
```
We already have an implementation of Iterator on a `Pin<&mut GenIter<G>>`, so we can just delegate to it!

## Conlusion
It wasn't quite as simple to make generators iterables.
We had to make sure we didn't invalidate the generator, but we also had to make sure we didn't get invalid generators passed into our code.
At the same time, we wanted to be panic-free, and have an easy to use, fast implementation.
But! In the end we did it!
