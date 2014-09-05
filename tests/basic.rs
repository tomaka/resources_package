#![feature(phase)]

#[phase(plugin)]
extern crate resources_package;

#[test]
fn test() {
    static package: &'static [(&'static [u8], &'static [u8])] = resources_package!(
        "fixture/*.txt"
    );

    assert_eq!(package.len(), 2);
    assert_eq!(package[0].ref0(), &b"fixture/aaa.txt");
    assert_eq!(package[0].ref1(), &b"aaa\naaa");

    assert_eq!(package[1].ref0(), &b"fixture/b.txt");
    assert_eq!(package[1].ref1(), &b"b b b");
}
