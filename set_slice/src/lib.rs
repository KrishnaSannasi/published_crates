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
    SLICE = VALUE_1, VALUE_2, VALUE_3, ...;          // list
    SLICE = move VALUE;                              // move
    SLICE = clone REFERENCE;                         // clone ref
    SLICE = copy REFERENCE;                          // copy ref
    unsafe SLICE: (SIZE) = ref REFERENCE;            // unsafe copy ref
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

## list
the list: `VALUE_1`, `VALUE_2`, `VALUE_3`, ... is counted and converted into an array \
after conversion it is has the same semantics as move applied to the new array

## move
the `VALUE` is moved into set_slice and dropped \
the contents of `VALUE` are stored into the slice

## copy
the `REFERENCE` `&[T]` values are copied into the slice \
`T` must implement `Copy`

## clone
the `REFERENCE` `&[T]` values are cloned into the slice \
`T` must implement `Clone`

## copy
the `REFERENCE` `&[T]` values are copied into the slice \
`T` must implement `Copy`

## unsafe copy
**VERY UNSAFE** \
the `REFERENCE` `&[T]` values are copied into the slice \
internally this uses ::core::mem::transmute_copy \
so, use this with caution, as it may cause undefined behaviour \
**VERY UNSAFE**

# Cargo features
This crate allows for use in no-std environment.
*/

#[doc(hidden)]
pub use core::ptr::swap as __swap_ptr;
#[doc(hidden)]
pub use core::mem::transmute_copy as __transmute_copy_mem;

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

    ($($ln:tt),* => move $slice:expr, $value:expr) => {{
        const LINE: usize = count!($($ln)*);

        #[inline(always)]
        fn set<T>(slice: &mut [T], value: &mut [T]) {
            let (sl, vl) = (slice.len(), value.len());

            assert_eq!(sl, vl, "line {}: value length ({}) is invalid, excepted: {}", LINE, vl, sl);
            slice.swap_with_slice(value);
        }

        let mut val = $value; // capture value
        set(&mut $slice, &mut val);
    }};
    ($($ln:tt),* => $slice:expr, $option:ident $value:expr) => {{
        const LINE: usize = count!($($ln)*);
        let input: &_ = $value;
        let slice = &mut $slice;
        let (il, sl) = (input.len(), slice.len());

        assert_eq!(il, sl, "ln({}) input length invalid: {}, expected: {}", LINE, il, sl);

        __set_slice_internals!($option slice, input);
    }};
    ($($ln:tt),* => ref $slice:expr, $size:expr, $value:expr) => {{
        const LINE: usize = count!($($ln)*);
        
        #[inline(always)]
        fn set<T>(slice: &mut [T], value: &[T]) {
            let (sl, vl) = (slice.len(), value.len());

            assert_eq!(sl, $size, "line {}: slice length ({}) is invalid, excepted: {}", LINE, sl, $size);
            assert_eq!(vl, $size, "line {}: value length ({}) is invalid, excepted: {}", LINE, vl, $size);
            
            unsafe {
                let slice = &mut *(slice as *mut [T] as *mut [T; $size]);
                let value = &*(value as *const [T] as *const [T; $size]);

                *slice = $crate::__transmute_copy_mem(value);
            }
        }

        let input: &_ = $value;
        let slice = &mut $slice;

        set(slice, input);
    }};
}

