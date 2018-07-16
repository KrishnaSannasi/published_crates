/// any branch that has an '@' preceeding is internal to the macro
#[macro_export]
macro_rules! set_slice {
    (@count )        => {0usize};
    (@count $one:tt) => {1usize};
    (@count $($pairs:tt $_p:tt)*) => {
        set_slice!(@count $($pairs)*) << 1usize
    };
    (@count $odd:tt $($rest:tt)*) => {
        set_slice!(@count $($rest)*) | 1usize
    };

    (@@copy $slice:expr, $value:expr) => {
        $slice.copy_from_slice($value);
    };
    (@@clone $slice:expr, $value:expr) => {
        $slice.clone_from_slice($value);
    };
    (@@$option:ident $slice:expr, $value:expr) => {
        compile_error!(stringify!(invalid option $option, valid options are copy, clone))
    };
    (@@$($ln:tt),* => $slice:expr, $size:expr, $value:expr) => {{
        const LINE: usize = set_slice!(@count $($ln)*);

        // function used for type inference
        #[inline(always)]
        fn set<T>(slice: &mut [T], value: &mut [T]) {
            let (sl, vl) = (slice.len(), value.len());

            if sl != $size { // validate slice size
                panic!("line {}: slice length ({}) is invalid, excepted: {}", LINE, sl, $size)
            }

            if vl != $size { // validate value size
                panic!("line {}: value length ({}) is invalid, excepted: {}", LINE, vl, $size)
            }
            
            let value = value as *mut [T] as *mut [T; $size];
            let slice = slice as *mut [T] as *mut [T; $size];
            
            unsafe { ::std::ptr::swap(slice, value); }
        }

        let mut val = $value; // capture value
        set(&mut $slice, &mut val);
    }};
    (@@$($ln:tt),* => $slice:expr, $option:ident $value:expr) => {{
        let input = $value;
        let slice = &mut $slice;
        let (il, sl) = (input.len(), slice.len());

        if il != sl {
            panic!("ln({}) input length invalid: {}, expected: {}", set_slice!(@count $($ln)*), il, sl)
        }

        set_slice!(@@$option slice, input);
    }};

    // ln is line number, for better error messages
    // it is stored as a list of zeros, which is counted (O(log n)) when
    // the line number is needed

    // this pattern is for values that will be moved into the slice
    (@$($ln:tt),* => $slice:ident: ($size:expr) = $value:expr; $($rest:tt)*) => {
        set_slice!(@@$($ln),* => $slice, $size, $value);
        set_slice!(@$($ln,)* 0 => $($rest)*);
    };
    // this pattern is for values that will be moved into the slice
    (@$($ln:tt),* => $slice:ident[$($range:tt)*]: ($size:expr) = $value:expr; $($rest:tt)*) => {
        set_slice!(@@$($ln),* => $slice[$($range)*], $size, $value);
        set_slice!(@$($ln,)* 0 => $($rest)*);
    };

    (@$($ln:tt),* => $slice:ident[$($range:tt)*]: $($rest:tt)*) => {
        compile_error!("invalid size: size must be an expression surrouned by parentheses");
    };

    // this pattern if for values that will be copied/cloned into the slice
    (@$($ln:tt),* => $slice:ident = $option:ident $value:expr; $($rest:tt)*) => {
        set_slice!(@@$($ln),* => $slice, $option $value);
        set_slice!(@$($ln,)* 0 => $($rest)*);
    };

    // this pattern if for values that will be a list of expressions
    (@$($ln:tt),* => $slice:ident = $($value:expr),+; $($rest:tt)*) => {
        set_slice!(@@$($ln),* => $slice, set_slice!(@count $( $value )+), [$($value),+]);
        set_slice!(@$($ln,)* 0 => $($rest)*);
    };

    // this pattern if for values that will be copied/cloned into the slice
    (@$($ln:tt),* => $slice:ident[$($range:tt)*] = $option:ident $value:expr; $($rest:tt)*) => {
        set_slice!(@@$($ln),* => $slice[$($range)*], $option $value);
        set_slice!(@$($ln,)* 0 => $($rest)*);
    };

    // this pattern if for values that will be a list of expressions
    (@$($ln:tt),* => $slice:ident[$($range:tt)*] = $($value:expr),+; $($rest:tt)*) => {
        set_slice!(@@$($ln),* => $slice[$($range)*], set_slice!(@count $( $value )+), [$($value),+]);
        set_slice!(@$($ln,)* 0 => $($rest)*);
    };

    // this pattern if for values that will be copied/cloned into the slice
    (@$($ln:tt),* => $slice:ident[$($range:tt)*] = ref $value:expr; $($rest:tt)*) => {
        compile_error!("option is missing: value should be of the form: \"{copy, clone} ref value\"")
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
        let mut v = vec![0; 6];
        let value = 0;
        let array = [2, 3]; 
        let vec = vec![4, 5, 6];

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
        let mut v = vec![0; 10];

        set_slice! {
            v[..] = 0, 1, 2, 3, 4, 5, 6, 7, 8, 9;
        }

        assert!(&v == &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);

        let mut v = vec![0; 10];

        set_slice! {
            v = 0, 1, 2, 3, 4, 5, 6, 7, 8, 9;
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
            v[1..=2] = copy &[5, 3];
            v[3..6] = copy &values;
            v[..2] = copy &array;
            v[6..] = copy &deref;
        }
        let _ = values;

        assert!(&v == &[0, 2, 3, 4, 5, 6, 7, 8]);
    }
}