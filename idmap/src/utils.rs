use alloc::boxed::Box;
use core::alloc::Layout;
use core::mem::MaybeUninit;

pub mod bitsets;

macro_rules! box_uninit_alloc_impl {
    (for $tp:ident {
        Box::new(MaybeUninit::$explicit_create:ident),
        unsafe { Box::from_raw(std::alloc::$alloc_func:ident) }
    }) => {{
        let layout = Layout::new::<$tp>();
        if layout.size() == 0 {
            // this does not move any memory because `T` is a ZST
            Box::new(MaybeUninit::$explicit_create())
        } else {
            // SAFETY: Not a zero sized type
            let allocated = unsafe { alloc::alloc::alloc(layout) }.cast::<MaybeUninit<$tp>>();
            if allocated.is_null() {
                alloc::alloc::handle_alloc_error(layout)
            } else {
                // SAFETY: Allocated using the regular global allocator
                // No need to initialize since the return type is `MaybeUninit`
                unsafe { Box::from_raw(allocated) }
            }
        }
    }};
}

/// A polyfill for [`Box::new_uninit`].
///
/// Useful for allocating memory in-place without copying.
#[inline]
pub fn box_alloc_uninit<T>() -> Box<MaybeUninit<T>> {
    box_uninit_alloc_impl!(for T {
        Box::new(MaybeUninit::uninit),
        // SAFETY: Allocated using the global allocator.
        // Since the return type is `MaybeUninit`, the contents don't matter
        unsafe { Box::from_raw(std::alloc::alloc) }
    })
}

/// A polyfill for [`Box::new_zeroed`].
///
/// Useful for allocating memory in-place without copying.
#[inline]
pub fn box_alloc_zeroed<T>() -> Box<MaybeUninit<T>> {
    box_uninit_alloc_impl!(for T {
        Box::new(MaybeUninit::uninit),
        // SAFETY: Allocated using the global allocator.
        // Since the return type is `MaybeUninit`,
        // the zero-initialization of the contents doesn't matter
        unsafe { Box::from_raw(std::alloc::alloc_zeroed) }
    })
}

/// A polyfill for [`Box<MaybeUninit<T>>::assume_init`].
///
/// # Safety
/// Undefined behavior if the memory is not initialized.
#[inline]
pub unsafe fn box_assume_init<T>(value: Box<MaybeUninit<T>>) -> Box<T> {
    let ptr: *mut MaybeUninit<T> = Box::into_raw(value);
    // SAFETY: Initialization is guaranteed by the caller
    unsafe { Box::from_raw(ptr.cast::<T>()) }
}

/// Indicates that a type can be zero-initialized.
///
/// This is equivalent to the [`bytemuck::Zeroable`] trait,
/// but is an implementation detail that is not exposed publicly.
///
/// [`bytemuck::Zeroable`]: https://docs.rs/bytemuck/1/bytemuck/trait.Zeroable.html
///
/// # Safety
/// The type must be valid to initialize with zeroes.
///
/// Must not override any of the inherent methods.
pub(crate) unsafe trait Zeroable: Sized {
    #[inline]
    fn zeroed_boxed() -> Box<Self> {
        let zeroed = box_alloc_zeroed();
        // SAFETY: Implementation of the trait means that Self can be zero initialized
        unsafe { box_assume_init(zeroed) }
    }
    #[inline]
    fn zeroed() -> Self {
        // SAFETY: We know that this type can be zero initialized
        unsafe { core::mem::zeroed() }
    }
}
