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
        (1, 1),
        (2, 2),
    ]
    test fn fooes(a: i32, b: i32) {
        assert_eq!(a, b);
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

plague! {
    for [
        (1, 2),
        (2, 3),
    ]
    test! fn bars(a: i32, b: i32) {
        assert_eq!(a, b);
    }
}
