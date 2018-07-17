#![no_std]

// Copyright 2018 lazy-static.rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0>. This file may not be
// copied, modified, or distributed except according to those terms.

/*! 
A macro for setting values to slices in batch

Using this macro it is possible to set values to slices
without setting each value individually

# Syntax

```ignore
set_slice! {
    SLICE: (SIZE) = VALUE;                           // move
    SLICE = VALUE_1, VALUE_2, VALUE_3, ...;          // list
    SLICE = copy REFERENCE;                          // copy ref
    SLICE = clone REFERENCE;                         // clone ref
    ...
}
```
## Variable Definitions
`SLICE: &mut [T]` = name of slice \
`VALUE: impl Deref<Target = [T]> | AsRef<[T]>` = value to be stored in slice \
`SIZE: usize` = a constexpr that specifies the size of the slice \
`REFERENCE: &[T]` = a reference to a slice \
`copy`/`clone` = an identifier that speficies how to handle REFERENCE

## Examples
```rust
# #[macro_use]
# extern crate set_slice;
# fn main() {
let mut slice = [0; 3]; // example slice
let vec = [-1, -2];


set_slice! {
    slice = 1, 2, 3;
    slice[..2] = copy &vec;
}

assert_eq!(slice, [-1, -2, 3]);
# }
```

# Semantics and Notes on Syntax

## move
the `VALUE` is moved into set_slice and dropped \
the contents of `VALUE` are stored into the slice

## list
the list: `VALUE_1`, `VALUE_2`, `VALUE_3`, ... is counted and converted into an array \
after conversion it is has the same semantics as move

## copy
the reference slice `&[T]` values are copied into the slice \
`T` must implement `Copy`

## clone
the reference slice `&[T]` values are cloned into the slice \
`T` must implement `Clone`
Cargo features
This crate provides two cargo features:

# Cargo features
This crate allows for use in no-std environment.
*/

#[doc(hidden)]
pub use core::ptr::swap as __swap_ptr;

#[macro_export(local_inner_macros)]
#[doc(hidden)]
macro_rules! count {
    ( )        => {0usize};
    ($one:tt) => {1usize};
    ($($pairs:tt $_p:tt)*) => {
        count!($($pairs)*) << 1usize
    };
    ($odd:tt $($rest:tt)*) => {
        count!($($rest)*) | 1usize
    };
}

#[macro_export(local_inner_macros)]
#[doc(hidden)]
macro_rules! __set_slice_internals {
    (copy $slice:expr, $value:expr) => {
        $slice.copy_from_slice($value);
    };
    (clone $slice:expr, $value:expr) => {
        $slice.clone_from_slice($value);
    };
    ($option:ident $slice:expr, $value:expr) => {
        compile_error!(stringify!(invalid option $option, valid options are copy, clone))
    };

    ($($ln:tt),* => $slice:expr, $size:expr, $value:expr) => {{
        const LINE: usize = count!($($ln)*);

        #[inline(always)]
        fn set<T>(slice: &mut [T], value: &[T]) {
            let (sl, vl) = (slice.len(), value.len());

            assert_eq!(sl, $size, "line {}: slice length ({}) is invalid, excepted: {}", LINE, sl, $size);
            assert_eq!(vl, $size, "line {}: value length ({}) is invalid, excepted: {}", LINE, vl, $size);
            
            let value = value as *const [T] as *mut [T] as *mut [T; $size];
            let slice = slice as *mut [T] as *mut [T; $size];
            
            unsafe { $crate::__swap_ptr(slice, value); }
        }

        let val = $value; // capture value
        set(&mut $slice, &val);
    }};
    ($($ln:tt),* => $slice:expr, $option:ident $value:expr) => {{
        let input = $value;
        let slice = &mut $slice;
        let (il, sl) = (input.len(), slice.len());

        assert_eq!(il, sl, "ln({}) input length invalid: {}, expected: {}", count!($($ln)*), il, sl);

        __set_slice_internals!($option slice, input);
    }};
}

