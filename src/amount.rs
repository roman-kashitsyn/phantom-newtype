// Copyright 2019 DFINITY
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::displayer::{DisplayProxy, DisplayerOf};
#[cfg(feature="serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::ops::{Add, AddAssign, Div, Mul, MulAssign, Sub, SubAssign};

/// `Amount<Unit>` provides a type-safe way to keep an amount of
/// some `Unit`.
///
///  E.g. the following code must not compile:
///
/// ```compile_fail
/// use phantom_newtype::Amount;
///
/// // These structs are just tags and have no semantic meaning.
/// struct Apples {}
/// struct Oranges {}
///
/// let trois_pommes = Amount::<Apples, u64>::from(3);
/// let five_oranges = Amount::<Oranges, u64>::from(5);
///
/// assert_eq!(8, (trois_pommes + five_oranges).get())
/// ```
///
/// `Amount<Unit, Repr>` defines common boilerplate to make type-safe
/// amounts more convenient.  For example, you can compare amounts:
///
/// ```
/// use phantom_newtype::Amount;
///
/// struct Apples {}
/// type NumApples = Amount<Apples, u64>;
///
/// assert_eq!(true, NumApples::from(3) < NumApples::from(5));
/// assert_eq!(false, NumApples::from(3) > NumApples::from(5));
/// assert_eq!(true, NumApples::from(3) != NumApples::from(5));
/// assert_eq!(true, NumApples::from(5) == NumApples::from(5));
/// assert_eq!(false, NumApples::from(5) != NumApples::from(5));
///
/// assert_eq!(vec![NumApples::from(3), NumApples::from(5)].iter().max().unwrap(),
///            &NumApples::from(5));
/// ```
///
/// You can do simple arithmetics with amounts:
///
/// ```
/// use phantom_newtype::Amount;
///
/// struct Apples {}
/// struct Oranges {}
///
/// let x = Amount::<Apples, u64>::from(3);
/// let y = Amount::<Oranges, u64>::from(5);
///
/// assert_eq!(x + x, Amount::<Apples, u64>::from(6));
/// assert_eq!(y - y, Amount::<Oranges, u64>::from(0));
/// ```
///
/// Multiplication of amounts is not supported: multiplying meters by
/// meters gives square meters. However, you can scale an amount by a
/// scalar or divide amounts:
///
/// ```
/// use phantom_newtype::Amount;
///
/// struct Apples {}
///
/// let x = Amount::<Apples, u64>::from(3);
/// assert_eq!(x * 3, Amount::<Apples, u64>::from(9));
/// assert_eq!(1, x / x);
/// assert_eq!(3, (x * 3) / x);
/// ```
///
/// Note that the unit is only available at compile time, thus using
/// `Amount` instead of `u64` doesn't incur any runtime penalty:
///
/// ```
/// use phantom_newtype::Amount;
///
/// struct Meters {}
///
/// let ms = Amount::<Meters, u64>::from(10);
/// assert_eq!(std::mem::size_of_val(&ms), std::mem::size_of::<u64>());
/// ```
///
/// Amounts can be serialized and deserialized with `serde`. Serialized
/// forms of `Amount<Unit, Repr>` and `Repr` are identical.
///
/// ```
/// #[cfg(feature = "serde")] {
/// use phantom_newtype::Amount;
/// use serde::{Serialize, Deserialize};
/// use serde_json;
/// struct Meters {}
///
/// let repr: u64 = 10;
/// let m_10 = Amount::<Meters, u64>::from(repr);
/// assert_eq!(serde_json::to_string(&m_10).unwrap(), serde_json::to_string(&repr).unwrap());
///
/// let copy: Amount<Meters, u64> = serde_json::from_str(&serde_json::to_string(&m_10).unwrap()).unwrap();
/// assert_eq!(copy, m_10);
/// }
/// ```
///
/// You can also declare constants of `Amount<Unit, Repr>` using `new`
/// function:
/// ```
/// use phantom_newtype::Amount;
/// struct Meters {}
/// type Distance = Amount<Meters, u64>;
/// const ASTRONOMICAL_UNIT: Distance = Distance::new(149_597_870_700);
///
/// assert!(ASTRONOMICAL_UNIT > Distance::from(0));
/// ```
///
/// Amounts can be sent between threads if the `Repr` allows it, no
/// matter which `Unit` is used.
///
/// ```
/// use phantom_newtype::Amount;
///
/// type Cell = std::cell::RefCell<i64>;
/// type NumCells = Amount<Cell, i64>;
/// const N: NumCells = NumCells::new(1);
///
/// let n_from_thread = std::thread::spawn(|| &N).join().unwrap();
/// assert_eq!(N, *n_from_thread);
/// ```
pub struct Amount<Unit, Repr>(Repr, PhantomData<std::sync::Mutex<Unit>>);

impl<Unit, Repr: Copy> Amount<Unit, Repr> {
    /// Returns the wrapped value.
    ///
    /// ```
    /// use phantom_newtype::Amount;
    ///
    /// struct Apples {}
    ///
    /// let three_apples = Amount::<Apples, u64>::from(3);
    /// assert_eq!(9, (three_apples * 3).get());
    /// ```
    pub fn get(&self) -> Repr {
        self.0
    }
}

impl<Unit, Repr> Amount<Unit, Repr> {
    /// `new` is a synonym for `from` that can be evaluated in
    /// compile time. The main use-case of this functions is defining
    /// constants.
    pub const fn new(repr: Repr) -> Amount<Unit, Repr> {
        Amount(repr, PhantomData)
    }
}

