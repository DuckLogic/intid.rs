#![allow(missing_docs)]

intid::define_newtype_id! {
    /// Docs should work fine.
    pub struct Plain(u32);
}

intid::define_newtype_counter! {
    /// So should other derive marcos.
    #[derive(Default)]
    pub struct Counter(u32);
}

#[test]
fn verify_derive() {
    assert_id::<Plain>();
    assert_counter::<Counter>();
}

fn assert_id<T: intid::IntegerId>() {}
fn assert_contiguous<T: intid::IntegerIdContiguous>() {
    assert_id::<T>();
}
fn assert_counter<T: intid::IntegerIdCounter>() {
    assert_contiguous::<T>();
    assert_eq!(T::START.to_int(), T::START_INT);
}
