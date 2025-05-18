use std::fs;
use std::env;
use std::process::ExitCode;

use quiz_app::Quiz;

fn main() -> ExitCode {
    let quiz_path = match env::args().skip(1).next() {
        Some(path) => path,
        None => {
            println!("Please launch the quiz application with the path to the quiz as the first argument!");
            return ExitCode::FAILURE;
        },
    };

    let quiz_str = match fs::read_to_string(quiz_path) {
        Ok(s) => s,
        Err(e) => {
            println!("Could not read quiz file: {e}");
            return ExitCode::FAILURE;
        }
    };

    let quiz: Quiz = match quiz_str.parse() {
        Ok(quiz) => quiz,
        Err(e) => {
            println!("Could not parse quiz: {e:?}");
            return ExitCode::FAILURE;
        }
    };

    let _ = match quiz.take() {
        Ok(score) => score,
        Err(e) => {
            println!("Could not take quiz: {e:?}");
            return ExitCode::FAILURE;
        }
    };

    ExitCode::SUCCESS
}
