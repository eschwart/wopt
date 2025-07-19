mod common;
use common::named::*;

#[test]
fn test_named_stable() {
    let mut ex = ExampleNamed { a: A, b: B, c: C };

    let mut ex_opt = ExampleNamedOpt::default();
    assert!(!ex_opt.is_modified());

    ex_opt.a = Some(1);
    assert!(ex_opt.is_modified());

    ex.patch(&mut ex_opt);
    assert!(!ex_opt.is_modified());

    assert_eq!(ex, ExampleNamed { a: 1, b: B, c: C })
}

#[test]
fn test_named_stable_req() {
    let mut ex = ExampleNamedReq { a: A, b: B, c: C };

    let mut ex_opt = ExampleNamedReqOpt::default();
    assert!(!ex_opt.is_modified());

    ex_opt.a = Some(1);
    ex_opt.b = 2.0;
    assert!(ex_opt.is_modified());

    ex.patch(&mut ex_opt);
    assert!(!ex_opt.is_modified());
    assert_eq!(ex_opt.b, 2.0);

    assert_eq!(ex, ExampleNamedReq { a: 1, b: B, c: C })
}

#[test]
#[cfg(feature = "bytemuck")]
fn test_named_stable_flat() {
    let mut ex = ExampleNamedFlat {
        a: A,
        b: ExampleNamed { a: A, b: B, c: C },
        c: C,
    };

    let mut ex_opt = ExampleNamedFlatOpt::default();
    assert!(!ex_opt.is_modified());

    ex_opt.a = Some(1);
    ex_opt.b.c = Some(-3);
    assert!(ex_opt.is_modified());

    ex.patch(&mut ex_opt);
    assert!(!ex_opt.is_modified());

    assert_eq!(
        ex,
        ExampleNamedFlat {
            a: 1,
            b: ExampleNamed { a: A, b: B, c: -3 },
            c: C
        }
    )
}

#[test]
#[cfg(feature = "bytemuck")]
fn test_named_bytemuck_serialize() {
    let ex = ExampleNamed { a: A, b: B, c: C };
    let serialized = ex.serialize();
    assert_eq!(
        [ExampleNamed::ID, 69, 0, 0, 210, 67, 0, 248, 255, 255].as_slice(),
        serialized,
    );
}

#[test]
#[cfg(feature = "bytemuck")]
fn test_named_bytemuck_serialize_with() {
    let ex = ExampleNamedWith { a: A, b: B, c: C };
    let serialized = ex.serialize();
    assert_eq!(
        [ExampleNamedWith::ID, 69, 0, 0, 210, 67, 0, 248, 255, 255].as_slice(),
        serialized,
    );
}

#[test]
#[cfg(feature = "bytemuck")]
fn test_named_bytemuck_serialize_opt() {
    let ex = ExampleNamedOpt {
        a: Some(A),
        b: None,
        c: Some(C),
    };
    let serialized = ex.serialize();
    assert_eq!(
        [ExampleNamedOpt::ID, 5, 69, 0, 248, 255, 255].as_slice(),
        serialized
    );
}

#[test]
#[cfg(feature = "bytemuck")]
fn test_named_bytemuck_serialize_req() {
    let ex = ExampleNamedReq { a: A, b: B, c: C };
    let serialized = ex.serialize();
    assert_eq!(
        [ExampleNamedReq::ID, 69, 0, 0, 210, 67, 0, 248, 255, 255].as_slice(),
        serialized,
    );
}

#[test]
#[cfg(feature = "bytemuck")]
fn test_named_bytemuck_serialize_req_opt() {
    let ex = ExampleNamedReqOpt {
        a: Some(A),
        b: B,
        c: None,
    };
    let serialized = ex.serialize();
    assert_eq!(
        [ExampleNamedReqOpt::ID, 1, 69, 0, 0, 210, 67].as_slice(),
        serialized
    );
}

#[test]
#[cfg(feature = "bytemuck")]
fn test_named_bytemuck_deserialize() {
    let ex = ExampleNamed { a: A, b: B, c: C };
    let bytes = ex.serialize();
    let deserialized = ExampleNamed::deserialize(&bytes[1..]);
    assert_eq!(ex, deserialized);
}

#[test]
#[cfg(feature = "bytemuck")]
fn test_named_bytemuck_deserialize_with() {
    let ex = ExampleNamedWith { a: A, b: B, c: C };
    let bytes = ex.serialize();
    let deserialized = ExampleNamedWith::deserialize(&bytes[1..]);
    assert_eq!(ex, deserialized);
}

