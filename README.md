# *Plague* [![Build Status][travis-svg]][travis] [![Crates.io][crate-svg]][crate] [![License][license-svg]][license]
Parametrized tests tools for Rust

## What
This `rustc` plugin adds a macro to help you make parametrized tests:

```rust
// Basic usage
plague! {
    for [
        (1, 1),
        (2, 2),
    ]
    test fn eq(a: i32, b: i32) {
        assert_eq!(a, b);
    }
}

// You can also specify the expected value, Plague will `assert_eq!` the result for you
plague! {
    for [1 -> 2, 2 -> 4]
    test fn double(a: i32) -> i32 {
        2*a
    }
}

// And you can call functions defined somewhere else
plague! {
    for [
        ("",) -> 0,
        ("foo",) -> 3,
        ("foo\u{0}bar",) -> 7,
    ]
    test str::len
}
```

## Why
The plugin will generate one test function for each parameter set, this way,
running `cargo test` will show each failed value, instead of just one:
```
running 8 tests
test pos_0 ... ok
test pos_2 ... FAILED
test pos_3 ... FAILED
test pos_1 ... FAILED
test pos_4 ... ok
test pos_5 ... ok
test pos_6 ... ok
test without_plague ... FAILED

failures:

---- pos_2 stdout ----
	thread 'pos_2' panicked at 'assertion failed: `(left == right)` (left: `Some(0)`, right: `None`)', examples/cmp.rs:15

---- pos_3 stdout ----
	thread 'pos_3' panicked at 'assertion failed: `(left == right)` (left: `Some(0)`, right: `Some(2)`)', examples/cmp.rs:15

---- pos_1 stdout ----
	thread 'pos_1' panicked at 'assertion failed: `(left == right)` (left: `Some(0)`, right: `None`)', examples/cmp.rs:15

---- without_plague stdout ----
	thread 'without_plague' panicked at 'assertion failed: `(left == right)` (left: `None`, right: `Some(0)`)', examples/cmp.rs:41


failures:
    pos_1
    pos_2
    pos_3
    without_plague

test result: FAILED. 4 passed; 4 failed; 0 ignored; 0 measured
```

[crate-svg]: https://img.shields.io/crates/v/plague.svg
[crate]: https://crates.io/crates/plague/
[license-svg]: https://img.shields.io/crates/l/plague.svg
[license]: https://github.com/mcarton/rust-plague/blob/master/LICENSE
[travis-svg]: https://travis-ci.org/mcarton/rust-plague.svg
[travis]: https://travis-ci.org/mcarton/rust-plague/
