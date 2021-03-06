//! Array multiple elements constructor syntax.
//!
//! While Rust does provide those, they require copy, and you cannot obtain the
//! index that will be created. This crate provides syntax that fixes both of
//! those issues.
//!
//! # Examples
//!
//! ```
//! # #[macro_use]
//! # extern crate array_macro;
//! # fn main() {
//! assert_eq!(array![String::from("x"); 2], [String::from("x"), String::from("x")]);
//! assert_eq!(array![|x| x; 3], [0, 1, 2]);
//! # }
//! ```

#![no_std]

#[doc(hidden)]
pub extern crate core as __core;

/// Array constructor macro.
///
/// This macro provides a way to repeat the same macro element multiple times
/// without requiring `Copy` implementation.
///
/// It's possible to define a callback by starting expression with `|` or `move`. As
/// every closure is it own unique type, it is not possible to have an array of
/// closures, so this syntax was reused for creating arrays with known indexes.
///
/// # Examples
///
/// ```
/// # #[macro_use]
/// # extern crate array_macro;
/// # fn main() {
/// assert_eq!(array!["string"; 3], ["string", "string", "string"]);
/// assert_eq!(array![|x| x; 3], [0, 1, 2]);
/// # }
/// ```
#[macro_export]
macro_rules! array {
    [@INTERNAL $callback:expr; $count:expr] => {{
        #[allow(unused_mut)]
        let mut callback = $callback;
        #[allow(unsafe_code)]
        unsafe {
            struct ArrayVec<'a, T: 'a> {
                slice: &'a mut [T],
                position: usize,
            }
            impl<'a, T: 'a> Drop for ArrayVec<'a, T> {
                fn drop(&mut self) {
                    for i in 0..self.position {
                        unsafe {
                            $crate::__core::ptr::drop_in_place(
                                self.slice.get_unchecked_mut(i)
                            );
                        }
                    }
                }
            }
            fn needs_drop<T>(_: &T) -> bool {
                $crate::__core::mem::needs_drop::<T>()
            }
            let arr: [_; $count] = $crate::__core::mem::uninitialized();
            let needs_drop = needs_drop(&arr);
            let mut arr = $crate::__core::mem::ManuallyDrop::new(arr);
            if needs_drop {
                let mut vec = ArrayVec { slice: &mut *arr, position: 0 };
                for (i, elem) in vec.slice.iter_mut().enumerate() {
                    vec.position = i;
                    $crate::__core::ptr::write(elem, callback(i));
                }
                $crate::__core::mem::forget(vec);
            } else {
                for (i, elem) in arr.iter_mut().enumerate() {
                    $crate::__core::ptr::write(elem, callback(i));
                }
            }
            $crate::__core::mem::ManuallyDrop::into_inner(arr)
        }
    }};
    [| $($rest:tt)*] => {
        array![@INTERNAL | $($rest)*]
    };
    [move $($rest:tt)*] => {
        array![@INTERNAL move $($rest)*]
    };
    [$expr:expr; $count:expr] => {
        array![|_| $expr; $count]
    };
}
