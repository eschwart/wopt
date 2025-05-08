mod common;
use common::named::*;

#[test]
fn test_stable() {
    // base is_modified test
    let mut ex_opt = ExampleNamedOpt::default();
    assert!(!ex_opt.is_modified());

    // modify (b, c) of optional struct
    ex_opt.b = Some(B);
    ex_opt.c = Some(C);
    assert!(ex_opt.is_modified());

    // instantiate, then modify (a) of original struct
    let mut ex = ExampleNamed::default();
    ex.a = A;
    ex.patch(&mut ex_opt); // apply patch

    // optional struct should no longer be modified
    assert!(!ex_opt.is_modified());
    // original struct now contains all test parameters
    assert_eq!(
        ex,
        ExampleNamed {
            a: 69,
            b: 420.0,
            c: -2048
        }
    );

    // optional struct is now zeroed out
    assert_eq!(ex_opt, ExampleNamedOpt::default())
}

#[test]
fn test_stable_req() {
    // base is_modified test
    let mut ex_opt = ExampleNamedReqOpt::default();
    assert!(!ex_opt.is_modified());

    // modify (b, c) of optional struct
    ex_opt.b = B;
    ex_opt.c = Some(C);
    assert!(ex_opt.is_modified());

    // instantiate, then modify (a) of original struct
    let mut ex = ExampleNamedReq::default();
    ex.a = A;
    ex.patch(&mut ex_opt); // apply patch

    // optional struct should no longer be modified
    assert!(!ex_opt.is_modified());
    // original struct now contains all test parameters
    assert_eq!(
        ex,
        ExampleNamedReq {
            a: 69,
            b: 0.0,
            c: -2048
        }
    );

    // optional struct is now zeroed out
    assert_eq!(
        ex_opt,
        ExampleNamedReqOpt {
            a: None,
            b: 420.0,
            c: None
        }
    )
}

#[test]
#[cfg(feature = "rkyv")]
fn test_rkyv_serialize() {
    let mut ex_opt = ExampleNamedOpt::default();
    ex_opt.a = Some(A);
    ex_opt.c = Some(C);

    let serialized = ex_opt.serialize();
    assert_eq!(serialized, [0, 5, 69, 0, 248, 255, 255]);
}

#[test]
#[cfg(feature = "rkyv")]
fn test_rkyv_serialize_req() {
    let mut ex_opt = ExampleNamedReqOpt::default();
    ex_opt.a = Some(A);
    ex_opt.b = B;

    let serialized = ex_opt.serialize();
    assert_eq!(serialized, [1, 1, 69, 0, 0, 210, 67]);
}

#[test]
#[cfg(feature = "rkyv")]
fn test_rkyv_deserialize() {
    let bytes = [0, 5, 69, 0, 248, 255, 255];

    let deserialized = ExampleNamedOpt::deserialize(&bytes[1..]);
    assert_eq!(
        deserialized,
        ExampleNamedOpt {
            a: Some(A),
            b: None,
            c: Some(C)
        }
    );
}

#[test]
#[cfg(feature = "rkyv")]
fn test_rkyv_deserialize_req() {
    let bytes = [1, 1, 69, 0, 0, 210, 67];

    let deserialized = ExampleNamedReqOpt::deserialize(&bytes[1..]);
    assert_eq!(
        deserialized,
        ExampleNamedReqOpt {
            a: Some(A),
            b: B,
            c: None
        }
    );
}
