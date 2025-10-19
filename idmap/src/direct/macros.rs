macro_rules! impl_direct_map_iter {
    ($target:ident<$($l:lifetime,)? $kt:ident: $key_bound:ident, $vt:ident> {
        fn map($k:ident, $v:ident) -> $item_ty:ty {
            $map:expr
        }
    }) => {
        impl<$($l,)* $kt: $key_bound, $vt> Iterator for $target<$($l,)* $kt, $vt> {
            type Item = $item_ty;
            #[inline]
            fn next(&mut self) -> Option<Self::Item> {
                loop {
                    match self.source.next() {
                        Some((index, Some($v))) => {
                            // SAFETY: Value exists => index is valid
                            let $k = unsafe {
                                $kt::from_int_unchecked(intid::uint::from_usize_wrapping(index))
                            };
                            self.len -= 1;
                            return Some($map)
                        },
                        Some((_, None)) => continue,
                        None => return None,
                    }
                }
            }
            #[inline]
            fn size_hint(&self) -> (usize, Option<usize>) {
                let len = self.len as usize;
                (len, Some(len))
            }
        }
        impl<$($l,)* $kt: $key_bound, $vt> DoubleEndedIterator for $target<$($l,)* $kt, $vt> {
            #[inline]
            fn next_back(&mut self) -> Option<Self::Item> {
                loop {
                    match self.source.next_back() {
                        Some((index, Some($v))) => {
                            // SAFETY: Value exists => index is valid
                            let $k = unsafe {
                                $kt::from_int_unchecked(intid::uint::from_usize_wrapping(index))
                            };
                            self.len -= 1;
                            return Some($map)
                        },
                        Some((_, None)) => continue,
                        None => return None,
                    }
                }
            }
        }
        impl<$($l,)* $kt: $key_bound, $vt> ExactSizeIterator for $target<$($l,)* $kt, $vt> {}
        impl<$($l,)* $kt: $key_bound, $vt> core::iter::FusedIterator for $target<$($l,)* $kt, $vt> {}
    }
}

macro_rules! impl_direct_set_iter {
    ($target:ident<$($lt:lifetime,)? $kt:ident: $key_bound:ident>) => {
        impl<$($lt,)* $kt: $key_bound> Iterator for $target<$($lt,)* $kt> {
            type Item = $kt;

            #[inline]
            fn next(&mut self) -> Option<Self::Item> {
                match self.handle.next() {
                    Some(index) => {
                        self.len -= 1;
                        // SAFETY: Id is present => id is valid
                        Some(unsafe { K ::from_int_unchecked(intid::uint::from_usize_wrapping(index)) })
                    }
                    None => {
                        debug_assert_eq!(self.len, 0);
                        None
                    }
                }
            }
            #[inline]
            fn size_hint(&self) -> (usize, Option<usize>) {
                (self.len, Some(self.len))
            }
            #[inline]
            fn count(self) -> usize
            where
                Self: Sized,
            {
                self.len
            }
        }
        impl<$($lt,)* T: IntegerId> DoubleEndedIterator for $target<$($lt,)* T> {
            #[inline]
            fn next_back(&mut self) -> Option<Self::Item> {
                match self.handle.next_back() {
                    Some(index) => {
                        self.len -= 1;
                        // SAFETY: Id is present => id is valid
                        Some(unsafe { T::from_int_unchecked(intid::uint::from_usize_wrapping(index)) })
                    }
                    None => {
                        debug_assert_eq!(self.len, 0);
                        None
                    }
                }
            }
        }
        impl<$($lt,)* T: IntegerId> ExactSizeIterator for $target<$($lt,)* T> {}
        impl<$($lt,)* T: IntegerId> FusedIterator for $target<$($lt,)* T> {}
    };
}

pub(crate) use {impl_direct_map_iter, impl_direct_set_iter};
