#![feature(plugin)]
#![plugin(plague)]

plague! {
    for [ //~ERROR: this function takes 0 parameters but 2 parameters were supplied
        (1, 2),
    ]
    test fn foo() {
    }
}

fn main() {}
