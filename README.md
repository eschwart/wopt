# wopt (with-options)

## Description
A procedural macro that automatically generates an Option-wrapped version of a struct, reducing boilerplate for optional updates.

## Example
```rust
use wopt::*;

#[derive(WithOpt)]
#[wopt(derive(Debug, Clone, PartialEq))]
struct Example {
    a: u8,
    b: f32,
    c: String,
}

#[test]
fn main() {
    let mut ex_opt = ExampleOpt::default();
    ex_opt.b = Some(420.0);

    assert_eq!(
        ex_opt,
        ExampleOpt {
            a: None,
            b: Some(420.0),
            c: None
        },
    )
}
```

## Optional Features
| Name | Description |
| ---- | ----------- |
| [rkyv](https://crates.io/crates/bitflags) | Serialize/Deserialize using `rkyv` |
| [serde](https://crates.io/crates/serde) | Seriailze/Deserialize using `serde` |