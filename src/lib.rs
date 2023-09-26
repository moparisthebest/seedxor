//! # seedxor
//!
//! seedxor builds on top of [rust-bip39](https://github.com/rust-bitcoin/rust-bip39/) and is a fork of [seed-xor](https://github.com/kaiwolfram/seed-xor)
//! and lets you XOR bip39 mnemonics as described in [Coldcards docs](https://github.com/Coldcard/firmware/blob/master/docs/seed-xor.md).
//!
//! It also lets you split existing mnemonics into as many seeds as you wish
//!
//! It is also possible to XOR mnemonics with differing numbers of words.
//! For this the xored value takes on the entropy surplus of the longer seed.
//!
//! ## Example
//!
//! ```rust
//! use seedxor::{Mnemonic, SeedXor};
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
//!
//! // split a into 3 mnemonics
//! let a = Mnemonic::from_str(a_str).unwrap();
//! let split = a.splitn(3).unwrap();
//! let recombined_a = Mnemonic::xor_all(&split).unwrap();
//! assert_eq!(a_str, recombined_a.to_string());
//! ```
//!
pub use bip39::{Error, Language};
use std::fmt::Display;
use std::ops::{Deref, DerefMut};
use std::{
    fmt,
    ops::{BitXor, BitXorAssign},
    str::FromStr,
};

/// Trait for a `XOR`.
pub trait SeedXor {
    /// XOR two values without consuming them.
    fn xor(&self, rhs: &Self) -> Self;

    fn xor_all(slice: &[Self]) -> Option<Self>
    where
        Self: Sized + Clone,
    {
        let first = slice.get(0)?;
        // expensive clone :)
        //let first = first.xor(first).xor(first);
        let first = first.clone();
        Some(slice.iter().skip(1).fold(first, |x, y| x.xor(y)))
    }
}

impl SeedXor for bip39::Mnemonic {
    /// XOR self with another [bip39::Mnemonic] without consuming it or itself.
    fn xor(&self, rhs: &Self) -> Self {
        let (mut entropy, entropy_len) = self.to_entropy_array();
        let (xor_values, xor_values_len) = rhs.to_entropy_array();
        let entropy = &mut entropy[0..entropy_len];
        let xor_values = &xor_values[0..xor_values_len];

        // XOR each Byte
        entropy
            .iter_mut()
            .zip(xor_values.iter())
            .for_each(|(a, b)| *a ^= b);

        // Extend entropy with values of xor_values if it has a shorter entropy length.
        if entropy.len() < xor_values.len() {
            let mut entropy = entropy.to_vec();
            entropy.extend(xor_values.iter().skip(entropy.len()));

            bip39::Mnemonic::from_entropy(&entropy).unwrap()
        } else {
            // We unwrap here because entropy has either as many Bytes
            // as self or rhs and both are valid mnemonics.
            bip39::Mnemonic::from_entropy(&entropy).unwrap()
        }
    }
}

/// Wrapper for a [bip39::Mnemonic] for the implementation of `^` and `^=` operators.
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Mnemonic {
    /// Actual [bip39::Mnemonic] which is wrapped to be able to implement the XOR operator.
    pub inner: bip39::Mnemonic,
}

impl Mnemonic {
    pub fn split(&self) -> Result<[Self; 2], Error> {
        let random = Self::generate_in(self.language(), self.word_count())?;
        let calc = self.xor(&random);
        Ok([calc, random])
    }

    pub fn splitn(self, n: usize) -> Result<Vec<Self>, Error> {
        let mut ret: Vec<Self> = Vec::with_capacity(n);
        if n == 1 {
            ret.push(self);
        } else {
            ret.extend_from_slice(&self.split()?);
            for _ in 0..n - 2 {
                let split = ret.pop().expect("cannot be empty").split()?;
                ret.extend_from_slice(&split);
            }
        }
        Ok(ret)
    }

