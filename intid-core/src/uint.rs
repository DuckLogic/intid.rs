//! Re-exports [`primint::UnsignedPrimInt`] and various module level functions.
//!
//! Exists primarily for compatibility with older versions of this crate,
//! before [`primint`] was its own crate.
//!
//! It will be removed in the next semver-breaking release.

/// A trait alias for [`primint::UnsignedPrimInt`].
///
/// This is logically a re-export,
/// but a wrapper function is used instead as re-exports can't currently be deprecated ([rust-lang/rust#30827])
///
/// Since trait aliases are not usable on stable rust,
/// we use a blanket impl and trait bound instead.
///
/// This trait alias will be removed in the next semver-breaking release.
///
/// [rust-lang/rust#30827]: https://github.com/rust-lang/rust/issues/30827
#[deprecated(note = "Use primint::UnsignedPrimInt directly")]
pub trait UnsignedPrimInt: primint::UnsignedPrimInt {}
#[allow(deprecated)]
impl<T: primint::UnsignedPrimInt> UnsignedPrimInt for T {}

/// A type alias for [`primint::fmt::DebugDesc`].
///
/// This is logically a re-export,
/// but a type alias is used instead as re-exports can't currently be deprecated ([rust-lang/rust#30827])
///
/// This type alias will be removed in the next semver-breaking release.
///
/// [rust-lang/rust#30827]: https://github.com/rust-lang/rust/issues/30827
#[deprecated(note = "Use primint::fmt::DebugDesc directly")]
pub type DebugDesc<T> = primint::fmt::DebugDesc<T>;

macro_rules! deprecated_shim_fn {
    (@add_attrs $name:ident => $target:path {$decl:item}) => {
        #[doc = concat!("An alias for [`", stringify!($target), "`].")]
        ///
        /// This is logically a re-export,
        /// but a wrapper function is used instead as re-exports can't currently be deprecated ([rust-lang/rust#30827])
        ///
        /// This function will be removed in the next semver-breaking release.
        ///
        /// [rust-lang/rust#30827]: https://github.com/rust-lang/rust/issues/30827
        #[deprecated = concat!("Use ", stringify!($target), " directly.")]
        #[inline] // just a wrapper
        #[allow(deprecated)]
        $decl
    };
    (@determine_target $name:ident $target:path) => ($target);
    (@determine_target $name:ident) => (primint::$name);
    ($($({$x:ident})? fn $name:ident $(<$($param:ident: $bound:ident),+>)? ($($arg:ident: $argty:ty),*) $(-> $ret:ty)?;)+) => {
        $(deprecated_shim_fn!(@add_attrs $name => primint::$name {
            pub $($x)? fn $name$(<$($param: $bound),*>)?($($arg: $argty,)*) $(-> $ret)? {
                primint::$name $(::<$($param, )*>)?($($arg),*)
            }
        });)*
    };
}
deprecated_shim_fn! {
    {const} fn bits<T: UnsignedPrimInt>() -> u32;
    fn checked_add<T: UnsignedPrimInt>(left: T, right: T) -> Option<T>;
    fn checked_cast<T: UnsignedPrimInt, U: UnsignedPrimInt>(value: T) -> Option<U>;
    fn checked_sub<T: UnsignedPrimInt>(left: T, right: T) -> Option<T>;
    fn count_ones<T: UnsignedPrimInt>(value: T) -> u32;
    fn from_usize_checked<T: UnsignedPrimInt>(value: usize) -> Option<T>;
    fn from_usize_wrapping<T: UnsignedPrimInt>(value: usize) -> T;
    fn leading_zeros<T: UnsignedPrimInt>(value: T) -> u32;
    {const} fn max_value<T: UnsignedPrimInt>() -> T;
    {const} fn one<T: UnsignedPrimInt>() -> T;
    fn to_usize_checked<T: UnsignedPrimInt>(value: T) -> Option<usize>;
    fn to_usize_wrapping<T: UnsignedPrimInt>(value: T) -> usize;
    fn trailing_zeros<T: UnsignedPrimInt>(value: T) -> u32;
    {const} fn zero<T: UnsignedPrimInt>() -> T;
}
deprecated_shim_fn! {
    @add_attrs debug_desc => primint::fmt::debug_desc {
        pub fn debug_desc<T: UnsignedPrimInt>(value: T) -> DebugDesc<T> {
            primint::fmt::debug_desc(value)
        }
    }
}

/// Panic with a message indicating that an ID is not valid.
///
/// Used to implement the panic in [`crate::IntegerId::from_int`].
///
/// This is the only functionality in this crate not re-exported from [`primint`].
#[inline(never)]
#[track_caller]
#[cold]
pub(crate) fn invalid_id<T: primint::UnsignedPrimInt>(id: T) -> ! {
    panic!("Invalid id: {}", primint::fmt::debug_desc(id))
}
