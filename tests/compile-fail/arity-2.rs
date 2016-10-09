#![feature(plugin)]
#![plugin(plague)]

plague! {
    for [
        (1, 2), //~ERROR: this function takes 0 parameters but 2 parameters were supplied
    ]
    test fn foo() {
    }
}

fn main() {}
