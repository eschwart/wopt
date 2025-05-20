mod common;

#[cfg(feature = "bytemuck")]
use common::unit::*;

#[test]
#[cfg(feature = "bytemuck")]
fn test_rkyv_serialize() {
    let serialized = ExampleUnit::serialize();
    assert_eq!(serialized, [ExampleUnit::ID])
}