/// a macro for setting parts of slices, see crate level docs for more info 
#[macro_export]
macro_rules! set_slice {
    (@$($ln:tt),* => $slice:ident: ($size:expr) = $value:expr; $($rest:tt)*) => {
        __set_slice_internals!($($ln),* => $slice, $size, $value);
        set_slice!(@$($ln,)* 0 => $($rest)*);
    };

    (@$($ln:tt),* => $slice:ident = $option:ident $value:expr; $($rest:tt)*) => {
        __set_slice_internals!($($ln),* => $slice, $option $value);
        set_slice!(@$($ln,)* 0 => $($rest)*);
    };

    (@$($ln:tt),* => $slice:ident = $($value:expr),+; $($rest:tt)*) => {
        __set_slice_internals!($($ln),* => $slice, count!($( $value )+), [$($value),+]);
        set_slice!(@$($ln,)* 0 => $($rest)*);
    };

    (@$($ln:tt),* => $slice:ident[$($range:tt)*]: ($size:expr) = $value:expr; $($rest:tt)*) => {
        __set_slice_internals!($($ln),* => $slice[$($range)*], $size, $value);
        set_slice!(@$($ln,)* 0 => $($rest)*);
    };

    (@$($ln:tt),* => $slice:ident[$($range:tt)*] = $option:ident $value:expr; $($rest:tt)*) => {
        __set_slice_internals!($($ln),* => $slice[$($range)*], $option $value);
        set_slice!(@$($ln,)* 0 => $($rest)*);
    };

    (@$($ln:tt),* => $slice:ident[$($range:tt)*] = $($value:expr),+; $($rest:tt)*) => {
        __set_slice_internals!($($ln),* => $slice[$($range)*], count!($( $value )+), [$($value),+]);
        set_slice!(@$($ln,)* 0 => $($rest)*);
    };

    (@$($ln:tt),* => $slice:ident: $($rest:tt)*) => {
        compile_error!("invalid size: size must be an expression surrouned by parentheses");
    };

    (@$($ln:tt),* => $slice:ident = ref $value:expr; $($rest:tt)*) => {
        compile_error!("option is missing: value should be of the form: \"{copy, clone} ref value\"")
    };

    (@$($ln:tt),* => $slice:ident = ; $($rest:tt)*) => {
        compile_error!("there must be a non-zero number of arguments in a list");
    };

    (@$($ln:tt),* => $slice:ident $($rest:tt)*) => {
        compile_error!("punctuation is missing!");
    };

    (@$($ln:tt),* => $slice:ident[$($range:tt)*]: $($rest:tt)*) => {
        compile_error!("invalid size: size must be an expression surrouned by parentheses");
    };

    (@$($ln:tt),* => $slice:ident[$($range:tt)*] = ; $($rest:tt)*) => {
        compile_error!("there must be a non-zero number of arguments in a list");
    };

    (@$($ln:tt),* => $slice:ident[$($range:tt)*] $($rest:tt)*) => {
        compile_error!("punctuation is missing!");
    };

    (@$($ln:tt),* => ) => {};
    () => {};
    
    (@$($ln:tt),* => [$($range:tt)*] $($rest:tt)*) => {
        compile_error!("missing identifier");
    };
    (@$($ln:tt),* => $($rest:tt)+) => {
        compile_error!("missing rvalue");
    };
    ($($rest:tt)+) => {
        set_slice!(@0 => $($rest)+);
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn test_move_values() {
        let mut v = [0; 6];
        let value = 0;
        let array = [2, 3]; 
        let vec = [4, 5, 6];

        set_slice! {
            v[0..1] = value;
            v[1..3]: (2) = array;
            v[3..]: (3) = vec;
        }
        // println!("{:?}", vec); // COMPILE ERROR: use after move

        assert!(&v == &[0, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn test_full_range() {
        let mut v = [0; 10];

        set_slice! {
            v[..] = 0, 1, 2, 3, 4, 5, 6, 7, 8, 9;
        }

        assert!(&v == &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);

        let mut v = [0; 10];

        set_slice! {
            v = 0, 1, 2, 3, 4, 5, 6, 7, 8, 9;
        }

        assert!(&v == &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }

    #[test]
    fn set_slice_test() {
        let mut v = [0; 8];
        let values = [4, 5, 6];
        let array = [0, 2];
        let deref = [7, 8];

        set_slice! {
            v[1..=2] = copy &[5, 3];
            v[3..6] = copy &values;
            v[..2] = copy &array;
            v[6..] = copy &deref;
        }
        let _ = values;

        assert!(&v == &[0, 2, 3, 4, 5, 6, 7, 8]);
    }

    #[test]
    fn doc_test() {
        let mut slice = [0; 3]; // example slice
        let vec = [-1, -2];


        set_slice! {
            slice = 1, 2, 3;
            slice[..2] = copy &vec;
        }

        assert_eq!(slice, [-1, -2, 3]);
    }
}