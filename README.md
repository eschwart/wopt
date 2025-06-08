# wopt (with-options)

## Description
A procedural macro that automatically generates an Option-wrapped version of a struct, reducing boilerplate for optional updates.

## Example
```rust
use wopt::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, WithOpt)]
#[wopt(derive(Debug, Default, PartialEq))]
struct Example {
    a: u8,
    b: f32,
    c: i16,
}

fn main() {
    // original struct
    let mut ex = Example {
        a: 1,
        b: 2.0,
        c: -3,
    };

    // the original's optional struct
    let mut ex_opt = ExampleOpt {
        a: None,
        b: Some(420.0), // this will patch `ex.b`
        c: None,
    };
    // currently has a modification (b)
    assert!(ex_opt.is_modified());

    // "patch" the original with the optional struct.
    ex.patch(&mut ex_opt);

    // patching mutably "takes" the optional struct, meaning,
    // after patching it no longer has any modifications.
    assert!(!ex_opt.is_modified());

    assert_eq!(
        ex,
        Example {
            a: 1,
            b: 420.0, // the only field to change
            c: -3
        }
    )
}
```

## Field Attributes
For more information on how to use these attributes, refer to the structures in `tests\common\mod.rs`.
| Name | Description |
| ---- | ----------- |
| `optional` | Force the optional version of the current struct to use the optional version of the current field. |
| `ser`/`de` | Specify methods of serialization/deserialization (if specified, both are required). |
| `serde`    | Force the generated `serialize`/`deserialize` methods of the field (must derive `WithOpt`) to be used (usually paired with `optional`). |
| `required` | Does not wrap the specified field with an `Option`. |
| `skip`     | Does not include the current field. |


## Optional Feature(s)
| Name | Description |
| ---- | ----------- |
| [`bytemuck`](https://crates.io/crates/bitflags) | Serialize/Deserialize using `bytemuck`. |
| `unchecked` | Disable unwrap checks.


## Additional Notes
The automatically generated optional-struct does not come with any trait/derivation implementations. The fields are publicized, however, it may be helpful to specify the `Default` trait:
```rust
#[derive(WithOpt)]
#[wopt(derive(Default))] // attempts to implement `Default`
struct ExampleWithDefault(u8);
```