#![feature(plugin)]
#![plugin(plague)]

#![deny(dead_code)]

plague! {
    for []
    test fn empty() { //~ERROR: function is never used: `empty`
    }
}

fn main() {}
