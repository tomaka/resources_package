This crate allows you to package several files in your executable.

This is similar to `include_bin!` but easier to use when you have
a lot of files.

Usage:

```rust
#![feature(phase)]

#[phase(plugin)]
extern crate resources_package;

static package: &'static [(&'static [u8], &'static [u8])] = resources_package!([
    "path/to/resources/*.png",
    "path/to/resources/*.mp3"
]);
```
