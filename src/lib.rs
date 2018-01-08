extern crate clear_on_drop;
extern crate serde;
extern crate constant_time_eq;
extern crate rand;
#[macro_use]
extern crate serde_derive;

use clear_on_drop::clear::{Clear, InitializableFromZeroed, ZeroSafe};

// FIXME: can we use core here?
use std::cmp;

use self::rand::Rng;
use self::constant_time_eq::constant_time_eq;

/// Secure token data structure
///
/// Tokens are generated using the system's random number generator (the
/// default of the rand crate)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Token<S> {
    data: S,
}

impl<S: rand::Rand> Token<S> {
    pub unsafe fn create(data: S) -> Token<S> {
        Token { data }
    }

    pub fn generate() -> Token<S> {
        let mut rng = rand::thread_rng();

        Token { data: rng.gen() }
    }
}

impl<S: AsRef<[u8]>> PartialEq for Token<S> {
    fn eq(&self, other: &Self) -> bool {
        constant_time_eq(self.data.as_ref(), other.data.as_ref())
    }
}

impl<S: AsRef<[u8]>> Eq for Token<S> {}

impl<S: AsRef<[u8]>> PartialOrd for Token<S> {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let len_self = self.data.as_ref().len();
        let len_other = other.data.as_ref().len();

        if len_self == len_other {
            Some(
                self.data
                    .as_ref()
                    .iter()
                    .zip(other.data.as_ref().iter())
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

unsafe impl<S: ZeroSafe> ZeroSafe for Token<S> {}
impl<S: InitializableFromZeroed> InitializableFromZeroed for Token<S> {
    unsafe fn initialize(place: *mut Self) {}
}

trait Foo {
    fn has_foo() -> bool {
        true
    }
}

// FIXME: would love to implement this, but currently it seems to be impossible
//        https://github.com/rust-lang/rust/commit/5b2e8693e42dee545d336c0364773b3fbded93a5
// impl<S: Clear> Drop for Token<S> {
//     fn drop(&mut self) {
//         self.clear();
//     }
// }

#[cfg(test)]
mod tests {
    use clear_on_drop::clear::Clear;
    use super::*;
    use std::{mem, slice};

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

    #[test]
    fn test_zeroing_on_drop() {
        // `as_bytes()` has been lifted from clear_on_drop
        fn as_bytes<T>(x: &T) -> &[u8] {
            unsafe { slice::from_raw_parts(x as *const T as *const u8, mem::size_of_val(x)) }
        }

        let mut tok = unsafe { Token::create([0x42; 16]) };
        tok.clear();
        assert!(!as_bytes(&tok).contains(&0x42));
    }
}
