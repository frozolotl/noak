# Fuzzing Noak
## Fuzz Targets
You can find the available targets in `noak/fuzz/fuzz_targets`.

## Using libFuzzer
Install [cargo-fuzz](https://github.com/rust-fuzz/cargo-fuzz) and [nightly rust](https://rust-lang.github.io/rustup/concepts/channels.html).

Fuzz the `mutf8` target using the following command:
``` sh
$ cargo +nightly fuzz run --features libfuzzer mutf8
```

## Using AFL++
Install cargo-afl:
``` sh
$ cargo install afl
```

Enter the `noak/fuzz` folder if you didn't already:

``` sh
$ cd fuzz
```

Build the `mutf8` target using the following command:
``` sh
$ cargo afl build --features afl --bin mutf8
```

Run the fuzzer:
``` sh
$ cargo afl fuzz -i afl-data/mutf8/in/ -o afl-data/mutf8/out target/debug/mutf8
```