impl<Unit: Default, Repr: Copy> Amount<Unit, Repr> {
    /// Provides a useful shortcut to access units of an amount if
    /// they implement the `Default` trait:
    ///
    /// ```
    /// use phantom_newtype::Amount;
    ///
    /// #[derive(Debug, Default)]
    /// struct Seconds {}
    /// let duration = Amount::<Seconds, u64>::from(5);
    ///
    /// assert_eq!("5 Seconds", format!("{} {:?}", duration, duration.unit()));
    /// ```
    pub fn unit(&self) -> Unit {
        Default::default()
    }
}

impl<Unit, Repr> Amount<Unit, Repr>
where
    Unit: DisplayerOf<Amount<Unit, Repr>>,
{
    /// `display` provides a machanism to implement a custom display
    /// for phantom types.
    ///
    /// ```
    /// use phantom_newtype::{Amount, DisplayerOf};
    /// use std::fmt;
    ///
    /// struct Cents {}
    /// type Money = Amount<Cents, u64>;
    ///
    /// impl DisplayerOf<Money> for Cents {
    ///   fn display(amount: &Money, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    ///     write!(f, "${}.{:02}", amount.get() / 100, amount.get() % 100)
    ///   }
    /// }
    ///
    /// assert_eq!(format!("{}", Money::from(1005).display()), "$10.05");
    /// ```
    pub fn display(&self) -> DisplayProxy<'_, Self, Unit> {
        DisplayProxy::new(self)
    }
}

impl<Unit, Repr: Copy> From<Repr> for Amount<Unit, Repr> {
    fn from(repr: Repr) -> Self {
        Self::new(repr)
    }
}

// Note that we only have to write the boilerplate trait
// implementation below because default implementations of traits put
// unnecessary restrictions on the type parameters. E.g. deriving
// `PartialEq<Wrapper<T>>` require `T` to implement `PartialEq`, which
// is not what we want: `T` is phantom in our case.

impl<Unit, Repr: Copy> Clone for Amount<Unit, Repr> {
    fn clone(&self) -> Self {
        Amount(self.0, PhantomData)
    }
}

impl<Unit, Repr: Copy> Copy for Amount<Unit, Repr> {}

impl<Unit, Repr: PartialEq> PartialEq for Amount<Unit, Repr> {
    fn eq(&self, rhs: &Self) -> bool {
        self.0.eq(&rhs.0)
    }
}

impl<Unit, Repr: Eq> Eq for Amount<Unit, Repr> {}

impl<Unit, Repr: PartialOrd> PartialOrd for Amount<Unit, Repr> {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&rhs.0)
    }
}

impl<Unit, Repr: Ord> Ord for Amount<Unit, Repr> {
    fn cmp(&self, rhs: &Self) -> Ordering {
        self.0.cmp(&rhs.0)
    }
}

impl<Unit, Repr: Hash> Hash for Amount<Unit, Repr> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

impl<Unit, Repr> Add for Amount<Unit, Repr>
where
    Repr: AddAssign + Copy,
{
    type Output = Self;
    fn add(mut self, rhs: Self) -> Self {
        self.add_assign(rhs);
        self
    }
}

impl<Unit, Repr> AddAssign for Amount<Unit, Repr>
where
    Repr: AddAssign + Copy,
{
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.get()
    }
}

impl<Unit, Repr> SubAssign for Amount<Unit, Repr>
where
    Repr: SubAssign + Copy,
{
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.get()
    }
}

impl<Unit, Repr> Sub for Amount<Unit, Repr>
where
    Repr: SubAssign + Copy,
{
    type Output = Self;

    fn sub(mut self, rhs: Self) -> Self {
        self.sub_assign(rhs);
        self
    }
}

impl<Unit, Repr> MulAssign<Repr> for Amount<Unit, Repr>
where
    Repr: MulAssign + Copy,
{
    fn mul_assign(&mut self, rhs: Repr) {
        self.0 *= rhs;
    }
}

impl<Unit, Repr> Mul<Repr> for Amount<Unit, Repr>
where
    Repr: MulAssign + Copy,
{
    type Output = Self;

    fn mul(mut self, rhs: Repr) -> Self {
        self.mul_assign(rhs);
        self
    }
}

impl<Unit, Repr> Div<Self> for Amount<Unit, Repr>
where
    Repr: Div<Repr> + Copy,
{
    type Output = <Repr as Div>::Output;

    fn div(self, rhs: Self) -> Self::Output {
        self.0.div(rhs.0)
    }
}

impl<Unit, Repr> fmt::Debug for Amount<Unit, Repr>
where
    Repr: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl<Unit, Repr> fmt::Display for Amount<Unit, Repr>
where
    Repr: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// Derived serde `impl Serialize` produces an extra `unit` value for
// phantom data, e.g. `Amount::<Meters>::from(10)` is serialized
// into json as `[10, null]` by default.
//
// We want serialization format of `Repr` and the `Amount` to match
// exactly, that's why we have to provide custom instances.
#[cfg(feature="serde")]
impl<Unit, Repr: Serialize> Serialize for Amount<Unit, Repr> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

#[cfg(feature="serde")]
impl<'de, Unit, Repr> Deserialize<'de> for Amount<Unit, Repr>
where
    Repr: Deserialize<'de>,
{
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Repr::deserialize(deserializer).map(Amount::<Unit, Repr>::new)
    }
}