    pub fn generate_in(language: Language, word_count: usize) -> Result<Self, Error> {
        //let inner = bip39::Mnemonic::generate_in(language, word_count)?;
        let mut inner = vec![0u8; (word_count / 3) * 4];
        getrandom::getrandom(&mut inner)
            .map_err(|e| Error::BadEntropyBitCount(e.code().get() as usize))?;
        bip39::Mnemonic::from_entropy_in(language, &inner).map(|m| m.into())
    }

    /// Wrapper for the same method as in [bip39::Mnemonic].
    pub fn from_entropy(entropy: &[u8]) -> Result<Self, Error> {
        bip39::Mnemonic::from_entropy(entropy).map(|m| m.into())
    }

    pub fn parse_normalized_without_checksum_check(s: &str) -> Result<Mnemonic, Error> {
        let lang = bip39::Mnemonic::language_of(s).unwrap_or(Language::English);
        Self::parse_in_normalized_without_checksum_check(lang, s)
    }

    pub fn parse_in_normalized_without_checksum_check(
        language: Language,
        s: &str,
    ) -> Result<Mnemonic, Error> {
        bip39::Mnemonic::parse_in_normalized_without_checksum_check(
            language,
            &expand_words_in(language, s)?,
        )
        .map(|m| m.into())
    }

    pub fn to_short_string(&self) -> String {
        let mut ret = self.word_iter().fold(String::new(), |mut s, w| {
            w.chars().take(4).for_each(|c| s.push(c));
            s.push(' ');
            s
        });
        ret.pop();
        ret
    }

    pub fn to_display_string(&self, short: bool) -> String {
        if short {
            self.to_short_string()
        } else {
            self.to_string()
        }
    }
}

pub fn expand_words(seed: &str) -> Result<String, Error> {
    let lang = bip39::Mnemonic::language_of(seed).unwrap_or(Language::English);
    expand_words_in(lang, seed)
}

pub fn expand_words_in(language: Language, seed: &str) -> Result<String, Error> {
    let mut ret = String::new();
    for (i, prefix) in seed.to_lowercase().split_whitespace().enumerate() {
        let words = language.words_by_prefix(prefix);
        let word = if words.len() == 1 {
            words[0]
        } else if words.contains(&prefix) {
            prefix
        } else {
            // println!("prefix: '{prefix}', words: {words:?}");
            // not unique or correct prefix
            return Err(Error::UnknownWord(i));
        };
        ret.push_str(word);
        ret.push(' ');
    }
    ret.pop();
    Ok(ret)
}

impl SeedXor for Mnemonic {
    /// XOR two [Mnemonic]s without consuming them.
    /// If consumption is not of relevance the XOR operator `^` and XOR assigner `^=` can be used as well.
    fn xor(&self, rhs: &Self) -> Self {
        self.inner.xor(&rhs.inner).into()
    }
}

