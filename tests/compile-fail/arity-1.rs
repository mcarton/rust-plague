#![feature(plugin)]
#![plugin(plague)]

plague! {
    for [1] //~ERROR: expected tuple, the test function has several arguments
    test fn foo(a: i32, b: i32) {
        assert_eq!(a, b)
    }
}

fn main() {}
