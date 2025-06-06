# wopt (with-options)

## Description
A procedural macro that automatically generates an Option-wrapped version of a struct, reducing boilerplate for optional updates.

## Example
```rust
use wopt::*;

#[derive(WithOpt)]
#[wopt(derive(Debug, Default, PartialEq))]
struct Example {
    a: u8,
    #[wopt(required)]
    b: f32,
    c: String,
}

fn main() {
    let b = 420.0;
    let mut ex_opt = ExampleOpt::default();
    ex_opt.b = b;

    assert_eq!(
        ex_opt,
        ExampleOpt {
            a: None,
            b,
            c: None
        },
    )
}
```

## Field Attributes
| Name | Description |
| ---- | ----------- |
| `required` | Does not wrap the specified field with an `Option`. |
| `skip` | Does not include the specified field. |


## Optional Feature(s)
| Name | Description |
| ---- | ----------- |
| [bytemuck](https://crates.io/crates/bitflags) | Serialize/Deserialize using `bytemuck`. |


## Additional Notes
The automatically generated optional-struct does not come with any trait/derivation implementations. The fields are publicized, however, it may be helpful to specify the `Default` trait:
```rust
#[derive(WithOpt)]
#[wopt(derive(Default))] // attempts to implement `Default`
struct ExampleWithDefault(u8);
```