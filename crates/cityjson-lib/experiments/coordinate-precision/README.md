Compile cjlib with either f32 or f64 coordinates.

However, the Cargo book said that such design should be avoided.

"There are rare cases where features may be mutually incompatible with one another. This should be avoided if at all possible, because it requires coordinating all uses of the package in the dependency graph to cooperate to avoid enabling them together." [Mutually exclusive features](https://doc.rust-lang.org/cargo/reference/features.html#mutually-exclusive-features)