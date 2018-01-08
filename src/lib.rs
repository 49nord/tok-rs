extern crate serde;
extern crate constant_time_eq;
extern crate rand;
#[macro_use]
extern crate serde_derive;

// FIXME: can we use core here?
use std::cmp;

use self::rand::Rng;
use self::constant_time_eq::constant_time_eq;

/// Secure token data structure
///
/// Tokens are generated using the system's random number generator (the
/// default of the rand crate)
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Token<S> {
    data: S,
}

impl<S: rand::Rand> Token<S> {
    unsafe fn create(data: S) -> Token<S> {
        Token { data }
    }

    fn generate() -> Token<S> {
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
