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

/// `Id<Entity, Repr>` provides a type-safe way to keep ids of
/// entities. Note that there's no default for `Repr` type, the type
/// of the identifier should be always provided explicitly.
///
/// Example:
///
/// ```
/// use phantom_newtype::Id;
///
/// struct User {
///   id: Id<User, u64>,
///   name: String,
///   posts: Vec<Id<Post, u64>>,
/// }
///
/// struct Post {
///   id: Id<Post, u64>,
///   title: String,
/// }
/// ```
///
/// `Enity` doesn't have to be a struct, any type will do. It's just a
/// marker that differentiate incompatible ids.
///
/// ```compile_fail
/// use phantom_newtype::Id;
///
/// enum Recepient {}
/// enum Message {}
///
/// type RecepientId = Id<Recepient, u64>;
/// type MessageId = Id<Message, u64>;
///
/// assert_eq!(RecepientId::from(15), MessageId::from(15));
/// ```
///
/// `Id` is cheap to copy if `Repr` is:
///
/// ```
/// use phantom_newtype::Id;
///
/// enum Message {}
/// type MessageId = Id<Message, u64>;
///
/// let x = MessageId::from(5);
/// let y = x;
/// assert_eq!(x, y);
/// ```
///
/// `Id` can be used as a key in a hash map as long as `Repr` has
/// this property:
///
/// ```
/// use phantom_newtype::Id;
/// use std::collections::HashMap;
///
/// #[derive(PartialEq, Debug)]
/// struct User {}
/// type UserId = Id<User, String>;
///
/// let mut users_by_id = HashMap::new();
/// let id = UserId::from("john".to_string());
/// users_by_id.insert(id.clone(), User {});
///
/// assert!(users_by_id.get(&id).is_some());
/// ```
///
/// Ids are ordered if the `Repr` is. Note that this is mostly useful
/// e.g. for storing Ids in a `BTreeMap`, there is usually little
/// semantic value in comparing ids.
///
/// ```
/// use std::collections::BTreeMap;
/// use phantom_newtype::Id;
///
/// #[derive(PartialEq, Debug)]
/// struct User {}
/// type UserId = Id<User, u64>;
///
/// let mut map = BTreeMap::new();
/// let id = UserId::from(5);
/// map.insert(id.clone(), User {});
///
/// assert!(map.get(&id).is_some());
/// ```
///
/// Ids can be sent between threads if the `Repr` allows it, no
/// matter which `Entity` is used.
///
/// ```
/// use phantom_newtype::Id;
///
/// type Cell = std::cell::RefCell<i64>;
/// type CellId = Id<Cell, i64>;
/// const ID: CellId = CellId::new(42);
///
/// let id_from_thread = std::thread::spawn(|| &ID).join().unwrap();
/// assert_eq!(ID, *id_from_thread);
/// ```
///
/// Ids can be serialized and deserialized with `serde`. Serialized
/// forms of `Id<Entity, Repr>` and `Repr` are identical.
///
/// ```
/// #[cfg(feature = "serde")] {
/// use phantom_newtype::Id;
/// use serde::{Serialize, Deserialize};
/// use serde_json;
/// enum User {}
///
/// let repr: u64 = 10;
/// let user_id = Id::<User, u64>::from(repr);
/// assert_eq!(serde_json::to_string(&user_id).unwrap(), serde_json::to_string(&repr).unwrap());
/// }
/// ```
#[repr(transparent)]
pub struct Id<Entity, Repr>(Repr, PhantomData<std::sync::Mutex<Entity>>);

impl<Entity, Repr> Id<Entity, Repr> {
    /// `get` returns the underlying representation of the identifier.
    ///
    /// ```
    /// use phantom_newtype::Id;
    ///
    /// enum User {}
    /// type UserId = Id<User, u64>;
    ///
    /// assert_eq!(*UserId::from(15).get(), 15);
    /// ```
    pub const fn get(&self) -> &Repr {
        &self.0
    }

    /// `new` is a synonym for `from` that can be evaluated in
    /// compile time. The main use-case of this functions is defining
    /// constants:
    ///
    /// ```
    /// use phantom_newtype::Id;
    /// enum User {}
    /// type UserId = Id<User, u64>;
    ///
    /// const ADMIN_ID: UserId = UserId::new(42);
    ///
    /// assert_eq!(*ADMIN_ID.get(), 42);
    /// ```
    pub const fn new(repr: Repr) -> Id<Entity, Repr> {
        Id(repr, PhantomData)
    }
}

impl<Entity, Repr> Id<Entity, Repr>
where
    Entity: DisplayerOf<Id<Entity, Repr>>,
{
    /// `display` provides a machanism to implement a custom display
    /// for phantom types.
    ///
    /// ```
    /// use phantom_newtype::{Id, DisplayerOf};
    /// use std::fmt;
    ///
    /// enum Message {}
    /// type MessageId = Id<Message, [u8; 32]>;
    ///
    /// impl DisplayerOf<MessageId> for Message {
    ///   fn display(id: &MessageId, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    ///     id.get().iter().try_for_each(|b| write!(f, "{:02x}", b))
    ///   }
    /// }
    ///
    /// let vec: Vec<_> = (0u8..32u8).collect();
    /// let mut arr: [u8; 32] = [0u8; 32];
    /// (&mut arr[..]).copy_from_slice(&vec[..]);
    ///
    /// assert_eq!(format!("{}", MessageId::from(arr).display()),
    ///            "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f");
    /// ```
    pub fn display(&self) -> DisplayProxy<'_, Self, Entity> {
        DisplayProxy::new(self)
    }
}

impl<Entity, Repr: Clone> Clone for Id<Entity, Repr> {
    fn clone(&self) -> Self {
        Self::from(self.get().clone())
    }
}

impl<Entity, Repr: Copy> Copy for Id<Entity, Repr> {}

impl<Entity, Repr: PartialEq> PartialEq for Id<Entity, Repr> {
    fn eq(&self, rhs: &Self) -> bool {
        self.get().eq(&rhs.get())
    }
}

impl<Entity, Repr: PartialOrd> PartialOrd for Id<Entity, Repr> {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        self.get().partial_cmp(&rhs.get())
    }
}

impl<Entity, Repr: Ord> Ord for Id<Entity, Repr> {
    fn cmp(&self, rhs: &Self) -> Ordering {
        self.get().cmp(&rhs.get())
    }
}

impl<Entity, Repr: Hash> Hash for Id<Entity, Repr> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get().hash(state)
    }
}

impl<Entity, Repr> From<Repr> for Id<Entity, Repr> {
    fn from(repr: Repr) -> Self {
        Self::new(repr)
    }
}

impl<Entity, Repr: Eq> Eq for Id<Entity, Repr> {}

impl<Entity, Repr: fmt::Debug> fmt::Debug for Id<Entity, Repr> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.get())
    }
}

impl<Entity, Repr: fmt::Display> fmt::Display for Id<Entity, Repr> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.get())
    }
}

#[cfg(feature="serde")]
impl<Entity, Repr> Serialize for Id<Entity, Repr>
where
    Repr: Serialize,
{
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.get().serialize(serializer)
    }
}

#[cfg(feature="serde")]
impl<'de, Entity, Repr> Deserialize<'de> for Id<Entity, Repr>
where
    Repr: Deserialize<'de>,
{
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Repr::deserialize(deserializer).map(Id::<Entity, Repr>::from)
    }
}
