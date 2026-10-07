/*
 * Copyright (C) 2026 - Universidad Politécnica de Madrid - UPM
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

use crate::errors::{BadFormat, Errors, Outcome};
use serde::de::value::{MapAccessDeserializer, SeqAccessDeserializer, StrDeserializer};
use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{self, Formatter};
use std::marker::PhantomData;

/// A JSON value that may appear either alone or inside an array, as JSON-LD
/// allows for `@context`, for the verification relationships of a DID
/// Document, and for the `proof` property of a secured document.
///
/// The variant records the form the value was written in, and serialisation
/// reproduces it. This matters because these values sit inside canonicalised,
/// hashed content: promoting a lone value to an array would change the bytes
/// and break the digest.
#[derive(Debug, Clone, PartialEq)]
pub enum OneOrMany<T> {
    One(T),
    Many(Vec<T>),
}

impl<T> OneOrMany<T> {
    /// Both forms seen as a slice. `One` borrows as a slice of length 1, so
    /// every other accessor can ignore the distinction.
    pub fn as_slice(&self) -> &[T] {
        match self {
            OneOrMany::One(v) => std::slice::from_ref(v),
            OneOrMany::Many(v) => v.as_slice(),
        }
    }

    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.as_slice().iter()
    }

    pub fn len(&self) -> usize {
        self.as_slice().len()
    }

    pub fn is_empty(&self) -> bool {
        self.as_slice().is_empty()
    }

    /// The only element, for properties that must carry exactly one.
    ///
    /// # Errors
    /// Returns [`Errors::FormatError`] if the value holds zero or more than
    /// one element.
    pub fn single(&self) -> Outcome<&T> {
        match self.as_slice() {
            [one] => Ok(one),
            other => Err(Errors::format(
                BadFormat::Received,
                format!("Expected exactly one element, found {}", other.len()),
                None,
            )),
        }
    }

    /// Appends a value, promoting the single form to the array form.
    pub fn push(&mut self, value: T) {
        match std::mem::replace(self, OneOrMany::Many(Vec::new())) {
            OneOrMany::One(first) => *self = OneOrMany::Many(vec![first, value]),
            OneOrMany::Many(mut values) => {
                values.push(value);
                *self = OneOrMany::Many(values);
            }
        }
    }

    /// Keeps only the elements matching the predicate.
    pub fn retain(&mut self, f: impl Fn(&T) -> bool) {
        match std::mem::replace(self, OneOrMany::Many(Vec::new())) {
            OneOrMany::One(value) => {
                *self = if f(&value) {
                    OneOrMany::One(value)
                } else {
                    OneOrMany::Many(Vec::new())
                }
            }
            OneOrMany::Many(mut values) => {
                values.retain(|v| f(v));
                *self = OneOrMany::Many(values);
            }
        }
    }
}

impl<T: PartialEq> OneOrMany<T> {
    pub fn contains(&self, value: &T) -> bool {
        self.as_slice().contains(value)
    }
}

impl<'a, T> IntoIterator for &'a OneOrMany<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T> From<T> for OneOrMany<T> {
    fn from(value: T) -> Self {
        OneOrMany::One(value)
    }
}

impl<T> From<Vec<T>> for OneOrMany<T> {
    fn from(values: Vec<T>) -> Self {
        OneOrMany::Many(values)
    }
}

impl<T: Serialize> Serialize for OneOrMany<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            OneOrMany::One(value) => value.serialize(serializer),
            OneOrMany::Many(values) => values.serialize(serializer),
        }
    }
}

impl<'de, T> Deserialize<'de> for OneOrMany<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct OneOrManyVisitor<T>(PhantomData<T>);

        impl<'de, T> Visitor<'de> for OneOrManyVisitor<T>
        where
            T: Deserialize<'de>,
        {
            type Value = OneOrMany<T>;

            fn expecting(&self, f: &mut Formatter) -> fmt::Result {
                f.write_str("a single value or an array of values")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                T::deserialize(StrDeserializer::<E>::new(v)).map(OneOrMany::One)
            }

            fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                T::deserialize(MapAccessDeserializer::new(map)).map(OneOrMany::One)
            }

            fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                Vec::<T>::deserialize(SeqAccessDeserializer::new(seq)).map(OneOrMany::Many)
            }
        }

        deserializer.deserialize_any(OneOrManyVisitor(PhantomData))
    }
}
