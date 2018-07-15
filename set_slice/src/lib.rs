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
    (@$($ln:tt),* => &mut $to_slice:ident[$($range:tt)*]: ($size:expr) = $value:expr;) => {
        unsafe {
            // capture value
            let mut a = $value;

            // get mutable refernce to be turned into pointer
            let input = &mut a;

            // get slice 
            let slice = &mut $to_slice[$($range)*];

            // input validation to make sure it is a safe operation
            let (il, sl) = (input.len(), slice.len());

            if il != sl {
                panic!("ln({}) input length invalid: {}, expected: {}", count!($($ln)*), il, sl)
            }

            if sl != $size {
                panic!("ln({}) slice length invalid: {}, expected: {}", count!($($ln)*), sl, $size)
            }

            /// this function is used for type inference
            #[inline(always)]
            unsafe fn do_swap<T>(slice: &mut [T], input: &mut [T]) {
                let input = input as *mut [T] as *mut [T; $size];
                let slice = slice as *mut [T] as *mut [T; $size];
                ::std::ptr::swap(slice, input);
            }

            // swap pointers of input and slice
            do_swap(slice, input);
        }
    };
    (@$($ln:tt),* => &mut $to_slice:ident[$($range:tt)*] = ref $value:expr;) => {{
        let input = $value;
        let slice = &mut $to_slice[$($range)*];
        let (il, sl) = (input.len(), slice.len());

        if il != sl {
            panic!("ln({}) input length invalid: {}, expected: {}", count!($($ln)*), il, sl)
        }

        slice.copy_from_slice(input);
    }};
    
    // ln is line number, for better error messages
    // it is stored as a list of zeros, which is counted (O(log n)) when
    // the line number is needed
    (&mut $to_slice:ident[$($range:tt)*]: ($size:expr) = $value:expr; $($rest:tt)*) => {
        set_slice!(@0 => &mut $to_slice[$($range)*]: ($size) = $value;);
        set_slice!(@0, 0 => $($rest)*);
    };
    (&mut $to_slice:ident[$($range:tt)*] = $($value:expr),*; $($rest:tt)*) => {
        set_slice!(@0 => &mut $to_slice[$($range)*]: (count!($( $value )*)) = [$($value),*];);
        set_slice!(@0, 0 => $($rest)*);
    };
    (&mut $to_slice:ident[$($range:tt)*] = ref $value:expr; $($rest:tt)*) => {
        set_slice!(@0 => &mut $to_slice[$($range)*] = ref $value;);
        set_slice!(@0, 0 => $($rest)*);
    };
    
    // full range sugar
    (&mut $to_slice:ident $($rest:tt)*) => {
        set_slice!(&mut $to_slice[..] $($rest)*);
    };

    // ln is line number, for better error messages
    // it is stored as a list of zeros, which is counted (O(log n)) when
    // the line number is needed
    (@$($ln:tt),* => &mut $to_slice:ident[$($range:tt)*]: ($size:expr) = $value:expr; $($rest:tt)*) => {
        set_slice!(@$($ln),* => &mut $to_slice[$($range)*]: ($size) = $value;);
        set_slice!(@$($ln,)* 0 => $($rest)*);
    };
    (@$($ln:tt),* => &mut $to_slice:ident = $($value:expr),*; $($rest:tt)*) => {
        set_slice!(@$($ln),* => &mut $to_slice[..]: (count!($( $value )*)) = [$($value),*];);
        set_slice!(@$($ln,)* 0 => $($rest)*);
    };
    (@$($ln:tt),* => &mut $to_slice:ident[$($range:tt)*] = ref $value:expr; $($rest:tt)*) => {
        set_slice!(@$($ln),* => &mut $to_slice[$($range)*] = ref $value;);
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
            &mut v[0..1] = 0;
            &mut v[1..3]: (2) = [2, 3];
            &mut v[3..]: (3) = values;
        }
        // println!("{:?}", values); // COMPILE ERROR: use after move

        assert!(&v == &[0, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn test_full_range() {
        let mut v = vec![0; 10];

        set_slice! {
            &mut v[..] = 0, 1, 2, 3, 4, 5, 6, 7, 8, 9;
        }

        assert!(&v == &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);

        let mut v = vec![0; 10];

        set_slice! {
            &mut v = 0, 1, 2, 3, 4, 5, 6, 7, 8, 9;
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
            &mut v[1..=2] = ref &[5, 3];
            &mut v[3..6] = ref &values;
            &mut v[..2] = ref &array;
            &mut v[6..] = ref &deref;
        }
        let _ = values;

        assert!(&v == &[0, 2, 3, 4, 5, 6, 7, 8]);
    }
}