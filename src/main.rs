use seedxor::{expand_words, Language, Mnemonic, SeedXor};
use std::{process::ExitCode, str::FromStr};

pub struct Args {
    args: Vec<String>,
}

impl Args {
    pub fn new(args: Vec<String>) -> Args {
        Args { args }
    }
    pub fn flags(&mut self, flags: &[&str]) -> bool {
        let mut i = 0;
        while i < self.args.len() {
            if flags.contains(&self.args[i].as_str()) {
                self.args.remove(i);
                return true;
            } else {
                i += 1;
            }
        }
        false
    }
    pub fn flag(&mut self, flag: &str) -> bool {
        self.flags(&[flag])
    }
    pub fn get_option(&mut self, flags: &[&str]) -> Option<String> {
        let mut i = 0;
        while i < self.args.len() {
            if flags.contains(&self.args[i].as_str()) {
                // remove the flag
                self.args.remove(i);
                return if i < self.args.len() {
                    Some(self.args.remove(i))
                } else {
                    None
                };
            } else {
                i += 1;
            }
        }
        return None;
    }
    pub fn get_str(&mut self, flags: &[&str], def: &str) -> String {
        match self.get_option(flags) {
            Some(ret) => ret,
            None => def.to_owned(),
        }
    }
    pub fn get<T: FromStr>(&mut self, flags: &[&str], def: T) -> T {
        match self.get_option(flags) {
            Some(ret) => match ret.parse::<T>() {
                Ok(ret) => ret,
                Err(_) => def, // or panic
            },
            None => def,
        }
    }
    pub fn remaining(self) -> Vec<String> {
        self.args
    }
}

impl Default for Args {
    fn default() -> Self {
        Self::new(std::env::args().skip(1).collect())
    }
}

const NUM_SEEDS: usize = 2;
const WORD_COUNT: usize = 24;

fn help(success: bool) -> ExitCode {
    println!(
        r###"usage: seedxor [options...]
 -h, --help                        Display this help
 -s, --split <seed>                Split seed into num-seeds
 -n, --num-seeds <num>             Number of seeds to split into or generate
                                   default {NUM_SEEDS}
 -y, --no-validate                 Do not validate a split can be successfully recombined, useful for
                                   non-bip39 seeds, like ethereum
 -g, --generate                    Generate num-seeds
 -w, --word-count <num>            Number of words to generate in the seed
                                   default {WORD_COUNT}
 -c, --combine <seeds...>          Combine seeds into one seed
 -r, --short                       Display only first 4 letters of seed words
 -u, --unscramble <seed-parts...>  Unscramble seed words in random order to valid seeds
        "###
    );
    if success {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn main() -> ExitCode {
    let mut args = Args::default();

    let short = args.flags(&["-r", "--short"]);
    let num_seeds = args.get(&["-n", "--num-seeds"], NUM_SEEDS);
    if num_seeds < 1 {
        println!("error: num-seeds must be > 1");
        return help(false);
    } else if args.flags(&["-h", "--help"]) {
        return help(true);
    } else if args.flags(&["-s", "--split"]) {
        let no_validate = args.flags(&["-y", "--no-validate"]);
        let remaining = args.remaining();
        if remaining.len() != 1 {
            println!("remaining: {remaining:?}");
            println!("error: --split needs exactly 1 seed argument");
            return help(false);
        }
        let seed = &remaining[0];
        let seed: Mnemonic = if no_validate {
            Mnemonic::parse_normalized_without_checksum_check(seed).expect("invalid mnemonic")
        } else {
            Mnemonic::from_str(seed).expect("invalid bip39 mnemonic")
        };
        let parts = seed
            .clone()
            .splitn(num_seeds)
            .expect("could not split mnemonic");
        if !no_validate {
            let result = Mnemonic::xor_all(&parts).unwrap();
            if result != seed {
                panic!("error: result != seed, '{result}' != '{seed}'");
            }
        }
        for part in parts {
            println!("{}", part.to_display_string(short));
        }
    } else if args.flags(&["-g", "--generate"]) {
        let word_count = args.get(&["-w", "--word-count"], WORD_COUNT);
        if !args.remaining().is_empty() {
            println!("error: --generate needs 0 arguments");
            return help(false);
        }
        for _ in 0..num_seeds {
            println!(
                "{}",
                Mnemonic::generate_in(Language::English, word_count)
                    .expect("cannot generate seed")
                    .to_display_string(short)
            );
        }
    } else if args.flags(&["-c", "--combine"]) {
        let remaining = args.remaining();
        if remaining.is_empty() {
            println!("error: --combine needs > 0 arguments");
            return help(false);
        }
        let parts: Vec<Mnemonic> = remaining
            .into_iter()
            .map(|s| Mnemonic::from_str(&s).expect("invalid bip39 mnemonic"))
            .collect();
        let seed = Mnemonic::xor_all(&parts).unwrap();
        println!("{}", seed.to_display_string(short));
    } else if args.flags(&["-u", "--unscramble"]) {
        let remaining = args.remaining();
        if remaining.is_empty() {
            println!("error: --unscramble needs > 0 arguments");
            return help(false);
        }
        let mut parts: Vec<String> = remaining
            .into_iter()
            .map(|s| expand_words(&s).expect("invalid bip39 seed words"))
            .collect();
        let total: u128 = (1..=parts.len() as u128).product();
        eprintln!("# total permutations: {total}");
        if total > u64::MAX as u128 {
            println!("total too large, will never finish, aborting");
            return ExitCode::FAILURE;
        }
        let mut heap = permutohedron::Heap::new(&mut parts);
        let mut good = 0u64;
        while let Some(words) = heap.next_permutation() {
            let words = words.join(" ");
            if let Ok(mnemonic) = Mnemonic::from_str(&words) {
                if short {
                    println!("{}", mnemonic.to_short_string());
                } else {
                    println!("{words}");
                }
                good += 1;
            }
        }
        let bad = total - good as u128;
        eprintln!("# good: {good} bad: {bad} total: {total}");
    } else {
        println!("error: need one of -s/-g/-c/-u");
        return help(false);
    }
    ExitCode::SUCCESS
}
