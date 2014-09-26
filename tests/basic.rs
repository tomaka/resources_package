#![feature(phase)]

#[phase(plugin)]
extern crate resources_package;
extern crate resources_package_package;

#[test]
fn test() {
    static package: resources_package_package::Package = resources_package!(
        "fixture"
    );

    assert_eq!(package.iter().count(), 3);

    assert_eq!(package.iter().find(|&(ref path, _)| path == &Path::new("aaa.txt"))
        .map(|(_, ctnt)| ctnt).as_ref(), Some(&b"aaa\naaa"));

    assert_eq!(package.iter().find(|&(ref path, _)| path == &Path::new("b.txt"))
        .map(|(_, ctnt)| ctnt).as_ref(), Some(&b"b b b"));

    assert_eq!(package.iter().find(|&(ref path, _)| path == &Path::new("subdir").join("cc.txt"))
        .map(|(_, ctnt)| ctnt).as_ref(), Some(&b"ccc"));

    assert_eq!(package.find(&Path::new("aaa.txt")).as_ref(), Some(&b"aaa\naaa"));
    assert_eq!(package.find(&Path::new("b.txt")).as_ref(), Some(&b"b b b"));
    assert_eq!(package.find(&Path::new("subdir").join("cc.txt")).as_ref(), Some(&b"ccc"));
}
