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

// And you can call functions defined somewhere else or give meaningful names to test cases
plague! {
    for [
        'empty ("",) -> 0,
        ("foo",) -> 3,
        'c_str ("foo\u{0}bar",) -> 7,
    ]
    test str::len
}
```

## Why
The plugin will generate one test function for each parameter set, this way,
running `cargo test` will show each failed value, instead of just one:
```
running 8 tests
test pos#4 ... ok
test pos#2 ... FAILED
test pos#3 ... FAILED
test pos#5 ... ok
test pos'empty ... ok
test pos'not_found ... ok
test pos'unary ... FAILED
test without_plague ... FAILED

failures:

---- pos#2 stdout ----
	thread 'pos#2' panicked at 'test failed: got `None`, expected `Some(0)`', examples/cmp.rs:15

---- pos#3 stdout ----
	thread 'pos#3' panicked at 'test failed: got `Some(2)`, expected `Some(0)`', examples/cmp.rs:15

---- pos'unary stdout ----
	thread 'pos'unary' panicked at 'test failed: got `None`, expected `Some(0)`', examples/cmp.rs:15

---- without_plague stdout ----
	thread 'without_plague' panicked at 'assertion failed: `(left == right)` (left: `None`, right: `Some(0)`)', examples/cmp.rs:41


failures:
    pos#2
    pos#3
    pos'unary
    without_plague

test result: FAILED. 4 passed; 4 failed; 0 ignored; 0 measured
```

[crate-svg]: https://img.shields.io/crates/v/plague.svg
[crate]: https://crates.io/crates/plague/
[license-svg]: https://img.shields.io/crates/l/plague.svg
[license]: https://github.com/mcarton/rust-plague/blob/master/LICENSE
[travis-svg]: https://travis-ci.org/mcarton/rust-plague.svg
[travis]: https://travis-ci.org/mcarton/rust-plague/
