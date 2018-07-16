# set_vec

A Rust macro for easily assigning to slices

## rules for using set_vec

1. you can only use slices, or anything that implements Deref<Target = [T]> to set to slices
2. range checks are all done at run-time
    1. the input slice must be the same size as the slice you assign to
    2. if you are selected a part of the slice to assign to then the input slice must match the size of the selected part
3. the types must match
    1. ***note: set_vec uses a generic function internally to figure out type information
4. for move values, the size of the slice must be known at compile time, as a constexpr
5. for refereces, the internal types must be Clone or Copy to work

## set_vec by example

```Rust
// you can set the entire contents of the slice to whatever you want
let slice = &mut [0; 3] as &mut [i32]; // this is to simulate having only a slice without knowning its size

set_slice! {
    slice = 1, 2, 3; // this list is internally counted and converted to an array at compile-time
}
assert_eq!(slice, &[1, 2, 3]);

// ... or you can only set parts of the slice 
let slice = &mut [0; 5] as &mut [i32];

set_slice! {
    slice[..3] = 1, 2, 3;
}
assert_eq!(slice, &[1, 2, 3, 0, 0]);
```

```Rust
// ... you can also do multiple assigns in one macro call
let mut slice = [0; 5];
let slice = &mut slice as &mut [i32];

set_slice! {
    slice[..2] = 1, 2;
    slice[3..] = 4, 5;
}
assert_eq!(slice, &[1, 2, 0, 4, 5]);
```

You can use expressions to set to the slices, either as values to be moved in, or as references
if they are move values you must specify a const expression size in parentheses
```Rust
let slice = &mut [0; 5] as &mut [i32];
let array = [1, 2];
let vec = vec![3, 4];

set_slice! {
    slice[..2]: (2) = array; // array is moved into set_slice
    slice[3..]: (2) = vec; // vec is moved into set_slice
}
println!("array = {:?}", array); // fine, array is a copy type
// println!("vec = {:?}", vec); // compile time error, vec is moved into the set_slice and dropped
assert_eq!(slice, &[1, 2, 0, 3, 4]);
```

but you don't have to move into set_slice if you get a reference
with references you must specify if the contents should be copied or cloned
but they must derive Copy or Clone respectively
```Rust
let slice = &mut [0; 5] as &mut [i32];
let array = [1, 2];
let vec = vec![3, 4];

set_slice! {
    slice[..2] = copy &array; // array is NOT moved into set_slice, and contents are copied
    slice[3..] = copy &vec; // vec is NOT moved into set_slice, and contents are copied
}
println!("array = {:?}", array); // this is fine, array was borrowed
println!("vec = {:?}", vec); // this is fine, vec was borrowed
assert_eq!(slice, &[1, 2, 0, 3, 4]);

#[derive(Clone, Debug, PartialEq)]
enum A { Zero, One };
let mut slice: [A; 5] = [A::Zero, A::Zero, A::Zero, A::Zero, A::Zero];
let slice = &mut slice as &mut [A];
let array = [A::One, A::One];
let vec = vec![A::One; 2];

set_slice! {
    slice[..2] = clone &array; // array is NOT moved into set_slice, and contents are cloned
    slice[3..] = clone &vec; // vec is NOT moved into set_slice, and contents are cloned
    // slice[3..] = copy &vec; // this won't work because 'A' is not a copy type
}
println!("array = {:?}", array); // this is fine, array was borrowed
println!("vec = {:?}", vec); // this is fine, vec was borrowed
assert_eq!(slice, &[A::One, A::One, A::Zero, A::One, A::One]);
```

## all possible valid use cases

### with lists, and ranges (these ranges can be mixed and matched with the other sub-sections)
```Rust
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
    slice[index..2] = 2;
    slice[1..end] = 2;
    slice[index..end] = 2;
    slice[index..] = 2, 3;
    slice[..end] = 1, 2;
}
```

### with move types
```Rust
let slice = &mut [0; 3] as &mut [i32];
let vec_move = vec![1, 2, 3];

set_slice! {
    slice = vec_move;
}
let vec_move = vec![1, 2, 3];

set_slice! {
    slice[..] = vec_move;
}
let vec_move = vec![1, 2];

set_slice! {
    slice[..2] = vec_move;
}
```

### with references
```Rust
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
```