#![feature(phase)]

#[phase(plugin)]
extern crate resources_package;

#[test]
fn test() {
    static package: &'static [(&'static [u8], &'static [u8])] = resources_package!(
        "fixture"
    );

    assert_eq!(package.len(), 3);
    assert!(Path::new(*package[0].ref0()) == Path::new("subdir").join("cc.txt"));
    assert_eq!(package[0].ref1(), &b"ccc");

    assert!(Path::new(*package[1].ref0()) == Path::new("b.txt"));
    assert_eq!(package[1].ref1(), &b"b b b");

    assert!(Path::new(*package[2].ref0()) == Path::new("aaa.txt"));
    assert_eq!(package[2].ref1(), &b"aaa\naaa");
}
