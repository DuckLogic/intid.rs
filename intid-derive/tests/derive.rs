#![allow(missing_docs)]

#[derive(Copy, Clone, Debug, Eq, PartialEq, intid_derive::IntegerId, intid_derive::EnumId)]
pub enum Letter {
    A,
    B,
    C,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, intid_derive::IntegerId)]
pub struct Plain(u64);

#[derive(
    Copy, Clone, Debug, Eq, PartialEq, intid_derive::IntegerId, intid_derive::IntegerIdCounter,
)]
pub struct Counter(u32);

#[derive(Copy, Clone, Debug, Eq, PartialEq, intid_derive::IntegerId, intid_derive::EnumId)]
enum Void {}

#[test]
fn verify_derive() {
    assert_id::<Letter>();
    assert_id::<Plain>();
    assert_id::<Void>();
    assert_counter::<Counter>();
    assert_enum::<Letter>();
    assert_enum::<Void>();
}

fn assert_id<T: intid::IntegerId>() {}
fn assert_contiguous<T: intid::IntegerIdContiguous>() {
    assert_id::<T>();
}
fn assert_counter<T: intid::IntegerIdCounter>() {
    assert_contiguous::<T>();
    assert_eq!(T::START.to_int(), T::START_INT);
}
fn assert_enum<T: intid::EnumId>() {
    use intid::array::Array;
    assert_id::<T>();
    assert_eq!(
        T::Array::<()>::LEN,
        T::MAX_ID_INT.map_or(0, |x| intid::uint::to_usize_checked(x).unwrap() + 1)
    );
}
