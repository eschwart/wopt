mod common;
use common::unnamed::*;

#[test]
fn test_named_stable() {
    let mut ex = ExampleUnnamed { 0: A, 1: B, 2: C };

    let mut ex_opt = ExampleUnnamedOpt::default();
    assert!(!ex_opt.is_modified());

    ex_opt.0 = Some(1);
    assert!(ex_opt.is_modified());

    ex.patch(&mut ex_opt);
    assert!(!ex_opt.is_modified());

    assert_eq!(ex, ExampleUnnamed { 0: 1, 1: B, 2: C })
}

#[test]
fn test_named_stable_req() {
    let mut ex = ExampleUnnamedReq { 0: A, 1: B, 2: C };

    let mut ex_opt = ExampleUnnamedReqOpt::default();
    assert!(!ex_opt.is_modified());

    ex_opt.0 = Some(1);
    ex_opt.1 = 2.0;
    assert!(ex_opt.is_modified());

    ex.patch(&mut ex_opt);
    assert!(!ex_opt.is_modified());
    assert_eq!(ex_opt.1, 2.0);

    assert_eq!(ex, ExampleUnnamedReq { 0: 1, 1: B, 2: C })
}

#[test]
#[cfg(feature = "bytemuck")]
fn test_named_stable_flat() {
    let mut ex = ExampleUnnamedFlat {
        0: A,
        1: ExampleUnnamed { 0: A, 1: B, 2: C },
        2: C,
    };

    let mut ex_opt = ExampleUnnamedFlatOpt::default();
    assert!(!ex_opt.is_modified());

    ex_opt.0 = Some(1);
    ex_opt.1.2 = Some(-3);
    assert!(ex_opt.is_modified());

    ex.patch(&mut ex_opt);
    assert!(!ex_opt.is_modified());

    assert_eq!(
        ex,
        ExampleUnnamedFlat {
            0: 1,
            1: ExampleUnnamed { 0: A, 1: B, 2: -3 },
            2: C
        }
    )
}

#[test]
#[cfg(feature = "bytemuck")]
fn test_named_bytemuck_serialize() {
    let ex = ExampleUnnamed { 0: A, 1: B, 2: C };
    let serialized = ex.serialize();
    assert_eq!(
        [ExampleUnnamed::ID, 69, 0, 0, 210, 67, 0, 248, 255, 255].as_slice(),
        serialized,
    );
}

#[test]
#[cfg(feature = "bytemuck")]
fn test_named_bytemuck_serialize_with() {
    let ex = ExampleUnnamedWith { 0: A, 1: B, 2: C };
    let serialized = ex.serialize();
    assert_eq!(
        [ExampleUnnamedWith::ID, 69, 0, 0, 210, 67, 0, 248, 255, 255].as_slice(),
        serialized,
    );
}

#[test]
#[cfg(feature = "bytemuck")]
fn test_named_bytemuck_serialize_opt() {
    let ex = ExampleUnnamedOpt {
        0: Some(A),
        1: None,
        2: Some(C),
    };
    let serialized = ex.serialize();
    assert_eq!(
        [ExampleUnnamedOpt::ID, 5, 69, 0, 248, 255, 255].as_slice(),
        serialized
    );
}

#[test]
#[cfg(feature = "bytemuck")]
fn test_named_bytemuck_serialize_req() {
    let ex = ExampleUnnamedReq { 0: A, 1: B, 2: C };
    let serialized = ex.serialize();
    assert_eq!(
        [ExampleUnnamedReq::ID, 69, 0, 0, 210, 67, 0, 248, 255, 255].as_slice(),
        serialized,
    );
}

#[test]
#[cfg(feature = "bytemuck")]
fn test_named_bytemuck_serialize_req_opt() {
    let ex = ExampleUnnamedReqOpt {
        0: Some(A),
        1: B,
        2: None,
    };
    let serialized = ex.serialize();
    assert_eq!(
        [ExampleUnnamedReqOpt::ID, 1, 69, 0, 0, 210, 67].as_slice(),
        serialized
    );
}