#[test]
#[cfg(feature = "bytemuck")]
fn test_named_bytemuck_deserialize_opt() {
    let ex = ExampleNamedOpt {
        a: Some(A),
        b: None,
        c: Some(C),
    };
    let bytes = ex.serialize();
    let deserialized = ExampleNamedOpt::deserialize(&bytes[1..]);
    assert_eq!(ex, deserialized);
}

#[test]
#[cfg(feature = "bytemuck")]
fn test_named_bytemuck_deserialize_req() {
    let ex = ExampleNamedReq { a: A, b: B, c: C };
    let bytes = ex.serialize();
    let deserialized = ExampleNamedReq::deserialize(&bytes[1..]);
    assert_eq!(ex, deserialized);
}

#[test]
#[cfg(feature = "bytemuck")]
fn test_named_bytemuck_deserialize_req_opt() {
    let ex = ExampleNamedReqOpt {
        a: Some(A),
        b: B,
        c: None,
    };
    let bytes = ex.serialize();
    let deserialized = ExampleNamedReqOpt::deserialize(&bytes[1..]);
    assert_eq!(ex, deserialized);
}

#[test]
#[cfg(feature = "bytemuck")]
fn test_named_bytemuck_serialize_flat() {
    let ex = ExampleNamedFlat {
        a: A,
        b: ExampleNamed { a: A, b: B, c: C },
        c: C,
    };
    let bytes = ex.serialize();
    assert_eq!(
        [
            ExampleNamedFlat::ID,
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
    let ex = ExampleNamedFlatOpt {
        a: Some(A),
        b: ExampleNamedOpt {
            a: Some(A),
            b: Some(B),
            c: Some(C),
        },
        c: Some(C),
    };
    let bytes = ex.serialize();
    assert_eq!(
        [
            ExampleNamedFlatOpt::ID,
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
    let ex = ExampleNamedFlat {
        a: A,
        b: ExampleNamed { a: A, b: B, c: C },
        c: C,
    };
    let bytes = ex.serialize();
    let deserialized = ExampleNamedFlat::deserialize(&bytes[1..]);
    assert_eq!(ex, deserialized);
}

#[test]
#[cfg(feature = "bytemuck")]
fn test_named_bytemuck_deserialize_flat_opt() {
    let ex = ExampleNamedFlatOpt {
        a: Some(A),
        b: ExampleNamedOpt {
            a: Some(A),
            b: Some(B),
            c: Some(C),
        },
        c: Some(C),
    };
    let bytes = ex.serialize();
    let deserialized = ExampleNamedFlatOpt::deserialize(&bytes[1..]);
    assert_eq!(ex, deserialized);
}

#[test]
#[cfg(feature = "bytemuck")]
fn test_named_bytemuck_serialize_vec() {
    let ex = ExampleNamedVec {
        a: A,
        b: vec![1, 2, 3, 4],
        c: C,
    };
    let bytes = ex.serialize();
    assert_eq!(
        [ExampleNamedVec::ID, 69, 4, 0, 1, 2, 3, 4, 0, 248, 255, 255].as_slice(),
        bytes
    );
}

#[test]
#[cfg(feature = "bytemuck")]
fn test_named_bytemuck_deserialize_vec() {
    let ex = ExampleNamedVec {
        a: A,
        b: vec![1, 2, 3, 4],
        c: C,
    };
    let bytes = ex.serialize();
    let deserialized = ExampleNamedVec::deserialize(&bytes[1..]);
    assert_eq!(ex, deserialized);
}

#[test]
#[cfg(feature = "bytemuck")]
fn test_named_bytemuck_serialize_vec_opt() {
    let ex = ExampleNamedVecOpt {
        a: Some(A),
        b: Some(vec![1, 2, 3, 4]),
        c: Some(C),
    };
    let bytes = ex.serialize();
    assert_eq!(
        [
            ExampleNamedVecOpt::ID,
            ExampleNamedVecOptUnit::all().bits(),
            69,
            4,
            0,
            1,
            2,
            3,
            4,
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
fn test_named_bytemuck_deserialize_vec_opt() {
    let ex = ExampleNamedVecOpt {
        a: Some(A),
        b: Some(vec![1, 2, 3, 4]),
        c: Some(C),
    };
    let bytes = ex.serialize();
    let deserialized = ExampleNamedVecOpt::deserialize(&bytes[1..]);
    assert_eq!(ex, deserialized);
}
