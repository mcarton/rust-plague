#![feature(plugin)]
#![plugin(plague)]

plague! {
    for [ //~ERROR: this function takes 0 parameters but 1 parameter was supplied [E0061]
        (1, 2),
    ]
    test fn foo() {
    }
}

fn main() {}
