/// Defines a new type [`IntegerIdCounter`],
/// which wraps another [`IntegerIdCounter`]
///
/// This wraps the similar [`define_newtype_id!`] macro,
/// so it also derives [`IntegerId`], [`Copy`], [`Clone`], [`PartialEq`], [`Eq`], [`PartialOrd`], [`Ord`], [`Hash`], and [`Debug`].
///
/// This is more convenient than using `#[derive(IntegerId, IntegerIdCounter)]`,
/// because it also derives the secondary traits.
/// In addition, it reduces build time dependencies by avoiding procedural macros.
///
/// [`IntegerIdCounter`]: crate::IntegerIdCounter
/// [`IntegerId`]: crate::IntegerId
/// [`define_newtype_id!`]: crate::define_newtype_id
/// [`Hash`]: core::hash::Hash
/// [`Debug`]: core::fmt::Debug
#[macro_export]
macro_rules! define_newtype_counter {
    (
        $(#[$ty_attr:meta])*
        $vis:vis struct $name:ident($(#[$field_attr:meta])* $inner_vis:vis $inner:ty);
    ) => {
        $crate::define_newtype_id! {
            $(#[$ty_attr])*
            $vis struct $name($(#[$field_attr])* $inner_vis $inner);
        }
        impl $crate::IntegerIdContiguous for $name {
            const MIN_ID: Self = $name(<$inner as $crate::IntegerIdContiguous>::MIN_ID);
            const MAX_ID: Self = $name(<$inner as $crate::IntegerIdContiguous>::MAX_ID);
        }
        impl $crate::IntegerIdCounter for $name {
            const START: Self = $name(<$inner as $crate::IntegerIdCounter>::START);
            const START_INT: Self::Int = <$inner as $crate::IntegerIdCounter>::START_INT;
        }
    };
}

/// Defines a newtype [`IntegerId`], which wraps another  [`IntegerID`].
///
/// Automatically derives implementations of
///  [`Copy`], [`Clone`], [`PartialEq`], [`Eq`], [`PartialOrd`], [`Ord`], [`Hash`], and [`Debug`].
/// These traits are required for to implement [`crate::IntegerId`].
///
/// This is more convenient than using `#[derive(IntegerId)]`,
/// because it also derives the necessary secondary traits.
/// In addition, it reduces build time dependencies by avoiding procedural macros.
///
/// See the similar [`define_newtype_counter!`] if you also wish to derive [`IntegerIdCounter`]
///
/// [`IntegerIdCounter`]: crate::IntegerIdCounter
/// [`IntegerId`]: crate::IntegerId
/// [`define_newtype_id!`]: crate::define_newtype_id
/// [`Hash`]: core::hash::Hash
/// [`Debug`]: core::fmt::Debug
#[macro_export]
macro_rules! define_newtype_id {
    (
        $(#[$ty_attr:meta])*
        $vis:vis struct $name:ident($(#[$field_attr:meta])* $inner_vis:vis $inner:ty);
    ) => {
        $(#[$ty_attr])*
        #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
        #[repr(transparent)]
        $vis struct $name($(#[$field_attr])* $inner_vis $inner);
        impl $crate::IntegerId for $name {
            type Int = <$inner as intid::IntegerId>::Int;
            #[inline]
            fn from_int(id: Self::Int) -> Self {
                $name(<$inner as $crate::IntegerId>::from_int(id))
            }
            #[inline]
            fn from_int_checked(id: Self::Int) -> Option<Self> {
                Some($name(<$inner as $crate::IntegerId>::from_int_checked(id)?))
            }
            #[inline]
            unsafe fn from_int_unchecked(id: Self::Int) -> Self {
                $name({
                    // SAFETY: Guaranteed by the caller
                    unsafe { <$inner as $crate::IntegerId>::from_int_unchecked(id) }
                })
            }
            #[inline]
            fn to_int(self) -> Self::Int {
                $crate::IntegerId::to_int(self.0)
            }
        }
    };
}