impl Deref for Mnemonic {
    type Target = bip39::Mnemonic;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Mnemonic {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl From<bip39::Mnemonic> for Mnemonic {
    fn from(inner: bip39::Mnemonic) -> Self {
        Self { inner }
    }
}

impl FromStr for Mnemonic {
    type Err = bip39::Error;

    fn from_str(mnemonic: &str) -> Result<Self, <Self as FromStr>::Err> {
        bip39::Mnemonic::from_str(&expand_words(mnemonic)?).map(|m| m.into())
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

impl fmt::Debug for Mnemonic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        <Mnemonic as Display>::fmt(self, f)
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
    use crate::*;
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

    #[test]
    fn seed_xor_works_12() {
        // Coldcard example: https://github.com/Coldcard/firmware/blob/master/docs/seed-xor.md
        let a_str = "romance wink lottery autumn shop bring dawn tongue range crater truth ability";
        let b_str = "lion misery divide hurry latin fluid camp advance illegal lab pyramid unhappy";
        let c_str = "vault nominee cradle silk own frown throw leg cactus recall talent wait";
        let result_str = "silent toe meat possible chair blossom wait occur this worth option boy";

        let a = Mnemonic::from_str(a_str).unwrap();
        let b = Mnemonic::from_str(b_str).unwrap();
        let c = Mnemonic::from_str(c_str).unwrap();
        let result = Mnemonic::from_str(result_str).unwrap();

        assert_eq!(result, a.clone() ^ b.clone() ^ c.clone());
        assert_eq!(result, b ^ c ^ a); // Commutative
    }

    #[test]
    fn test_electrum_seed() {
        let electrum_seed =
            "ramp exotic resource icon sun addict equip sand leisure spare swing tobacco";
        // ends up with the incorrect checksum
        let expected = "ramp exotic resource icon sun addict equip sand leisure spare swing toast";

        //let electrum_seed = Mnemonic::from_str(electrum_seed).unwrap();
        let seed = Mnemonic::from(
            bip39::Mnemonic::parse_in_normalized_without_checksum_check(
                Language::English,
                electrum_seed,
            )
            .unwrap(),
        );

        assert_eq!(electrum_seed, seed.to_string());

        let expected = Mnemonic::from_str(expected).unwrap();

        let split = seed.clone().split().unwrap();
        println!("1split: '{split:?}'");
        let result = Mnemonic::xor_all(&split).unwrap();
        println!("result: '{}'", result);
        if seed != result {
            assert_eq!(expected, result);
        }

        for x in 1..=5 {
            let split = seed.clone().splitn(x).unwrap();
            assert_eq!(x, split.len());
            println!("split: '{split:?}'");
            let result = Mnemonic::xor_all(&split).unwrap();
            println!("result: '{}'", result);
            if seed != result {
                assert_eq!(expected, result);
            }
        }
    }

    #[test]
    fn derive_from_seed() {
        // tl;dr for any seed you can generate a random seed and xor it to "split" it into 2 seeds
        // you can then do that any number of times for the sub-seeds
        let seed = "silent toe meat possible chair blossom wait occur this worth option boy";
        let seed = Mnemonic::from_str(seed).unwrap();

        let split = seed.clone().split().unwrap();
        println!("split: '{split:?}'");
        assert_eq!(seed.clone(), Mnemonic::xor_all(&split).unwrap());

        for x in 1..=5 {
            let split = seed.clone().splitn(x).unwrap();
            assert_eq!(x, split.len());
            println!("split: '{split:?}'");
            assert_eq!(seed.clone(), Mnemonic::xor_all(&split).unwrap());
        }
    }

    #[test]
    fn expand_seed() {
        let orig_seed = "silent toe meat possible chair blossom wait occur this worth option boy";
        let seed = Mnemonic::from_str(orig_seed).unwrap();

        let short_string = seed.to_short_string();
        assert_eq!(
            "sile toe meat poss chai blos wait occu this wort opti boy",
            short_string
        );
        //assert_eq!(Language::English, bip39::Mnemonic::language_of(&short_string).unwrap());

        assert_eq!(orig_seed, expand_words(&short_string).unwrap());

        // add and addict (addi) are both bip39 words, make sure those work
        let orig_seed = "song vanish mistake night drink add modify lens average cool evil chest";
        let seed = Mnemonic::from_str(orig_seed).unwrap();

        let short_string = seed.to_short_string();
        assert_eq!(
            "song vani mist nigh drin add modi lens aver cool evil ches",
            short_string
        );
        assert_eq!(
            Language::English,
            bip39::Mnemonic::language_of(&short_string).unwrap()
        );

        assert_eq!(orig_seed, expand_words(&short_string).unwrap());

        let orig_seed = "ramp exotic resource icon sun addict equip sand leisure spare swing toast";
        let seed = Mnemonic::from_str(orig_seed).unwrap();

        let short_string = seed.to_short_string();
        assert_eq!(
            "ramp exot reso icon sun addi equi sand leis spar swin toas",
            short_string
        );
        assert_eq!(
            Language::English,
            bip39::Mnemonic::language_of(&short_string).unwrap()
        );

        assert_eq!(orig_seed, expand_words(&short_string).unwrap());
    }
}
