#![feature(plugin)]
#![plugin(plague)]

plague! {
    for [(1,2)]
    test fn foo(a: i32, b: i32) {
        assert_eq!(a, b)
    } that should not be here //~ERROR: expected end of macro
}

fn main() {}
