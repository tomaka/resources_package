#![feature(plugin)]

#![plugin(resources_package)]

extern crate resources_package_package;

#[test]
fn test() {
    use std::path::Path;

    static PACKAGE: resources_package_package::Package = resources_package!(
        "fixture"
    );

    assert_eq!(PACKAGE.iter().count(), 3);

    // TODO: drop "[..]" when this bug is fixed:
    //    https://github.com/rust-lang/rust/issues/22649

    assert_eq!(PACKAGE.iter().find(|&(ref path, _)| path.as_path() == Path::new("aaa.txt"))
        .map(|(_, ctnt)| ctnt), Some(&b"aaa\naaa"[..]));

    assert_eq!(PACKAGE.iter().find(|&(ref path, _)| path.as_path() == Path::new("b.txt"))
        .map(|(_, ctnt)| ctnt), Some(&b"b b b"[..]));

    assert_eq!(PACKAGE.iter().find(|&(ref path, _)| path.as_path() == Path::new("subdir").join("cc.txt").as_path())
        .map(|(_, ctnt)| ctnt), Some(&b"ccc"[..]));

    assert_eq!(PACKAGE.find(Path::new("aaa.txt")), Some(&b"aaa\naaa"[..]));
    assert_eq!(PACKAGE.find(Path::new("b.txt")), Some(&b"b b b"[..]));
    assert_eq!(PACKAGE.find(Path::new("subdir").join("cc.txt").as_path()), Some(&b"ccc"[..]));
}
