 # seed-xor

 seed-xor builds on top of [rust-bip39](https://github.com/rust-bitcoin/rust-bip39/)
 and lets you XOR 24-word mnemonics as defined in [Coldcard docs](https://github.com/Coldcard/firmware/blob/master/docs/seed-xor.md).


 Future versions will also allow you to XOR different seed lengths.


 ## Example

 ```rust
 // Coldcard example: https://github.com/Coldcard/firmware/blob/master/docs/seed-xor.md
 let a_str = "romance wink lottery autumn shop bring dawn tongue range crater truth ability miss spice fitness easy legal release recall obey exchange recycle dragon room";
 let b_str = "lion misery divide hurry latin fluid camp advance illegal lab pyramid unaware eager fringe sick camera series noodle toy crowd jeans select depth lounge";
 let c_str = "vault nominee cradle silk own frown throw leg cactus recall talent worry gadget surface shy planet purpose coffee drip few seven term squeeze educate";
 let result_str = "silent toe meat possible chair blossom wait occur this worth option bag nurse find fish scene bench asthma bike wage world quit primary indoor";

 let a = Mnemonic::from_str(a_str).unwrap();
 let b = Mnemonic::from_str(b_str).unwrap();
 let c = Mnemonic::from_str(c_str).unwrap();
 let result = Mnemonic::from_str(result_str).unwrap();

 assert_eq!(result, a ^ b ^ c);
 ```

 ## Useful resources
 - Coldcard docs: https://github.com/Coldcard/firmware/blob/master/docs/seed-xor.md
 - Easy mnemonic seed explanation: https://learnmeabitcoin.com/technical/mnemonic