#[test]
#[cfg(feature = "bytemuck")]
fn test_named_bytemuck_deserialize() {
    let ex = ExampleUnnamed { 0: A, 1: B, 2: C };
    let bytes = ex.serialize();
    let deserialized = ExampleUnnamed::deserialize(&bytes[1..]);
    assert_eq!(ex, deserialized);
}

#[test]
#[cfg(feature = "bytemuck")]
fn test_named_bytemuck_deserialize_with() {
    let ex = ExampleUnnamedWith { 0: A, 1: B, 2: C };
    let bytes = ex.serialize();
    let deserialized = ExampleUnnamedWith::deserialize(&bytes[1..]);
    assert_eq!(ex, deserialized);
}

#[test]
#[cfg(feature = "bytemuck")]
fn test_named_bytemuck_deserialize_opt() {
    let ex = ExampleUnnamedOpt {
        0: Some(A),
        1: None,
        2: Some(C),
    };
    let bytes = ex.serialize();
    let deserialized = ExampleUnnamedOpt::deserialize(&bytes[1..]);
    assert_eq!(ex, deserialized);
}

#[test]
#[cfg(feature = "bytemuck")]
fn test_named_bytemuck_deserialize_req() {
    let ex = ExampleUnnamedReq { 0: A, 1: B, 2: C };
    let bytes = ex.serialize();
    let deserialized = ExampleUnnamedReq::deserialize(&bytes[1..]);
    assert_eq!(ex, deserialized);
}

#[test]
#[cfg(feature = "bytemuck")]
fn test_named_bytemuck_deserialize_req_opt() {
    let ex = ExampleUnnamedReqOpt {
        0: Some(A),
        1: B,
        2: None,
    };
    let bytes = ex.serialize();
    let deserialized = ExampleUnnamedReqOpt::deserialize(&bytes[1..]);
    assert_eq!(ex, deserialized);
}

#[test]
#[cfg(feature = "bytemuck")]
fn test_named_bytemuck_serialize_flat() {
    let ex = ExampleUnnamedFlat {
        0: A,
        1: ExampleUnnamed { 0: A, 1: B, 2: C },
        2: C,
    };
    let bytes = ex.serialize();
    assert_eq!(
        [
            ExampleUnnamedFlat::ID,
            69,
            69,
            0,
            0,
            210,
            67,
            0,
            248,
            255,
            255,
            0,
            248,
            255,
            255
        ]
        .as_slice(),
        bytes
    );
}

#[test]
#[cfg(feature = "bytemuck")]
fn test_named_bytemuck_serialize_flat_opt() {
    let ex = ExampleUnnamedFlatOpt {
        0: Some(A),
        1: ExampleUnnamedOpt {
            0: Some(A),
            1: Some(B),
            2: Some(C),
        },
        2: Some(C),
    };
    let bytes = ex.serialize();
    assert_eq!(
        [
            ExampleUnnamedFlatOpt::ID,
            7,
            69,
            7,
            69,
            0,
            0,
            210,
            67,
            0,
            248,
            255,
            255,
            0,
            248,
            255,
            255
        ]
        .as_slice(),
        bytes
    );
}

#[test]
#[cfg(feature = "bytemuck")]
fn test_named_bytemuck_deserialize_flat() {
    let ex = ExampleUnnamedFlat {
        0: A,
        1: ExampleUnnamed { 0: A, 1: B, 2: C },
        2: C,
    };
    let bytes = ex.serialize();
    let deserialized = ExampleUnnamedFlat::deserialize(&bytes[1..]);
    assert_eq!(ex, deserialized);
}

#[test]
#[cfg(feature = "bytemuck")]
fn test_named_bytemuck_deserialize_flat_opt() {
    let ex = ExampleUnnamedFlatOpt {
        0: Some(A),
        1: ExampleUnnamedOpt {
            0: Some(A),
            1: Some(B),
            2: Some(C),
        },
        2: Some(C),
    };
    let bytes = ex.serialize();
    let deserialized = ExampleUnnamedFlatOpt::deserialize(&bytes[1..]);
    assert_eq!(ex, deserialized);
}
