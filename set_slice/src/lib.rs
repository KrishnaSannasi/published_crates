#[macro_export]
macro_rules! count {
    ()        => {0usize};
    ($one:tt) => {1usize};
    ($($pairs:tt $_p:tt)*) => {
        count!($($pairs)*) << 1usize
    };
    ($odd:tt $($rest:tt)*) => {
        count!($($rest)*) | 1usize
    };
}

/// any branch that has an '@' preceeding is internal to the macro
#[macro_export]
macro_rules! set_slice {
    (@$($ln:tt),* => &mut $vec:ident[$($range:tt)*]: [$type:ty; $size:expr] = $value:expr;) => {
        unsafe {
            let mut a = $value;
            let input: &mut [$type] = &mut a;
            let slice = &mut $vec[$($range)*];
            let (il, sl) = (input.len(), slice.len());

            if il != sl {
                panic!("ln({}) input length invalid: {}, expected: {}", count!($($ln)*), il, sl)
            }

            if sl != $size {
                panic!("ln({}) slice length invalid: {}, expected: {}", count!($($ln)*), sl, $size)
            }

            let input = input as *mut [$type] as *mut [$type; $size];
            let slice = slice as *mut [$type] as *mut [$type; $size];
            ::std::ptr::swap(slice, input);
        }
    };
    (@$($ln:tt),* => &mut $vec:ident[$($range:tt)*]: &[$type:ty] = ref $value:expr;) => {{
        let input: &[$type] = $value;
        let slice = &mut $vec[$($range)*];
        let (il, sl) = (input.len(), slice.len());

        if il != sl {
            panic!("ln({}) input length invalid: {}, expected: {}", count!($($ln)*), il, sl)
        }

        slice.copy_from_slice(input);
    }};
    
    // ln is line number, for better error messages
    // it is stored as a list of zeros, which is counted (O(log n)) when
    // the line number is needed
    (&mut $vec:ident[$($range:tt)*]: [$type:ty; $size:expr] = $value:expr; $($rest:tt)*) => {
        set_slice!(@0 => &mut $vec[$($range)*]: [$type; $size] = $value;);
        set_slice!(@0, 0 => $($rest)*);
    };
    (&mut $vec:ident[$($range:tt)*]: [$type:ty] = $value:expr; $($rest:tt)*) => {
        compile_error!("size of slice is missing!");
    };
    (&mut $vec:ident[$($range:tt)*]: [$type:ty] = $($value:expr),*; $($rest:tt)*) => {
        set_slice!(@0 => &mut $vec[$($range)*]: [$type; count!($( $value )*)] = [$($value),*];);
        set_slice!(@0, 0 => $($rest)*);
    };
    (&mut $vec:ident[$($range:tt)*]: [$type:ty; $size:expr] = $($value:expr),*; $($rest:tt)*) => {
        compile_error!("size of slice is not needed for lists of values.");
    };
    (&mut $vec:ident[$($range:tt)*]: &[$type:ty] = ref $value:expr; $($rest:tt)*) => {
        set_slice!(@0 => &mut $vec[$($range)*]: &[$type] = ref $value;);
        set_slice!(@0, 0 => $($rest)*);
    };

    // ln is line number, for better error messages
    // it is stored as a list of zeros, which is counted (O(log n)) when
    // the line number is needed
    (@$($ln:tt),* => &mut $vec:ident[$($range:tt)*]: [$type:ty; $size:expr] = $value:expr; $($rest:tt)*) => {
        set_slice!(@$($ln),* => &mut $vec[$($range)*]: [$type; $size] = $value;);
        set_slice!(@$($ln,)* 0 => $($rest)*);
    };
    (@$($ln:tt),* => &mut $vec:ident[$($range:tt)*]: [$type:ty] = $($value:expr),*; $($rest:tt)*) => {
        set_slice!(@$($ln),* => &mut $vec[$($range)*]: [$type; count!($( $value )*)] = [$($value),*];);
        set_slice!(@$($ln,)* 0 => $($rest)*);
    };
    (@$($ln:tt),* => &mut $vec:ident[$($range:tt)*]: &[$type:ty] = ref $value:expr; $($rest:tt)*) => {
        set_slice!(@$($ln),* => &mut $vec[$($range)*]: &[$type] = ref $value;);
        set_slice!(@$($ln,)* 0 => $($rest)*);
    };
    (@$($ln:tt),* => ) => {};
    () => {};
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn test_move_values() {
        let mut v = vec![0; 6];
        let values = vec![4, 5, 6];

        set_slice! {
            &mut v[1..3]: [i32; 2] = [2, 3];
            &mut v[3..]: [i32; 3] = values;
        }
        // println!("{:?}", values); // COMPILE ERROR: use after move

        assert!(&v == &[0, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn test_full_range() {
        let mut v = vec![0; 10];

        set_slice! {
            &mut v[..]: [i32] = 0, 1, 2, 3, 4, 5, 6, 7, 8, 9;
        }

        assert!(&v == &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }

    #[test]
    fn set_slice_test() {
        let mut v = vec![0; 8];
        let values = vec![4, 5, 6];
        let array = [0, 2];
        let deref = vec![7, 8];

        set_slice! {
            &mut v[1..=2]: &[i32] = ref &[5, 3];
            &mut v[3..6]: &[i32] = ref &values;
            &mut v[..2]: &[i32] = ref &array;
            &mut v[6..]: &[i32] = ref &deref;
        }
        let _ = values;

        assert!(&v == &[0, 2, 3, 4, 5, 6, 7, 8]);
    }
}