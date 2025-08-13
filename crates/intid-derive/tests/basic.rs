#![allow(missing_docs)]

#[derive(Copy, Clone, Debug, Eq, PartialEq, intid_derive::IntegerId)]
pub enum Letter {
    A,
    B,
    C,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, intid_derive::IntegerId)]
pub struct Plain(u64);

#[derive(Copy, Clone, Debug, Eq, PartialEq, intid_derive::IntegerId)]
#[intid(counter, contiguous)]
pub struct Counter(u32);

#[test]
fn verify_derive() {
    assert_id::<Letter>();
    assert_id::<Plain>();
    assert_counter::<Counter>();
}

fn assert_id<T: intid::IntegerId>() {}
fn assert_contiguous<T: intid::ContiguousIntegerId>() {
    assert_id::<T>();
}
fn assert_counter<T: intid::IntegerIdCounter>() {
    assert_contiguous::<T>();
    assert_eq!(T::START.to_int(), T::START_INT);
}
