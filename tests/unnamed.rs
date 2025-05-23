mod common;
use common::unnamed::*;

#[test]
fn test_unnamed_stable() {
    // base is_modified test
    let mut ex_opt = ExampleUnnamedOpt::default();
    assert!(!ex_opt.is_modified());

    // modify (b, c) of optional struct
    ex_opt.1 = Some(B);
    ex_opt.2 = Some(C);
    assert!(ex_opt.is_modified());

    // instantiate, then modify (a) of original struct
    let mut ex = ExampleUnnamed::default();
    ex.0 = A;
    ex.patch(&mut ex_opt); // apply patch

    // optional struct should no longer be modified
    assert!(!ex_opt.is_modified());
    // original struct now contains all test parameters
    assert_eq!(
        ex,
        ExampleUnnamed {
            0: 69,
            1: 420.0,
            2: -2048
        }
    );
    // optional struct is now zeroed out
    assert_eq!(ex_opt, ExampleUnnamedOpt::default())
}

#[test]
fn test_unnamed_stable_req() {
    // base is_modified test
    let mut ex_opt = ExampleUnnamedReqOpt::default();
    assert!(!ex_opt.is_modified());

    // modify (b, c) of optional struct
    ex_opt.1 = B;
    ex_opt.2 = Some(C);
    assert!(ex_opt.is_modified());

    // instantiate, then modify (a) of original struct
    let mut ex = ExampleUnnamedReq::default();
    ex.0 = A;
    ex.patch(&mut ex_opt); // apply patch

    // optional struct should no longer be modified
    assert!(!ex_opt.is_modified());
    // original struct now contains all test parameters
    assert_eq!(
        ex,
        ExampleUnnamedReq {
            0: 69,
            1: 0.0,
            2: -2048
        }
    );
    // optional struct is now zeroed out
    assert_eq!(
        ex_opt,
        ExampleUnnamedReqOpt {
            0: None,
            1: 420.0,
            2: None
        }
    )
}

#[test]
#[cfg(feature = "bytemuck")]
fn test_unnamed_bytemuck_serialize() {
    let ex = ExampleUnnamed { 0: A, 1: B, 2: C };
    let serialized = ex.serialize();
    assert_eq!(
        serialized,
        [ExampleUnnamed::ID, 69, 0, 0, 210, 67, 0, 248, 255, 255]
    );
}

#[test]
#[cfg(feature = "bytemuck")]
fn test_unnamed_bytemuck_serialize_with() {
    let ex = ExampleUnnamedWith { 0: A, 1: B, 2: C };
    let serialized = ex.serialize();
    assert_eq!(
        serialized,
        [ExampleUnnamedWith::ID, 69, 0, 0, 210, 67, 0, 248, 255, 255]
    );
}

#[test]
#[cfg(feature = "bytemuck")]
fn test_unnamed_bytemuck_serialize_opt() {
    let ex = ExampleUnnamedOpt {
        0: Some(A),
        1: None,
        2: Some(C),
    };
    let serialized = ex.serialize();
    assert_eq!(serialized, [ExampleUnnamedOpt::ID, 5, 69, 0, 248, 255, 255]);
}

#[test]
#[cfg(feature = "bytemuck")]
fn test_unnamed_bytemuck_serialize_req() {
    let ex = ExampleUnnamedReq { 0: A, 1: B, 2: C };
    let serialized = ex.serialize();
    assert_eq!(
        serialized,
        [ExampleUnnamedReq::ID, 69, 0, 0, 210, 67, 0, 248, 255, 255]
    );
}

#[test]
#[cfg(feature = "bytemuck")]
fn test_unnamed_bytemuck_serialize_req_opt() {
    let ex = ExampleUnnamedReqOpt {
        0: Some(A),
        1: B,
        2: None,
    };
    let serialized = ex.serialize();
    assert_eq!(serialized, [ExampleUnnamedReqOpt::ID, 1, 69, 0, 0, 210, 67]);
}

#[test]
#[cfg(feature = "bytemuck")]
fn test_unnamed_bytemuck_deserialize() {
    let ex = ExampleUnnamed { 0: A, 1: B, 2: C };
    let bytes = ex.serialize();
    let deserialized = ExampleUnnamed::deserialize(&bytes[1..]);
    assert_eq!(deserialized, ex);
}

#[test]
#[cfg(feature = "bytemuck")]
fn test_unnamed_bytemuck_deserialize_with() {
    let ex = ExampleUnnamedWith { 0: A, 1: B, 2: C };
    let bytes = ex.serialize();
    let deserialized = ExampleUnnamedWith::deserialize(&bytes[1..]);
    assert_eq!(deserialized, ex);
}

#[test]
#[cfg(feature = "bytemuck")]
fn test_unnamed_bytemuck_deserialize_opt() {
    let ex = ExampleUnnamedOpt {
        0: Some(A),
        1: None,
        2: Some(C),
    };
    let bytes = ex.serialize();
    let deserialized = ExampleUnnamedOpt::deserialize(&bytes[1..]);
    assert_eq!(deserialized, ex);
}

#[test]
#[cfg(feature = "bytemuck")]
fn test_unnamed_bytemuck_deserialize_req() {
    let ex = ExampleUnnamedReq { 0: A, 1: B, 2: C };
    let bytes = ex.serialize();
    let deserialized = ExampleUnnamedReq::deserialize(&bytes[1..]);
    assert_eq!(deserialized, ex);
}

#[test]
#[cfg(feature = "bytemuck")]
fn test_unnamed_bytemuck_deserialize_req_opt() {
    let ex = ExampleUnnamedReqOpt {
        0: Some(A),
        1: B,
        2: None,
    };
    let bytes = ex.serialize();
    let deserialized = ExampleUnnamedReqOpt::deserialize(&bytes[1..]);
    assert_eq!(deserialized, ex);
}
