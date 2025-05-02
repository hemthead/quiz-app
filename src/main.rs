use std::fs;
use std::io::stdin;

use quiz_app::Quiz;

fn main() {
    let quiz: Quiz = fs::read_to_string("examples/working.qz").unwrap().parse().unwrap();
    let _ = quiz.take().unwrap();
}
