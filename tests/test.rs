#![feature(plugin)]
#![plugin(plague)]

plague! {
    for [
        1,
        2,
    ]
    test fn foo(a: i32) {
        println!("{}", a);
    }
}

plague! {
    for [
        1,
        2,
    ]
    test! fn bar(a: i32) {
        panic!("{}", a);
    }
}
