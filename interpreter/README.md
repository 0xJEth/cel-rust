# CEL Interpreter

[![Rust](https://github.com/clarkmcc/cel-rust/actions/workflows/rust.yml/badge.svg)](https://github.com/clarkmcc/cel-rust/actions/workflows/rust.yml)

The [Common Expression Language (CEL)](https://github.com/google/cel-spec) is a non-Turing complete language designed
for simplicity, speed, safety, and
portability. CEL's C-like syntax looks nearly identical to equivalent expressions in C++, Go, Java, and TypeScript. CEL
is ideal for lightweight expression evaluation when a fully sandboxed scripting language is too resource intensive.

```java
// Check whether a resource name starts with a group name.
resource.name.startsWith("/groups/" + auth.claims.group)
```

```go
// Determine whether the request is in the permitted time window.
request.time - resource.age < duration("24h")
```

```typescript
// Check whether all resource names in a list match a given filter.
auth.claims.email_verified && resources.all(r, r.startsWith(auth.claims.email))
```

## Getting Started

This project includes a parser and an interpreter which means that it can be used to evaluate CEL-expressions. The
library aims to be very simple to use, while still being fast, safe, and customizable.

```rust
use cel_interpreter::{Context, Program};

fn main() {
    // Compile a CEL program
    let program = Program::compile("add(2, 3)").unwrap();

    // Add any variables or functions that the program will need
    let mut context = Context::default();
    context.add_function("add", |a: i64, b: i64| a + b);

    // Run the program
    let value = program.execute(&context).unwrap();
    assert_eq!(value, 5.into());
}
```

### Examples

Check out these other examples to learn how to use this library:

- [Simple](../example/src/simple.rs) - A simple example of how to use the library.
- [Variables](../example/src/variables.rs) - Passing variables and using them in your program.
- [Functions](../example/src/functions.rs) - Defining and using custom functions in your program.
- [Concurrent Execution](../example/src/threads.rs) - Executing the same program concurrently.
