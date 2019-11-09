phantom-newtype
===============

Lightweight newtypes without macros.

`phantom-newtype` is a library that provides a simple way to define newtypes.
It's not a replacement for the [newtype idiom][1] but rather a nice addition to it that covers common use-cases.

Example:
```rust
// Amount and Id are "kinds" of newtypes people commonly use.
// Let's call them "archetypes".
use phantom_newtype::{Amount, Id, Instant};

// CentUnit here is just a marker that should never be constructed.
// It allows us to forge a new type of amounts.
enum CentUnit {}
type Cents = Amount<CentUnit, u64>;
//                            ^
//                            Representation used for `Cents`.

// Let's create another type of amounts.
// phantom-newtype is not a replacement for a powerful units library though.
enum YearUnit {}
type Years = Amount<YearUnit, u64>;

// Instants express the idea of time with respect to some point of reference.
// E.g. year of birth is instant but age is amount.
// Note that it is perfectly fine (and useful in practice) to use the same
// marker type for both instants and corresponding amounts.
type YearAD = Instant<YearUnit, u64>;

// Any type can be used as a marker, it's not necessary to always define
// fresh empty types.
type UserId = Id<User, u64>;
//                     ^
//                     Representation used for `Id`.

struct User {
    id: UserId,
    name: String,
    balance: Cents,
    age: Years,
}

impl User {
  fn new() -> Self {
      Self {
          id: UserId::from(1),
          name: "John".to_string(),
          balance: Cents::from(1000),
          age: Years::from(28),
          member_since: YearAD::from(2016),
      }
  }
}

// Tags used in archetypes can be useful in generic code.
fn load_by_id<EntityType>(id: Id<EntityType, u64>) -> EntityType;
```

## Benefits of using `phantom-newtype`

  1. Very little boilerplate required to define newtypes.
  1. Reusable semantics provided by archetypes.
     Once you get used to, say, amounts and know what they can do, you don't need to spend brainpower to understand other types of amounts in your code.
  1. No macros messing up your namespace.

## Implemented traits

| Trait\Archetype   | `Amount<T, Repr>` | `Id<T, Repr>` | `Instant<T, Repr>` |
|-------------------|:-----------------:|:-------------:|:------------------:|
| `Default`         | ✘                 | ✘             | ✘                  |
| `Clone`           | ✔                 | ✔             | ✔                  |
| `Copy`            | ✔                 | ✔             | ✔                  |
| `Debug`           | ✔                 | ✔             | ✔                  |
| `Display`         | ✔                 | ✔             | ✔                  |
| `Eq`              | ✔                 | ✔             | ✔                  |
| `Ord`             | ✔                 | ✔             | ✔                  |
| `Hash`            | ✔                 | ✔             | ✔                  |
| `From<Repr>`      | ✔                 | ✔             | ✔                  |
| `Add<Self>`       | ✔                 | ✘             | ✘                  |
| `AddAssign<Self>` | ✔                 | ✘             | ✘                  |
| `Sub<Self>`       | ✔                 | ✘             | ✔                  |
| `SubAssign<Self>` | ✔                 | ✘             | ✘                  |
| `Mul<Repr>`       | ✔                 | ✘             | ✔                  |
| `MulAssign<Repr>` | ✔                 | ✘             | ✔                  |
| `Div<Self>`       | ✔                 | ✘             | ✔                  |

## Instants/Amounts arithmetics

| Operation                       | Output type                             |
|---------------------------------|-----------------------------------------|
| `Instant<T, R> - Instant<T, R>` | `Amount<Unit, <R as Sub>::Output>`      |
| `Instant<T, R> - Amount<T, R2>` | `Instant<Unit, <R as Sub<R2>>::Output>` |
| `Instant<T, R> + Amount<T, R2>` | `Instant<Unit, <R as Add<R2>>::Output>` |

## Limitations

The approach taken by the library has some limitations due to design choices made by Rust:

  1. It's impossible to implement additional traits for the types forged using the archetypes provided by `phantom-newtype`.
     Every combination of desired traits requires a new archetype.
  1. It's impossible to customize implementations of traits provided by archetypes.
     With `phantom-newtype`, every newtype inherits implementations of its representation (including `Debug` and `Display`).

[1]: https://doc.rust-lang.org/rust-by-example/generics/new_types.html#new-type-idiom



