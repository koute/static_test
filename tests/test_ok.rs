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
