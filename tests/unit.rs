mod common;

#[cfg(feature = "rkyv")]
use common::unit::*;

#[test]
#[cfg(feature = "rkyv")]
fn test_rkyv_serialize() {
    let serialized = ExampleUnit::serialize();
    assert_eq!(serialized, [ExampleUnit::ID])
}
