//! Test [`IdMap`] with various impls.

use primint::num::NonMax;

#[test]
fn nonmax() {
    fn new(x: u32) -> NonMax<u32> {
        NonMax::<u32>::new(x).unwrap()
    }
    let map = idmap::direct_idmap! {
        new(0) => "foo",
        new(1) => "bar",
        new(2) => "baz",
    };
    assert_eq!(map.get(new(0)).copied(), Some("foo"));
    assert_eq!(map.get(new(20)), None);
}
