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
#[cfg(feature = "bytemuck")]
fn test_rkyv_serialize() {
    let ex = ExampleNamed { a: A, b: B, c: C };

    let serialized = ex.serialize();
    assert_eq!(
        serialized,
        [ExampleNamed::ID, 69, 0, 0, 210, 67, 0, 248, 255, 255]
    );
}

#[test]
#[cfg(feature = "bytemuck")]
fn test_rkyv_serialize_with() {
    let ex = ExampleNamedWith { a: A, b: B, c: C };

    let serialized = ex.serialize();
    assert_eq!(
        serialized,
        [ExampleNamedWith::ID, 69, 0, 0, 210, 67, 0, 248, 255, 255]
    );
}

#[test]
#[cfg(feature = "bytemuck")]
fn test_rkyv_serialize_opt() {
    let mut ex_opt = ExampleNamedOpt::default();
    ex_opt.a = Some(A);
    ex_opt.c = Some(C);

    let serialized = ex_opt.serialize();
    assert_eq!(serialized, [ExampleNamedOpt::ID, 5, 69, 0, 248, 255, 255]);
}

#[test]
#[cfg(feature = "bytemuck")]
fn test_rkyv_serialize_req() {
    let ex = ExampleNamedReq { a: A, b: B, c: C };

    let serialized = ex.serialize();
    assert_eq!(
        serialized,
        [ExampleNamedReq::ID, 69, 0, 0, 210, 67, 0, 248, 255, 255]
    );
}

#[test]
#[cfg(feature = "bytemuck")]
fn test_rkyv_serialize_req_opt() {
    let mut ex_opt = ExampleNamedReqOpt::default();
    ex_opt.a = Some(A);
    ex_opt.b = B;

    let serialized = ex_opt.serialize();
    assert_eq!(serialized, [ExampleNamedReqOpt::ID, 1, 69, 0, 0, 210, 67]);
}

#[test]
#[cfg(feature = "bytemuck")]
fn test_rkyv_deserialize() {
    let bytes = [ExampleNamed::ID, 69, 0, 0, 210, 67, 0, 248, 255, 255];

    let deserialized = ExampleNamed::deserialize(&bytes[1..]);
    assert_eq!(deserialized, ExampleNamed { a: A, b: B, c: C });
}

#[test]
#[cfg(feature = "bytemuck")]
fn test_rkyv_deserialize_with() {
    let bytes = [ExampleNamedWith::ID, 69, 0, 0, 210, 67, 0, 248, 255, 255];

    let deserialized = ExampleNamedWith::deserialize(&bytes[1..]);
    assert_eq!(deserialized, ExampleNamedWith { a: A, b: B, c: C });
}

#[test]
#[cfg(feature = "bytemuck")]
fn test_rkyv_deserialize_opt() {
    let bytes = [ExampleNamedOpt::ID, 5, 69, 0, 248, 255, 255];

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
#[cfg(feature = "bytemuck")]
fn test_rkyv_deserialize_req() {
    let bytes = [ExampleNamedReq::ID, 69, 0, 0, 210, 67, 0, 248, 255, 255];

    let deserialized = ExampleNamedReq::deserialize(&bytes[1..]);
    assert_eq!(deserialized, ExampleNamedReq { a: A, b: B, c: C });
}

#[test]
#[cfg(feature = "bytemuck")]
fn test_rkyv_deserialize_req_opt() {
    let bytes = [ExampleNamedReqOpt::ID, 1, 69, 0, 0, 210, 67];

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
