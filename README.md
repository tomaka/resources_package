This crate allows you to package several files in your executable.

This is similar to `include_bin!` but easier to use when you have
a lot of files.

Usage:

```rust
#![feature(phase)]

#[phase(plugin)]
extern crate resources_package;
extern crate resources_package_package;

static package: resources_package_package::Package = resources_package!([
    "path/to/resources",
    "other/path/to/resources"
]);

fn main() {
    for &(ref name, content) in package.iter() {
        println!("{}", name.display());
    }
}
```
