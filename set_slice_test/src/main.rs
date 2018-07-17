#[macro_use]
extern crate set_slice;

fn main() {
    let mut slice = vec![0; 4];
    let array = [-1, -2];

    set_slice! {
        slice[2..] = 1, 2;
        slice[..2] = copy &array;
    }

    println!("{:?}", slice);
}
