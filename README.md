# seedxor

seedxor builds on top of [rust-bip39](https://github.com/rust-bitcoin/rust-bip39/) and is a fork of [seed-xor](https://github.com/kaiwolfram/seed-xor)
and lets you XOR bip39 mnemonics as described in [Coldcards docs](https://github.com/Coldcard/firmware/blob/master/docs/seed-xor.md).

It also lets you split existing mnemonics into as many seeds as you wish

It is also possible to XOR mnemonics with differing numbers of words.
For this the xored value takes on the entropy surplus of the longer seed.

## Command line example

```
usage: seedxor [options...]
 -h, --help                      Display this help
 -s, --split <seed>              Split seed into num-seeds
 -n, --num-seeds <num>           Number of seeds to split into or generate
                                 default 2
 -y, --no-validate               Do not validate a split can be successfully recombined, useful for
                                 non-bip39 seeds, like ethereum
 -g, --generate                  Generate num-seeds
 -w, --word-count <num>          Number of words to generate in the seed
                                 default 24
 -c, --combine <seeds...>        Combine seeds into one seed
 -r, --short                     Display only first 4 letters of seed words
```

```
$ seedxor -s 'silent toe meat possible chair blossom wait occur this worth option boy'
vault junior rather gentle fresh measure waste powder resemble ocean until body
defy one debate situate jungle music achieve cradle fiscal govern intact acquire
$ seedxor -c 'vault junior rather gentle fresh measure waste powder resemble ocean until body' 'defy one debate situate jungle music achieve cradle fiscal govern intact acquire'
silent toe meat possible chair blossom wait occur this worth option boy

$ seedxor -r -s 'silent toe meat possible chair blossom wait occur this worth option boy'
sieg facu mult uniq agai spar diag widt foll worl rela twis
abou rate brea eagl cann sile skin ging risk able conc util
$ seedxor -c 'sieg facu mult uniq agai spar diag widt foll worl rela twis' 'abou rate brea eagl cann sile skin ging risk able conc util'
silent toe meat possible chair blossom wait occur this worth option boy
$ seedxor -r -c 'sieg facu mult uniq agai spar diag widt foll worl rela twis' 'abou rate brea eagl cann sile skin ging risk able conc util'
sile toe meat poss chai blos wait occu this wort opti boy

$ seedxor -g
spell system smoke army frame vacant trick jacket anchor gasp acoustic supply deputy portion butter similar trend scorpion cause fish outer armor process faint
solar lab option erosion unit example convince viable soft smart smile spoon range card gentle miracle latin they verify want reject side cheese panther
$ seedxor -c 'spell system smoke army frame vacant trick jacket anchor gasp acoustic supply deputy portion butter similar trend scorpion cause fish outer armor process faint' 'solar lab option erosion unit example convince viable soft smart smile spoon range card gentle miracle latin they verify want reject side cheese panther'
butter patch first doll raise safe side lounge shiver protect solid area melody member lazy easily nice canvas stomach pattern claim slot million stomach
```

## Library Example

```rust
use seedxor::{Mnemonic, SeedXor};
use std::str::FromStr;

// Coldcard example: https://github.com/Coldcard/firmware/blob/master/docs/seed-xor.md
let a_str = "romance wink lottery autumn shop bring dawn tongue range crater truth ability miss spice fitness easy legal release recall obey exchange recycle dragon room";
let b_str = "lion misery divide hurry latin fluid camp advance illegal lab pyramid unaware eager fringe sick camera series noodle toy crowd jeans select depth lounge";
let c_str = "vault nominee cradle silk own frown throw leg cactus recall talent worry gadget surface shy planet purpose coffee drip few seven term squeeze educate";
let result_str = "silent toe meat possible chair blossom wait occur this worth option bag nurse find fish scene bench asthma bike wage world quit primary indoor";

// Mnemonic is a wrapper for bip39::Mnemonic which implements the XOR operation `^`.
// Mnemonics can also be created from entropy.
let a = Mnemonic::from_str(a_str).unwrap();
let b = Mnemonic::from_str(b_str).unwrap();
let c = Mnemonic::from_str(c_str).unwrap();
let result = Mnemonic::from_str(result_str).unwrap();

assert_eq!(result, a ^ b ^ c);

// split a into 3 mnemonics
let a = Mnemonic::from_str(a_str).unwrap();
let split = a.splitn(3).unwrap();
let recombined_a = Mnemonic::xor_all(&split).unwrap();
assert_eq!(a_str, recombined_a.to_string());
```
