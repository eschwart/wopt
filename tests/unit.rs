mod common;

#[cfg(feature = "rkyv")]
use common::unit::*;

#[test]
#[cfg(feature = "rkyv")]
fn test_rkyv_serialize() {
    let ex_opt = ExampleUnitOpt::default();
    let serialized = ex_opt.serialize();
    assert_eq!(serialized, [0])
}
