//! # seed-xor
//!
//! seed-xor builds on top of [rust-bip39](https://github.com/rust-bitcoin/rust-bip39/)
//! and lets you XOR 24-word mnemonics as defined in [Coldcards docs](https://github.com/Coldcard/firmware/blob/master/docs/seed-xor.md).
//!
//!
//! Future versions will also allow you to XOR different seed lengths.
//!
//!
//! ## Example
//!
//! ```rust
//! // Coldcard example: https://github.com/Coldcard/firmware/blob/master/docs/seed-xor.md
//! let a_str = "romance wink lottery autumn shop bring dawn tongue range crater truth ability miss spice fitness easy legal release recall obey exchange recycle dragon room";
//! let b_str = "lion misery divide hurry latin fluid camp advance illegal lab pyramid unaware eager fringe sick camera series noodle toy crowd jeans select depth lounge";
//! let c_str = "vault nominee cradle silk own frown throw leg cactus recall talent worry gadget surface shy planet purpose coffee drip few seven term squeeze educate";
//! let result_str = "silent toe meat possible chair blossom wait occur this worth option bag nurse find fish scene bench asthma bike wage world quit primary indoor";
//!
//! // Mnemonic is a wrapper for bip39::Mnemonic which ensures a 24 word seed length.
//! // Mnemonics can also be created from 256bit entropy.
//! let a = Mnemonic::from_str(a_str).unwrap();
//! let b = Mnemonic::from_str(b_str).unwrap();
//! let c = Mnemonic::from_str(c_str).unwrap();
//! let result = Mnemonic::from_str(result_str).unwrap();
//!
//! assert_eq!(result, a ^ b ^ c);
//! ```
//!
use std::{
    fmt,
    ops::{BitXor, BitXorAssign},
    str::FromStr,
};

use bip39::Mnemonic as Inner;

/// Maximal number of words in a mnemonic.
const MAX_MNEMONIC_LENGTH: usize = 24;

/// Errors same as [bip39::Error] but specifically for 24 word mnemonics.
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum SeedXorError {
    /// Mnemonic has a word count that is not 24.
    WordCountNot24,
    /// Mnemonic contains an unknown word.
    /// Error contains the index of the word.
    UnknownWord(usize),
    /// Entropy was not a 256 bits in length.
    EntropyBitsNot256,
    /// The mnemonic has an invalid checksum.
    InvalidChecksum,
    /// The mnemonic can be interpreted as multiple languages.
    AmbiguousLanguages,
}

impl From<bip39::Error> for SeedXorError {
    fn from(err: bip39::Error) -> Self {
        match err {
            bip39::Error::BadEntropyBitCount(_) => return Self::EntropyBitsNot256,
            bip39::Error::BadWordCount(_) => return Self::WordCountNot24,
            bip39::Error::UnknownWord(i) => return Self::UnknownWord(i),
            bip39::Error::InvalidChecksum => return Self::InvalidChecksum,
            bip39::Error::AmbiguousLanguages(_) => Self::AmbiguousLanguages,
        }
    }
}

impl fmt::Display for SeedXorError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SeedXorError::WordCountNot24 => {
                write!(f, "Mnemonic has a word count that is not 24",)
            }
            SeedXorError::UnknownWord(i) => {
                write!(f, "Mnemonic contains an unknown word (word {})", i,)
            }
            SeedXorError::EntropyBitsNot256 => write!(f, "Entropy was not between 256 bits",),
            SeedXorError::InvalidChecksum => write!(f, "Mnemonic has an invalid checksum"),
            SeedXorError::AmbiguousLanguages => write!(f, "Ambiguous language"),
        }
    }
}

/// Wrapper for a [bip39::Mnemonic] which is aliased as `Inner`
#[derive(Clone, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub struct Mnemonic {
    inner: Inner,
}

impl Mnemonic {
    /// Private constructor which ensures that a new [Mnemonic] instance has 24 words.
    fn new(inner: Inner) -> Result<Self, SeedXorError> {
        ensure_24_words(&inner)?;

        Ok(Mnemonic { inner })
    }

    /// Access the inner [bip39::Mnemonic] for more functionality.
    pub fn inner(&self) -> &Inner {
        &self.inner
    }

    /// Wrapper for the same method as in [bip39::Mnemonic]
    /// but it returns an `Err` if the entropy does not result in a 24 word mnemonic.
    pub fn from_entropy(entropy: &[u8]) -> Result<Self, SeedXorError> {
        match Inner::from_entropy(entropy) {
            Ok(inner) => return Ok(Mnemonic::new(inner)?),
            Err(err) => return Err(SeedXorError::from(err)),
        }
    }