/// a macro for setting parts of slices, see crate level docs for more info 
#[macro_export]
macro_rules! set_slice {
    // no range branches
    (@$($ln:tt),* => unsafe $slice:ident: ($size:expr) = ref $value:expr; $($rest:tt)*) => {
        __set_slice_internals!($($ln),* => ref $slice, $size, $value);
        set_slice!(@$($ln,)* 0 => $($rest)*);
    };

    (@$($ln:tt),* => $slice:ident = move $value:expr; $($rest:tt)*) => {
        __set_slice_internals!($($ln),* => move $slice, $value);
        set_slice!(@$($ln,)* 0 => $($rest)*);
    };

    (@$($ln:tt),* => $slice:ident = $option:ident $value:expr; $($rest:tt)*) => {
        __set_slice_internals!($($ln),* => $slice, $option $value);
        set_slice!(@$($ln,)* 0 => $($rest)*);
    };

    (@$($ln:tt),* => $slice:ident = $($value:expr),+; $($rest:tt)*) => {
        __set_slice_internals!($($ln),* => move $slice, [$($value),+]);
        set_slice!(@$($ln,)* 0 => $($rest)*);
    };

    // with range branches
    (@$($ln:tt),* => unsafe $slice:ident[$($range:tt)*]: ($size:expr) = ref $value:expr; $($rest:tt)*) => {
        __set_slice_internals!($($ln),* => ref $slice[$($range)*], $size, $value);
        set_slice!(@$($ln,)* 0 => $($rest)*);
    };

    (@$($ln:tt),* => $slice:ident[$($range:tt)*] = move $value:expr; $($rest:tt)*) => {
        __set_slice_internals!($($ln),* => move $slice[$($range)*], $value);
        set_slice!(@$($ln,)* 0 => $($rest)*);
    };

    (@$($ln:tt),* => $slice:ident[$($range:tt)*] = $option:ident $value:expr; $($rest:tt)*) => {
        __set_slice_internals!($($ln),* => $slice[$($range)*], $option $value);
        set_slice!(@$($ln,)* 0 => $($rest)*);
    };

    (@$($ln:tt),* => $slice:ident[$($range:tt)*] = $($value:expr),+; $($rest:tt)*) => {
        __set_slice_internals!($($ln),* => move $slice[$($range)*], [$($value),+]);
        set_slice!(@$($ln,)* 0 => $($rest)*);
    };

    // errors and terminals
    (@$($ln:tt),* => unsafe $slice:ident[$($range:tt)*]: ($size:expr) = $value:expr; $($rest:tt)*) => {
        compile_error!("Moving values into the slice is safe");
    };
    (@$($ln:tt),* => unsafe $slice:ident: ($size:expr) = $value:expr; $($rest:tt)*) => {
        compile_error!("Moving values into the slice is safe");
    };
    
    (@$($ln:tt),* => $slice:ident[$($range:tt)*]: ($size:expr) = ref $value:expr; $($rest:tt)*) => {
        compile_error!("Copying arbitrary references in unsafe");
    };
    (@$($ln:tt),* => $slice:ident: ($size:expr) = ref $value:expr; $($rest:tt)*) => {
        compile_error!("Copying arbitrary references in unsafe");
    };
    (@$($ln:tt),* => $slice:ident[$($range:tt)*] = ref $value:expr; $($rest:tt)*) => {
        compile_error!("Copying arbitrary references in unsafe");
    };
    (@$($ln:tt),* => $slice:ident= ref $value:expr; $($rest:tt)*) => {
        compile_error!("Copying arbitrary references in unsafe");
    };
    
    (@$($ln:tt),* => unsafe $slice:ident[$($range:tt)*] = ref $value:expr; $($rest:tt)*) => {
        compile_error!("Unkown size: size must be an expression surrouned by parentheses");
    };
    (@$($ln:tt),* => unsafe $slice:ident= ref $value:expr; $($rest:tt)*) => {
        compile_error!("Unkown size: size must be an expression surrouned by parentheses");
    };

    (@$($ln:tt),* => $slice:ident: $($rest:tt)*) => {
        compile_error!("Invalid size: size must be an expression surrouned by parentheses");
    };

    (@$($ln:tt),* => $slice:ident = ref $value:expr; $($rest:tt)*) => {
        compile_error!("Option is missing: value should be of the form: \"{copy, clone} ref value\"")
    };

    (@$($ln:tt),* => $slice:ident = ; $($rest:tt)*) => {
        compile_error!("There must be a non-zero number of arguments in a list");
    };

    (@$($ln:tt),* => $slice:ident $($rest:tt)*) => {
        compile_error!("Punctuation is missing!");
    };

    (@$($ln:tt),* => $slice:ident[$($range:tt)*]: $($rest:tt)*) => {
        compile_error!("Invalid size: size must be an expression surrouned by parentheses!");
    };

    (@$($ln:tt),* => $slice:ident[$($range:tt)*] = ; $($rest:tt)*) => {
        compile_error!("There must be a non-zero number of arguments in a list!");
    };

    (@$($ln:tt),* => $slice:ident[$($range:tt)*] $($rest:tt)*) => {
        compile_error!("Punctuation is missing!");
    };

    (@$($ln:tt),* => ) => {};
    () => {};
    
    (@$($ln:tt),* => [$($range:tt)*] $($rest:tt)*) => {
        compile_error!("Missing identifier, there is a range, but no slice");
    };
    (@$($ln:tt),* => $($rest:tt)+) => {
        compile_error!("Missing rvalue, there seems to be a missing slice to assign to");
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
            v[1..3] = move array;
            v[3..] = move vec;
        }
        // println!("{:?}", vec); // COMPILE ERROR: use after move

        assert_eq!(v, [0, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn test_full_range() {
        let mut v = [0; 10];

        set_slice! {
            v[..] = 0, 1, 2, 3, 4, 5, 6, 7, 8, 9;
        }

        assert_eq!(v, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);

        let mut v = [0; 10];

        set_slice! {
            v = 0, 1, 2, 3, 4, 5, 6, 7, 8, 9;
        }

        assert_eq!(v, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
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

        assert_eq!(v, [0, 2, 3, 4, 5, 6, 7, 8]);
    }

    #[test]
    fn set_slice_test_unsafe() {
        #[derive(PartialEq, Debug)]
        struct A(i32);
        let mut v = [A(0), A(0), A(0), A(0), A(0), A(0), A(0), A(0)];
        let values = [A(4), A(5), A(6)];
        let array = [A(0), A(2)];
        let deref = [A(7), A(8)];

        set_slice! {
            unsafe v[1..=2]: (2) = ref &[A(5), A(3)];
            unsafe v[3..6]: (3) = ref &values;
            unsafe v[..2]: (2) = ref &array;
            unsafe v[6..]: (2) = ref &deref;
        }
        let _ = values;

        assert_eq!(v, [A(0), A(2), A(3), A(4), A(5), A(6), A(7), A(8)]);
    }
}