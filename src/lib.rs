#![no_std]

#[cfg(any(feature = "serialize", feature = "deserialize"))]
extern crate serde;

extern crate constant_time_eq;
extern crate rand;

#[cfg(feature = "serialize")]
use serde::Serializer;

#[cfg(feature = "deserialize")]
use serde::Deserializer;

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "std")]
use std::cmp;
#[cfg(not(feature = "std"))]
use core::cmp;

#[cfg(feature = "std")]
use std::hash::{Hasher, Hash};
#[cfg(not(feature = "std"))]
use core::hash::{Hasher, Hash};

use self::rand::Rng;
use self::constant_time_eq::constant_time_eq;

#[derive(Clone, Debug)]
pub struct Token<S>(S);

impl<S: rand::Rand> Token<S> {
    pub unsafe fn create(data: S) -> Token<S> {
        Token(data)
    }

    pub fn generate() -> Token<S> {
        let mut rng = rand::thread_rng();

        Token(rng.gen())
    }
}

impl<S: AsRef<[u8]>> PartialEq for Token<S> {
    fn eq(&self, other: &Self) -> bool {
        constant_time_eq(self.0.as_ref(), other.0.as_ref())
    }
}

impl<S: AsRef<[u8]>> Eq for Token<S> {}

impl<S: AsRef<[u8]>> PartialOrd for Token<S> {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let len_self = self.0.as_ref().len();
        let len_other = other.0.as_ref().len();

        if len_self == len_other {
            Some(
                self.0
                    .as_ref()
                    .iter()
                    .zip(other.0.as_ref().iter())
                    .filter_map(|(s, o)| s.partial_cmp(o))
                    .skip_while(|&ord| ord == cmp::Ordering::Equal)
                    .next()
                    .unwrap_or(cmp::Ordering::Equal),
            )
        } else {
            // if lengths don't match up, simply compare based on length
            len_self.partial_cmp(&len_other)
        }
    }
}


#[cfg(feature = "serialize")]
impl<T: serde::Serialize> serde::Serialize for Token<T> {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

#[cfg(feature = "deserialize")]
impl<'de, T: serde::Deserialize<'de>> serde::Deserialize<'de> for Token<T> {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        T::deserialize(deserializer).map(Token)
    }
}

impl<T: Hash> Hash for Token<T> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "std")]
    use std::collections::hash_map::DefaultHasher;

    #[test]
    fn test_token_eq() {
        let tok: Token<[u8; 32]> = Token::generate();
        let tok2 = Token::generate();

        assert!(tok != tok2);
    }

    #[test]
    fn test_token_ord() {
        let tok: Token<[u8; 32]> = Token::generate();
        let tok2 = Token::generate();

        assert!(tok < tok2 || tok > tok2)
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_hash() {
        let tok1: Token<[u8; 32]> = Token::generate();
        let mut s = DefaultHasher::new();
        tok1.hash(&mut s);
        let hash1 = s.finish();

        let tok2: Token<[u8; 32]> = Token::generate();
        let mut s = DefaultHasher::new();
        tok2.hash(&mut s);
        let hash2 = s.finish();

        assert!(hash1 != hash2);
    }
}