    /// XOR two [Mnemonic]s without consuming them.
    /// If consumption is not of relevance the XOR operator `^` and XOR assigner `^=` can be used as well.
    fn xor(&self, rhs: &Self) -> Self {
        let mut xor_result = Vec::with_capacity(MAX_MNEMONIC_LENGTH);

        // XOR self's and other's entropy and push result
        self.inner
            .to_entropy()
            .iter()
            .zip(rhs.inner.to_entropy().iter())
            .for_each(|(a, b)| xor_result.push(a ^ b));

        // We unwrap here because xor_result has as many bytes as self and rhs
        // which in turn have a valid number of bytes
        Mnemonic::from_entropy(&xor_result).unwrap()
    }
}

impl FromStr for Mnemonic {
    type Err = SeedXorError;

    fn from_str(mnemonic: &str) -> Result<Self, <Self as FromStr>::Err> {
        match Inner::from_str(mnemonic) {
            Ok(inner) => return Ok(Mnemonic::new(inner)?),
            Err(err) => Err(SeedXorError::from(err)),
        }
    }
}

impl fmt::Display for Mnemonic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (i, word) in self.inner.word_iter().enumerate() {
            if i > 0 {
                f.write_str(" ")?;
            }
            f.write_str(word)?;
        }
        Ok(())
    }
}

impl BitXor for Mnemonic {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        self.xor(&rhs)
    }
}

impl BitXorAssign for Mnemonic {
    fn bitxor_assign(&mut self, rhs: Self) {
        *self = self.xor(&rhs)
    }
}

/// Ensures that an [Inner] is a 24 word mnemonic for wrapping into a [Mnemonic].
fn ensure_24_words(inner: &Inner) -> Result<(), SeedXorError> {
    if inner.word_count() != MAX_MNEMONIC_LENGTH {
        return Err(SeedXorError::WordCountNot24);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::Mnemonic;
    use std::str::FromStr;

    #[test]
    fn seed_xor_works() {
        // Coldcard example: https://github.com/Coldcard/firmware/blob/master/docs/seed-xor.md
        let a_str = "romance wink lottery autumn shop bring dawn tongue range crater truth ability miss spice fitness easy legal release recall obey exchange recycle dragon room";
        let b_str = "lion misery divide hurry latin fluid camp advance illegal lab pyramid unaware eager fringe sick camera series noodle toy crowd jeans select depth lounge";
        let c_str = "vault nominee cradle silk own frown throw leg cactus recall talent worry gadget surface shy planet purpose coffee drip few seven term squeeze educate";
        let result_str = "silent toe meat possible chair blossom wait occur this worth option bag nurse find fish scene bench asthma bike wage world quit primary indoor";

        let a = Mnemonic::from_str(a_str).unwrap();
        let b = Mnemonic::from_str(b_str).unwrap();
        let c = Mnemonic::from_str(c_str).unwrap();
        let result = Mnemonic::from_str(result_str).unwrap();

        assert_eq!(result, a.clone() ^ b.clone() ^ c.clone());

        // Different order
        assert_eq!(result, b ^ c ^ a);
    }

    #[test]
    fn seed_xor_assignment_works() {
        // Coldcard example: https://github.com/Coldcard/firmware/blob/master/docs/seed-xor.md
        let a_str = "romance wink lottery autumn shop bring dawn tongue range crater truth ability miss spice fitness easy legal release recall obey exchange recycle dragon room";
        let b_str = "lion misery divide hurry latin fluid camp advance illegal lab pyramid unaware eager fringe sick camera series noodle toy crowd jeans select depth lounge";
        let c_str = "vault nominee cradle silk own frown throw leg cactus recall talent worry gadget surface shy planet purpose coffee drip few seven term squeeze educate";
        let result_str = "silent toe meat possible chair blossom wait occur this worth option bag nurse find fish scene bench asthma bike wage world quit primary indoor";

        let a = Mnemonic::from_str(a_str).unwrap();
        let b = Mnemonic::from_str(b_str).unwrap();
        let c = Mnemonic::from_str(c_str).unwrap();
        let result = Mnemonic::from_str(result_str).unwrap();

        let mut assigned = a.xor(&b); // XOR without consuming
        assigned ^= c;

        assert_eq!(result, assigned);
    }
}
