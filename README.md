Opinionated and very limited macro_rules reimplementation of the [typed_builder derive macro](https://docs.rs/typed-builder).

Currently only supports required fields and "private" (or "computed") fields and does not support Option stripping.
You can hack your way to "optional" fields though.

The struct you declare will be re-emitted as-is with all the custom syntax stripped
and with an impl block containing `::builder()` method, which returns an empty instance of `Builder`.
The builder implements `::build` method which is only implemented for builders with all fields filled in.
Fields with a default value don't have to be filled in before `::build` can be called.
(They're considered filled in, even if the user doesn't provide a value)

The syntax is pretty much "stock" Rust struct declaration with a few notable exceptions:
  - You can place `@` sign before a field declaration to make it "private" (i.e. it will not be part of the builder)
  - You can use `= expr` syntax after a field declaration to set the default value of a field
  - You can use `!` after the field name to force this field's setter to only accept specifically the type of that field.
    The default is `impl Into<T>`.

Private fields *have* to also specify the computed value.

# Caveats
Important implementation details:
  - All "default value expressions" are executed inside the Builder::build method.
    That means, when computing a value based on another field, you should refer to that field just by name,
    not by using self.field.
  - Field values are computed in order of declaration. If you want a field's default value to depend on another field,
    you have to declare it after.
  - Currently you cannot use this macro to declare a structure inside of a function or any other block item.
    It can only be called at module level.

# Examples
```rust
use typed_builder_rules::typed_builder;

typed_builder!(
  #[derive(Debug, PartialEq)]
  struct Foo {
    foo: String, // regular, required field. setter accepts `impl Into<String>`
    bar!: String, // reqular, required field. must use `String` in the setter
    baz: String = format!("{foo} {bar}"), // regular, required field. if not provided will take value `format!("{foo} {bar}")`
    @qux: u64 = {
      use std::hash::{DefaultHasher, Hash, Hasher};
      let mut s = DefaultHasher::new();
      foo.hash(&mut s);
      bar.hash(&mut s);
      baz.hash(&mut s);
      s.finish()
    }, // private field. only gets initialized inside of `::build()`
  }
);

fn main() {
  let expected = Foo {
    foo: "foo".to_string(),
    bar: "bar".to_string(),
    baz: "foo bar".to_string(),
    qux: 10576444654000555064,
  };

  let actual = Foo::builder()
    .foo("foo")
    .bar("bar".to_string())
    .build();

  assert_eq!(expected, actual);
}
```
