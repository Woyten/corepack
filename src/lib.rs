//! corepack is a no_std support for messagepack in serde.
//
// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at https://mozilla.org/MPL/2.0/.

#![cfg_attr(feature = "alloc", feature(alloc))]
#![allow(overflowing_literals)]

// testing requires std to be available
#![cfg_attr(all(not(feature = "std"), not(test)), no_std)]
#[cfg(all(not(feature = "std"), not(test)))]
extern crate core as std;
extern crate serde;
extern crate byteorder;
#[cfg(test)]
#[macro_use]
extern crate serde_derive;

#[cfg(feature = "alloc")]
#[macro_use]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::Vec;

pub use ser::Serializer;
pub use de::Deserializer;

pub mod error;
pub mod read;

mod defs;
mod seq_serializer;
mod map_serializer;
mod variant_deserializer;
mod ext_deserializer;
mod seq_deserializer;

mod ser;
mod de;

/// Parse V out of a stream of bytes.
pub fn from_iter<I, V>(mut iter: I) -> Result<V, error::Error>
    where I: Iterator<Item = u8>,
          V: serde::de::DeserializeOwned
{
    let mut de = Deserializer::new(read::CopyRead::new(|buf: &mut [u8]| {
        for i in 0..buf.len() {
            if let Some(byte) = iter.next() {
                buf[i] = byte;
            } else {
                return Err(error::Error::EndOfStream);
            }
        }

        Ok(())
    }));

    V::deserialize(&mut de)
}

/// Parse V out of a slice of bytes.
pub fn from_bytes<'a, V>(bytes: &'a [u8]) -> Result<V, error::Error>
    where V: serde::Deserialize<'a>
{
    let mut position: usize = 0;

    let mut de = Deserializer::new(read::BorrowRead::new(|len: usize| if position + len >
                                                                         bytes.len() {
        Err(error::Error::EndOfStream)
    } else {
        let result = &bytes[position..position + len];

        position += len;

        Ok(result)
    }));

    V::deserialize(&mut de)
}

/// Serialize V into a byte buffer.
pub fn to_bytes<V>(value: V) -> Result<Vec<u8>, error::Error>
    where V: serde::Serialize
{
    let mut bytes = vec![];

    {
        let mut ser = Serializer::new(|buf| {
            bytes.extend_from_slice(buf);
            Ok(())
        });

        try!(value.serialize(&mut ser));
    }

    Ok(bytes)
}

#[cfg(test)]
mod test {
    use serde::Serialize;
    use serde::de::DeserializeOwned;
    use std::fmt::Debug;

    #[derive(PartialEq, Eq, Debug, Serialize, Deserialize)]
    enum T {
        A(usize),
        B,
        C(i8, i8),
        D { a: isize, b: String },
    }

    fn test_through<T>(expected: T)
        where T: Serialize + DeserializeOwned + PartialEq + Debug
    {
        let x = ::to_bytes(&expected).expect("Failed to serialize");

        let actual = ::from_bytes(&x).expect("Failed to deserialize");

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_str() {
        test_through(format!("Hello World!"))
    }

    #[test]
    fn test_enum() {
        test_through(T::B)
    }

    #[test]
    fn test_enum_newtype() {
        test_through(T::A(42))
    }

    #[test]
    fn test_enum_tuple() {
        test_through(T::C(-3, 22))
    }

    #[test]
    fn test_enum_struct() {
        test_through(T::D {
            a: 9001,
            b: "Hello world!".into(),
        })
    }

    #[test]
    fn test_option() {
        test_through(Some(7))
    }

    #[test]
    fn test_unit_option() {
        test_through(Some(()))
    }

    #[test]
    fn test_char() {
        test_through('b')
    }
}
