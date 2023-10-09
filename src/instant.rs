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

use crate::amount::Amount;
use crate::displayer::{DisplayProxy, DisplayerOf};
#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::ops::{Add, AddAssign, Div, Mul, MulAssign, Sub, SubAssign};

/// `Instant<Unit>` provides a type-safe way to keep absolute time of
/// some events, expressed in `Unit`s (CPU ticks, seconds from epoch,
/// years from birth, etc).
///
/// You can compare instants:
///
/// ```
/// use phantom_newtype::Instant;
///
/// enum SecondsFromEpoch {}
/// type UnixTime = Instant<SecondsFromEpoch, i64>;
///
/// assert_eq!(true, UnixTime::from(3) < UnixTime::from(5));
/// assert_eq!(false, UnixTime::from(3) > UnixTime::from(5));
/// assert_eq!(true, UnixTime::from(3) != UnixTime::from(5));
/// assert_eq!(true, UnixTime::from(5) == UnixTime::from(5));
/// assert_eq!(false, UnixTime::from(5) != UnixTime::from(5));
///
/// assert_eq!(vec![UnixTime::from(3), UnixTime::from(5)].iter().max().unwrap(),
///            &UnixTime::from(5));
/// ```
///
/// Instants support basic arithmetics, you can:
/// * Subtract an instant from another instant to get amount of units between them.
/// * Add/subtract amount of units to/from an instant to get another instant.
///
/// ```
/// use phantom_newtype::{Amount, Instant};
///
/// enum SecondsFromEpoch {}
///
/// type UnixTime = Instant<SecondsFromEpoch, i64>;
/// type TimeDiff = Amount<SecondsFromEpoch, i64>;
///
/// let epoch = UnixTime::from(0);
/// let some_date = UnixTime::from(123456789);
/// let diff = TimeDiff::from(123456789);
///
/// assert_eq!(some_date - epoch, diff);
/// assert_eq!(some_date - diff, epoch);
/// assert_eq!(epoch + diff, some_date);
/// ```
///
/// Direct multiplication of instants is not supported, however, you
/// can scale them by a scalar or divide to get a scalar back:
///
/// ```
/// use phantom_newtype::Instant;
///
/// enum SecondsFromEpoch {}
/// type UnixTime = Instant<SecondsFromEpoch, i64>;
///
/// let x = UnixTime::from(123456);
/// assert_eq!(x * 3, UnixTime::from(3 * 123456));
/// assert_eq!(1, x / x);
/// assert_eq!(3, (x * 3) / x);
/// ```
///
/// Note that the unit is only available at compile time, thus using
/// `Instant` instead of `u64` doesn't incur any runtime penalty:
///
/// ```
/// use phantom_newtype::Instant;
///
/// enum SecondsFromEpoch {}
///
/// let ms = Instant::<SecondsFromEpoch, u64>::from(10);
/// assert_eq!(std::mem::size_of_val(&ms), std::mem::size_of::<u64>());
/// ```
///
/// Instants can be serialized and deserialized with `serde`. Serialized
/// forms of `Instant<Unit, Repr>` and `Repr` are identical.
///
/// ```
/// #[cfg(feature = "serde")] {
/// use phantom_newtype::Instant;
/// use serde::{Serialize, Deserialize};
/// use serde_json;
///
/// enum SecondsFromEpoch {}
/// type UnixTime = Instant<SecondsFromEpoch, i64>;
///
/// let repr: u64 = 123456;
/// let time = UnixTime::from(repr);
/// assert_eq!(serde_json::to_string(&time).unwrap(), serde_json::to_string(&repr).unwrap());
///
/// let copy: UnitTime = serde_json::from_str(&serde_json::to_string(&time).unwrap()).unwrap();
/// assert_eq!(copy, time);
/// }
/// ```
///
/// You can also declare constants of `Instant<Unit, Repr>` using `new`
/// function:
/// ```
/// use phantom_newtype::Instant;
///
/// enum SecondsFromEpoch {}
/// type UnixTime = Instant<SecondsFromEpoch, u64>;
///
/// const EPOCH: UnixTime = UnixTime::new(0);
/// ```
///
/// Instants can be sent between threads if the `Repr` allows it, no
/// matter which `Unit` is used.
///
/// ```
/// use phantom_newtype::Instant;
///
/// type Cell = std::cell::RefCell<i64>;
/// type CellInstant = Instant<Cell, i64>;
/// const I: CellInstant = CellInstant::new(1234);
///
/// let instant_from_thread = std::thread::spawn(|| &I).join().unwrap();
/// assert_eq!(I, *instant_from_thread);
/// ```
#[repr(transparent)]
pub struct Instant<Unit, Repr>(Repr, PhantomData<std::sync::Mutex<Unit>>);

