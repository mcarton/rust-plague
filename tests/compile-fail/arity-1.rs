#![feature(plugin)]
#![plugin(plague)]

plague! {
    for [(1,)] //~ERROR: this function takes 2 parameters but 1 parameter was supplied
    test fn foo(a: i32, b: i32) {
        assert_eq!(a, b)
    }
}

fn main() {}
