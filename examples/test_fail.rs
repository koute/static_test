// Compile this with:
//   cargo rustc --example test_fail --profile test

use static_test::static_test;

#[static_test]
fn test_fail( buffer: &[u8] ) -> u8 {
    match buffer.get( 0 ) {
        Some( &value ) => value,
        None => static_unreachable!()
    }
}

fn main() {}
