use std::io::{Write, Read, stdout, stdin};
use std::fs;
use std::env;
use std::process::ExitCode;

use quiz_app::Quiz;

fn main() -> ExitCode {
    let quiz_path = match env::args().skip(1).next() {
        Some(path) => path,
        None => {
            eprintln!("Please launch the quiz application with the path to the quiz as the first argument!");
            print!("Or, I guess you can enter the path to a quiz right now... ");
            _ = stdout().flush();

            let mut path = String::new();
            match stdin().read_line(&mut path) {
                Ok(_) => path.trim().to_owned(),
                Err(e) => {
                    eprintln!("Couldn't read your input: {e}");
                    return confirm_exit(ExitCode::FAILURE);
                }
            }
        },
    };

    println!("Taking Quiz: {quiz_path}");

    let quiz_str = match fs::read_to_string(quiz_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Could not read quiz file: {e}");
            return confirm_exit(ExitCode::FAILURE);
        }
    };

    let quiz: Quiz = match quiz_str.parse() {
        Ok(quiz) => quiz,
        Err(e) => {
            eprintln!("Could not parse quiz: {e}");
            return confirm_exit(ExitCode::FAILURE);
        }
    };

    let score = match quiz.take() {
        Ok(score) => score,
        Err(e) => {
            eprintln!("Could not take quiz: {e}");
            return confirm_exit(ExitCode::FAILURE);
        }
    };
    
    println!("\n\nQuiz finished!");
    println!("Your score: {score:.0}/{0:.0} ({1:.0}%)", quiz.total_score, score*100.0/quiz.total_score);
    
    return confirm_exit(ExitCode::SUCCESS);
}

fn confirm_exit(code: ExitCode) -> ExitCode {
    print!("Press enter to exit");
    _ = stdout().flush();

    // wait for ack, then exit
    _ = stdin().read(&mut[]);

    return code;
}
