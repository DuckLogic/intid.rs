use alloc::boxed::Box;
use core::alloc::Layout;
use core::mem::MaybeUninit;

/// A polyfill for [`Box::new_uninit`].
///
/// Useful for allocating memory in-place without copying.
#[inline]
pub fn box_alloc_uninit<T>() -> Box<MaybeUninit<T>> {
    let layout = Layout::new::<T>();
    if layout.size() == 0 {
        // this does not move any memory because `T` is a ZST
        Box::new(MaybeUninit::uninit())
    } else {
        // SAFETY: Not a zero sized type
        let allocated = unsafe { alloc::alloc::alloc(layout) }.cast::<MaybeUninit<T>>();
        if allocated.is_null() {
            alloc::alloc::handle_alloc_error(layout)
        } else {
            // SAFETY: Allocated using the regular global allocator
            // No need to initialize since the return type is `MaybeUninit`
            unsafe { Box::from_raw(allocated) }
        }
    }
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