impl<Unit, Repr: Copy> Instant<Unit, Repr> {
    /// Returns the wrapped value.
    ///
    /// ```
    /// use phantom_newtype::Instant;
    ///
    /// enum Apples {}
    ///
    /// let three_apples = Instant::<Apples, u64>::from(3);
    /// assert_eq!(9, (three_apples * 3).get());
    /// ```
    pub fn get(&self) -> Repr {
        self.0
    }
}

impl<Unit, Repr> Instant<Unit, Repr> {
    /// `new` is a synonym for `from` that can be evaluated in
    /// compile time. The main use-case of this functions is defining
    /// constants.
    pub const fn new(repr: Repr) -> Instant<Unit, Repr> {
        Instant(repr, PhantomData)
    }
}

impl<Unit: Default, Repr: Copy> Instant<Unit, Repr> {
    /// Provides a useful shortcut to access units of an instant if
    /// they implement the `Default` trait:
    ///
    /// ```
    /// use phantom_newtype::Instant;
    ///
    /// #[derive(Debug, Default)]
    /// struct SecondsFromEpoch;
    /// let when = Instant::<SecondsFromEpoch, i64>::from(5);
    ///
    /// assert_eq!("5 SecondsFromEpoch", format!("{} {:?}", when, when.unit()));
    /// ```
    pub fn unit(&self) -> Unit {
        Default::default()
    }
}

impl<Unit, Repr> Instant<Unit, Repr>
where
    Unit: DisplayerOf<Instant<Unit, Repr>>,
{
    /// `display` provides a machanism to implement a custom display
    /// for phantom types.
    ///
    /// ```
    /// use phantom_newtype::{Instant, DisplayerOf};
    /// use std::fmt;
    ///
    /// struct YearUnit;
    /// type YearAD = Instant<YearUnit, u64>;
    ///
    /// impl DisplayerOf<YearAD> for YearUnit {
    ///   fn display(year: &YearAD, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    ///     write!(f, "{} AD", year.get())
    ///   }
    /// }
    ///
    /// assert_eq!(format!("{}", YearAD::from(1221).display()), "1221 AD");
    /// ```
    pub fn display(&self) -> DisplayProxy<'_, Self, Unit> {
        DisplayProxy::new(self)
    }
}

impl<Unit, Repr: Copy> From<Repr> for Instant<Unit, Repr> {
    fn from(repr: Repr) -> Self {
        Self::new(repr)
    }
}

impl<Unit, Repr: Copy> Clone for Instant<Unit, Repr> {
    fn clone(&self) -> Self {
        Instant(self.0, PhantomData)
    }
}

impl<Unit, Repr: Copy> Copy for Instant<Unit, Repr> {}

impl<Unit, Repr: PartialEq> PartialEq for Instant<Unit, Repr> {
    fn eq(&self, rhs: &Self) -> bool {
        self.0.eq(&rhs.0)
    }
}

impl<Unit, Repr: Eq> Eq for Instant<Unit, Repr> {}

impl<Unit, Repr: PartialOrd> PartialOrd for Instant<Unit, Repr> {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&rhs.0)
    }
}

