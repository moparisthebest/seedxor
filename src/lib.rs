//! # seed-xor
//!
//! seed-xor builds on top of [rust-bip39](https://github.com/rust-bitcoin/rust-bip39/)
//! and lets you XOR bip39 mnemonics as described in [Coldcards docs](https://github.com/Coldcard/firmware/blob/master/docs/seed-xor.md).
//!
//!
//! It is also possible to XOR mnemonics with differing numbers of words.
//! For this the xored value takes on the entropy surplus of the longer seed.
//!
//!
//! ## Example
//!
//! ```rust
//! use seed_xor::Mnemonic;
//! use std::str::FromStr;
//!
//! // Coldcard example: https://github.com/Coldcard/firmware/blob/master/docs/seed-xor.md
//! let a_str = "romance wink lottery autumn shop bring dawn tongue range crater truth ability miss spice fitness easy legal release recall obey exchange recycle dragon room";
//! let b_str = "lion misery divide hurry latin fluid camp advance illegal lab pyramid unaware eager fringe sick camera series noodle toy crowd jeans select depth lounge";
//! let c_str = "vault nominee cradle silk own frown throw leg cactus recall talent worry gadget surface shy planet purpose coffee drip few seven term squeeze educate";
//! let result_str = "silent toe meat possible chair blossom wait occur this worth option bag nurse find fish scene bench asthma bike wage world quit primary indoor";
//!
//! // Mnemonic is a wrapper for bip39::Mnemonic which implements the XOR operation `^`.
//! // Mnemonics can also be created from entropy.
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

/// Trait for a `XOR`.
pub trait SeedXor {
    /// XOR two values without consuming them.
    fn xor(&self, rhs: &Self) -> Self;
}

impl SeedXor for bip39::Mnemonic {
    /// XOR self with another [bip39::Mnemonic] without consuming it or itself.
    fn xor(&self, rhs: &Self) -> Self {
        let mut entropy = self.to_entropy();
        let xor_values = rhs.to_entropy();

        // XOR each Byte
        entropy
            .iter_mut()
            .zip(xor_values.iter())
            .for_each(|(a, b)| *a ^= b);

        // Extend entropy with values of xor_values if it has a shorter entropy length.
        if entropy.len() < xor_values.len() {
            entropy.extend(xor_values.iter().skip(entropy.len()))
        }

        // We unwrap here because entropy has either as many Bytes
        // as self or rhs and both are valid mnemonics.
        bip39::Mnemonic::from_entropy(&entropy).unwrap()
    }
}

/// Wrapper for a [bip39::Mnemonic] for the implementation of `^` and `^=` operators.
#[derive(Clone, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub struct Mnemonic {
    /// Actual [bip39::Mnemonic] which is wrapped to be able to implement the XOR operator.
    pub inner: bip39::Mnemonic,
}

impl Mnemonic {
    /// Private constructor.
    fn new(inner: bip39::Mnemonic) -> Self {
        Mnemonic { inner }
    }

    /// Wrapper for the same method as in [bip39::Mnemonic].
    pub fn from_entropy(entropy: &[u8]) -> Result<Self, bip39::Error> {
        match bip39::Mnemonic::from_entropy(entropy) {
            Ok(inner) => Ok(Mnemonic::new(inner)),
            Err(err) => Err(err),
        }
    }

    /// XOR two [Mnemonic]s without consuming them.
    /// If consumption is not of relevance the XOR operator `^` and XOR assigner `^=` can be used as well.
    fn xor(&self, rhs: &Self) -> Self {
        Mnemonic::new(self.inner.xor(&rhs.inner))
    }
}

impl FromStr for Mnemonic {
    type Err = bip39::Error;

    fn from_str(mnemonic: &str) -> Result<Self, <Self as FromStr>::Err> {
        match bip39::Mnemonic::from_str(mnemonic) {
            Ok(inner) => Ok(Mnemonic::new(inner)),
            Err(err) => Err(err),
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
        assert_eq!(result, b ^ c ^ a); // Commutative
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

    #[test]
    fn seed_xor_with_different_lengths_works() {
        // Coldcard example: https://github.com/Coldcard/firmware/blob/master/docs/seed-xor.md
        // but truncated mnemonics with correct last word.
        let str_24 = "romance wink lottery autumn shop bring dawn tongue range crater truth ability miss spice fitness easy legal release recall obey exchange recycle dragon room";
        let str_16 = "lion misery divide hurry latin fluid camp advance illegal lab pyramid unaware eager fringe sick camera series number";
        let str_12 = "vault nominee cradle silk own frown throw leg cactus recall talent wisdom";
        let result_str = "silent toe meat possible chair blossom wait occur this worth option aware since milk mother grace rocket cement recall obey exchange recycle dragon rocket";

        let w_24 = Mnemonic::from_str(str_24).unwrap();
        let w_16 = Mnemonic::from_str(str_16).unwrap();
        let w_12 = Mnemonic::from_str(str_12).unwrap();
        let result = Mnemonic::from_str(result_str).unwrap();

        assert_eq!(result, w_24.clone() ^ w_16.clone() ^ w_12.clone());
        assert_eq!(result, w_12 ^ w_24 ^ w_16); // Commutative
    }
}
