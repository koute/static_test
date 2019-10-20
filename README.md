[![Build Status](https://api.travis-ci.org/koute/static_test.svg)](https://travis-ci.org/koute/static_test)

# `#[static_test]`

[![Documentation](https://docs.rs/static_test/badge.svg)](https://docs.rs/static_test/*/static_test/)

## Example

```rust
use static_test::static_test;

#[static_test]
fn test_slice_get_will_always_succeed_if_length_is_known( buffer: &[u8] ) -> u8 {
    assume!( buffer.len() == 1 );
    match buffer.get( 0 ) {
        Some( &value ) => value,
        None => static_unreachable!()
    }
}

#[static_test]
fn test_multiplication( value: u8 ) {
    assume!( value == 2 );
    static_assert!( value * 10 == 20 );
}
```

You can specify arbitrary types as parameters and as the return type of every
function you mark as `#[static_test]`. The bodies of those functions will never
actually be executed, however every `static_assert!` and `static_unreachable!`
will still be indirectly checked.

If the compiler can't prove that every `static_assert!` will *always* hold true
and that every `static_unreachable!` will *always* be unreachable then an error
will be generated at link time.

Every function marked as `#[static_test]` will be turned into a `#[test]` function.

The `assume!`, `static_assert!` and `static_unreachable!` macros are defined
by the procedural macro and are only available inside functions marked as `#[static_test]`.

## Caveats

* This requires at least Rust 1.40.
* This requires the linker to run, so it will have no effect on `cargo check`.
* This will only work when you compile your code with optimizations turned on,
  as it depends on the optimizer to remove unreachable `static_assert!`s and `static_unreachable!`s.
* *If* an assertion fails the error message [will not be great](https://asciinema.org/a/cvpHYZD6I3YR8TM3xfYfhyHwf).

## Acknowledgments

This is inspired by the [no_panic](https://github.com/dtolnay/no-panic) crate.

## License

Licensed under either of

  * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
  * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