impl<Unit, Repr: Ord> Ord for Instant<Unit, Repr> {
    fn cmp(&self, rhs: &Self) -> Ordering {
        self.0.cmp(&rhs.0)
    }
}

impl<Unit, Repr: Hash> Hash for Instant<Unit, Repr> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

impl<Unit, Repr, Repr2> Add<Amount<Unit, Repr2>> for Instant<Unit, Repr>
where
    Repr: AddAssign<Repr2> + Copy,
    Repr2: Copy,
{
    type Output = Self;
    fn add(mut self, rhs: Amount<Unit, Repr2>) -> Self {
        self.add_assign(rhs);
        self
    }
}

impl<Unit, Repr, Repr2> AddAssign<Amount<Unit, Repr2>> for Instant<Unit, Repr>
where
    Repr: AddAssign<Repr2> + Copy,
    Repr2: Copy,
{
    fn add_assign(&mut self, rhs: Amount<Unit, Repr2>) {
        self.0 += rhs.get()
    }
}

impl<Unit, Repr, Repr2> SubAssign<Amount<Unit, Repr2>> for Instant<Unit, Repr>
where
    Repr: SubAssign<Repr2> + Copy,
    Repr2: Copy,
{
    fn sub_assign(&mut self, rhs: Amount<Unit, Repr2>) {
        self.0 -= rhs.get()
    }
}

impl<Unit, Repr> Sub for Instant<Unit, Repr>
where
    Repr: Sub + Copy,
{
    type Output = Amount<Unit, <Repr as Sub>::Output>;

    fn sub(self, rhs: Self) -> Self::Output {
        Amount::<Unit, <Repr as Sub>::Output>::new(self.0 - rhs.0)
    }
}

impl<Unit, Repr, Repr2> Sub<Amount<Unit, Repr2>> for Instant<Unit, Repr>
where
    Repr: SubAssign<Repr2> + Copy,
    Repr2: Copy,
{
    type Output = Self;

    fn sub(mut self, rhs: Amount<Unit, Repr2>) -> Self {
        self.sub_assign(rhs);
        self
    }
}

impl<Unit, Repr> MulAssign<Repr> for Instant<Unit, Repr>
where
    Repr: MulAssign + Copy,
{
    fn mul_assign(&mut self, rhs: Repr) {
        self.0 *= rhs;
    }
}

impl<Unit, Repr> Mul<Repr> for Instant<Unit, Repr>
where
    Repr: MulAssign + Copy,
{
    type Output = Self;

    fn mul(mut self, rhs: Repr) -> Self {
        self.mul_assign(rhs);
        self
    }
}

impl<Unit, Repr> Div<Self> for Instant<Unit, Repr>
where
    Repr: Div<Repr> + Copy,
{
    type Output = <Repr as Div>::Output;

    fn div(self, rhs: Self) -> Self::Output {
        self.0.div(rhs.0)
    }
}

impl<Unit, Repr> fmt::Debug for Instant<Unit, Repr>
where
    Repr: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<Unit, Repr> fmt::Display for Instant<Unit, Repr>
where
    Repr: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(feature = "serde")]
impl<Unit, Repr: Serialize> Serialize for Instant<Unit, Repr> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de, Unit, Repr> Deserialize<'de> for Instant<Unit, Repr>
where
    Repr: Deserialize<'de>,
{
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Repr::deserialize(deserializer).map(Instant::<Unit, Repr>::new)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complex_instant_arithmetics() {
        enum Seconds {}
        enum UTC {}

        type Timestamp = Instant<Seconds, i64>;
        type TsDiff = Amount<Seconds, i64>;
        type Date = Instant<UTC, Timestamp>;

        let epoch = Date::new(Timestamp::new(0));
        let date = Date::new(Timestamp::new(123456789));
        let span = Amount::<UTC, TsDiff>::new(TsDiff::from(123456789));

        assert_eq!(date - epoch, span);
        assert_eq!(date - span, epoch);
        assert_eq!(epoch + span, date);
    }
}
