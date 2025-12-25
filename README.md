# positive

## Positive

A type-safe wrapper for guaranteed positive decimal values.

This crate provides the `Positive` type, which encapsulates a `Decimal` value and ensures
through its API that the contained value is always positive (greater than or equal to zero).

### Features

- **Type Safety**: Compile-time guarantees that values are non-negative
- **Decimal Precision**: Built on `rust_decimal` for accurate financial calculations
- **Rich API**: Comprehensive arithmetic operations, conversions, and utilities
- **Serde Support**: Full serialization/deserialization support

### Example

```rust
use positive::{Positive, pos};

let price = pos!(100.50);
let quantity = Positive::new(10.0).unwrap();
let total = price * quantity;
```

License: MIT
