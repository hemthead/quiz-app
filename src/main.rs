use std::fs;

fn main() {
    let quiz: quiz_app::Quiz = std::fs::read_to_string("working.qz").unwrap().parse().unwrap();
}
