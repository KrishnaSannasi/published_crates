# set_slice

A Rust macro for easily assigning to slices

[![Latest version](https://img.shields.io/crates/v/set_slice.svg)](https://crates.io/crates/set_slice)
[![Documentation](https://docs.rs/set_slice/badge.svg)](https://docs.rs/set_slice)
[![License](https://img.shields.io/crates/l/set_slice.svg)](https://github.com/KrishnaSannasi/published_crates/blob/master/set_slice/LICENSE.md)

## rules for using set_slice

1. you can only use slices, or anything coerces to a slice
2. lvalues must either be identifiers or indexes
    1. identifier: array, b, vector, something_else
    2. indexes: array[1..], b[..], vector[1..4], something_else[12..=14]
3. range checks are all done at run-time
    1. the input slice must be the same size as the slice you assign to
4. the types must match
    1. **note:** set_slice uses a generic function internally to figure out type information
5. for unsafe copy values, the size of the slice must be known at compile time, as a constexpr
6. for safe references assignments, the internal types must be Clone or Copy to work

## set_slice by example

you can set the entire contents of the slice to whatever you want
```rust
let slice = &mut [0; 3] as &mut [i32]; // this is to simulate having only a slice without knowning its size

set_slice! {
    slice = 1, 2, 3; // this list is internally counted and converted to an array at compile-time
}
assert_eq!(slice, [1, 2, 3]);

// ... or you can only set parts of the slice 
let slice = &mut [0; 5] as &mut [i32];

set_slice! {
    slice[..3] = 1, 2, 3;
}
assert_eq!(slice, [1, 2, 3, 0, 0]);
```

you can also do multiple assigns in one macro call
each instruction is executed in order, and can overwrite previous instructions
```rust
let slice = &mut [0; 5] as &mut [i32];

set_slice! {
    slice[..2] = 1, 2;
    slice[3..] = 4, 5;
}
assert_eq!(slice, [1, 2, 0, 4, 5]);
```

You can use expressions to set to the slices, either as values to be moved in, or as references
if they are move values you must specify a const expression size in parentheses
```rust
let slice = &mut [0; 5] as &mut [i32];
let array = [1, 2];
let vec = vec![3, 4];

set_slice! {
    slice[..2] = move array;
    slice[3..] = move vec;       // vec is moved into set_slice
}
println!("array = {:?}", array); // fine, array is a copy type
// println!("vec = {:?}", vec);  // compile time error, vec is moved into the set_slice and dropped
assert_eq!(slice, [1, 2, 0, 3, 4]);
```

but you don't have to move into set_slice if you get a reference
with references you must specify if the contents should be copied or cloned
but they must derive Copy or Clone respectively
```rust
let slice = &mut [0; 5] as &mut [i32];
let array = [1, 2];
let vec = vec![3, 4];

set_slice! {
    slice[..2] = copy &array; // array is NOT moved into set_slice, and contents are copied
    slice[3..] = copy &vec;   // vec is NOT moved into set_slice, and contents are copied
}
println!("array = {:?}", array); // this is fine, array was borrowed
println!("vec = {:?}", vec); // this is fine, vec was borrowed
assert_eq!(slice, [1, 2, 0, 3, 4]);
```

```rust
#[derive(Clone, Debug, PartialEq)]
enum A { Zero, One };
let slice: [A; 5] = [A::Zero, A::Zero, A::Zero, A::Zero, A::Zero];
let slice = &mut slice as &mut [A];
let array = [A::One, A::One];
let vec = vec![A::One; 2];

set_slice! {
    slice[..2] = clone &array;   // array is NOT moved into set_slice, and contents are cloned
    slice[3..] = clone &vec;     // vec is NOT moved into set_slice, and contents are cloned
    // slice[3..] = copy &vec;   // this won't work because 'A' is not a copy type
}
println!("array = {:?}", array); // this is fine, array was borrowed
println!("vec = {:?}", vec);     // this is fine, vec was borrowed
assert_eq!(slice, [A::One, A::One, A::Zero, A::One, A::One]);
```

## valid use cases

### with lists, and ranges
these ranges can be mixed and matched with the other sub-sections
```rust
let slice = &mut [0; 3] as &mut [i32];
let init = 1;
let end = 2;

set_slice! {
    slice = 1, 2, 3;
    slice[..] = 1, 2, 3;
    slice[0..] = 1, 2, 3;
    slice[..3] = 1, 2, 3;
    slice[0..3] = 1, 2, 3;
    slice[0..2] = 1, 2;
    slice[1..2] = 2;
    slice[init..2] = 2;
    slice[1..end] = 2;
    slice[init..end] = 2;
    slice[init..] = 2, 3;
    slice[..end] = 1, 2;
}

println!("{:?}", slice);
panic!();
```

### with move types
```rust
let slice = &mut [0; 3] as &mut [i32];
let vec_move = vec![1, 2, 3];

set_slice! {
    slice = move vec_move;
}
let vec_move = vec![1, 2, 3];

set_slice! {
    slice[..] = move vec_move;
}
let vec_move = vec![1, 2];

set_slice! {
    slice[..2] = move vec_move;
}
```

### with references
```rust
let slice = &mut [0; 3] as &mut [i32];
let array = [1, 2, 3];
let vec = vec![1, 2, 3];

// only works if slice implements copy
set_slice! {
    slice = copy &vec;
    slice = copy &array;
    slice[..2] = copy &vec[1..];
    slice[..2] = copy &array[1..];
}

// only works if slice implements clone
set_slice! {
    slice = clone &vec;
    slice = clone &array;
    slice[..2] = clone &vec[1..];
    slice[..2] = clone &array[1..];
}

// works with any type, but is incredibly unsafe
set_slice! {
    unsafe slice: (3) = ref &vec;
    unsafe slice: (3) = ref &array;
    unsafe slice[..2]: (2) = ref &vec[1..];
    unsafe slice[..2]: (2) = ref &array[1..];
}
```

## known undefined behaviour or unintended

all of these are when using unsafe slice assignment

* an array of Boxed values share the inner values after assignment
* an array of Mutexes may become corrupted
    * an array of Arc<Mutex<T>> is fine

