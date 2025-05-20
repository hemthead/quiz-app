use std::io::{self, Write, stdin, stdout};
use std::hash::{BuildHasher, Hasher, RandomState};

#[derive(Debug)]
pub enum ConfigValueParseError {
    ParseIntError(std::num::ParseIntError),
    ParseFloatError(std::num::ParseFloatError),
    ParseBoolError(std::str::ParseBoolError),
}
impl From<std::num::ParseIntError> for ConfigValueParseError {
    fn from(value: std::num::ParseIntError) -> Self {
        Self::ParseIntError(value)
    }
}
impl From<std::num::ParseFloatError> for ConfigValueParseError {
    fn from(value: std::num::ParseFloatError) -> Self {
        Self::ParseFloatError(value)
    }
}
impl From<std::str::ParseBoolError> for ConfigValueParseError {
    fn from(value: std::str::ParseBoolError) -> Self {
        Self::ParseBoolError(value)
    }
}

#[derive(Debug)]
pub enum ConfigErr {
    /// The option specified in the config is invalid
    InvalidOption,
    /// The value for the config option is invalid
    InvalidValue(ConfigValueParseError),
    /// The comment/config is missing an appropriate delimiter (`;`, `:`, or `#`)
    MissingDelimiter,
}
impl From<ConfigValueParseError> for ConfigErr {
    fn from(value: ConfigValueParseError) -> Self {
        ConfigErr::InvalidValue(value)
    }
}
impl From<std::num::ParseIntError> for ConfigErr {
    fn from(value: std::num::ParseIntError) -> Self {
        Self::InvalidValue(value.into())
    }
}
impl From<std::num::ParseFloatError> for ConfigErr {
    fn from(value: std::num::ParseFloatError) -> Self {
        Self::InvalidValue(value.into())
    }
}
impl From<std::str::ParseBoolError> for ConfigErr {
    fn from(value: std::str::ParseBoolError) -> Self {
        Self::InvalidValue(value.into())
    }
}

#[derive(Clone)]
pub struct Config {
    value: f32,
    case_sensitive: bool,
    ordered: bool,
    ordered_answers: bool,
    tutorial: bool,
}
impl std::default::Default for Config {
    fn default() -> Self {
        Self {
            value: 1.0,
            case_sensitive: false,
            ordered: true,
            ordered_answers: true,
            tutorial: true,
        }
    }
}

impl Config {
    fn parse_str(base_config: &Config, config_str: &str) -> Result<Self, ConfigErr> {
        let mut config = base_config.clone();

        for cfg in config_str.lines().map(|l| l.trim()) {
            if cfg.starts_with('#') || cfg.is_empty() { continue; } // skip comments and blanks
            
            if !cfg.starts_with(';') { return Err(ConfigErr::MissingDelimiter) }

            let (name, value) = match cfg.split_once(':') {
                Some(t) => t,
                None => (&cfg[..], ""), // just the name comes out (could be used to reset to
                // default)
            };

            // filter out `;`, trim, and replace acceptable name clarification characters
            // (exampleName == example_name == example-name == example name)
            let name = name[1..].trim().replace(['-','_',' '], "").to_lowercase();
            let value = value.trim().to_lowercase();

            match &name[..] {
                "value" => config.value = value.parse()?,
                "casesensitive" => config.case_sensitive = value.parse()?,
                "ordered" => config.ordered = value.parse()?,
                "orderedanswers" => config.ordered_answers = value.parse()?,
                "tutorial" => config.tutorial = value.parse()?,
                _ => return Err(ConfigErr::InvalidOption),
            };
        };

        Ok(config)
    }
}
impl std::str::FromStr for Config {
    type Err = ConfigErr;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Config::parse_str(&Config::default(), s)
    }
}

#[derive(Debug)]
pub enum Answer {
    Correct(String),
    Incorrect(String),
}

pub struct Question {
    title: String,
    answers: Vec<Answer>,
    config: Config,
}

impl Question {
    fn new() -> Question {
        Question {
            title: String::new(),
            answers: Vec::new(),
            config: Config::default(),
        }
    }
}

#[derive(Debug)]
pub enum QuestionErr {
    /// No `?` delimiter was found marking the start of a question
    /// This may be either a comment block or improperly formmated question
    MissingDelimiter,
    ConfigErr(ConfigErr),
    /// Question has no correct answer
    NoCorrectAnswer,
    /// There is only config/comments, this is likely a comment block
    OnlyConfig,
}
impl From<ConfigErr> for QuestionErr {
    fn from(value: ConfigErr) -> Self {
        QuestionErr::ConfigErr(value)
    }
}

impl Question {
    fn parse_str(base_config: &Config, q_text: &str) -> Result<Self, QuestionErr> {
        let mut question = Question::new();

        // parse configs
        // split the quiz by the '?' separator between config and quiz
        let (config_str, q_text) = match q_text.split_once("\n?") {
            Some((cfg, qz)) => (cfg, qz),
            None => {
                // if text starts with `?` (no newline) it's just a question with no config
                if q_text.starts_with('?') {
                    ("", q_text[..].trim_start_matches('?'))
                // else, everything is config/comment
                } else {
                    return Err(QuestionErr::OnlyConfig);
                    //(&q_text[..], "") // questions MUST start with `?` marker
                }
            },
            //None => ("",q_text[..].trim_start_matches('?')), // makes comment blocks harder
        };

        let config = Config::parse_str(base_config, config_str);

        question.config = match config {
            // if the config errors that it's missing a delimiter *and* we know there's no
            // question, there's a good chance that the quiz *meant* to put in a question (as
            // opposed to a comment block) and forgot the delimiter `?`
            Err(ConfigErr::MissingDelimiter) if q_text.is_empty() => return Err(QuestionErr::MissingDelimiter),
            // else just handle it normally
            _ => config?,
        };

        // parse question
        // parse answers
        let mut remaining = q_text;
        while !remaining.is_empty() {
            let part_end = std::cmp::min(
                // each answer starts with \n(+|-), pick the closest one
                std::cmp::min(
                    remaining[1..].find("\n+").unwrap_or(std::usize::MAX),
                    remaining[1..].find("\n-").unwrap_or(std::usize::MAX),
                ), remaining.len()-2 // or consume the rest (+2 later) if neither found
            ) + 2; // add two to consume that newline and split directly before the delimiter
            
            let to_parse;
            (to_parse, remaining) = remaining.split_at(part_end);

            match &to_parse[0..1] {
                "+" => question.answers.push(Answer::Correct(
                    to_parse[1..].trim().replace("\r\n", " ").replace("\n", " ")
                )),
                "-" => question.answers.push(Answer::Incorrect(
                    to_parse[1..].trim().replace("\r\n", " ").replace("\n", " ")
                )),
                _ => question.title = to_parse.trim().replace("\r\n", " ").replace("\n", " "),
            }
        }

        if question.answers.iter().filter(|ans| matches!(ans, Answer::Correct(_))).count() == 0 {
            return Err(QuestionErr::NoCorrectAnswer)
        }

        Ok(question)
    }
}

impl std::str::FromStr for Question {
    type Err = QuestionErr;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse_str(&Config::default(), s)
    }
}

pub struct Quiz {
    /// The File/Quiz -level config
    pub config: Config,

    /// The questions in the quiz
    pub questions: Vec<Question>,

    /// Total point value of all questions combined / max-score
    pub total_score: f32,
}

#[derive(Debug)]
pub enum QuizErr {
    ConfigErr(ConfigErr),
    QuestionErr(QuestionErr),
}
impl From<ConfigErr> for QuizErr {
    fn from(value: ConfigErr) -> Self {
        QuizErr::ConfigErr(value)
    }
}
impl From<QuestionErr> for QuizErr {
    fn from(value: QuestionErr) -> Self {
        QuizErr::QuestionErr(value)
    }
}

impl std::str::FromStr for Quiz {
    type Err = QuizErr;
    fn from_str(quiz_str: &str) -> Result<Self, Self::Err> {
        // split the quiz by the '---' separator between config and quiz
        let (config_str, quiz_text) = match quiz_str.split_once("\n---") {
            Some((cfg, qz)) => (cfg, qz),
            None => ("", &quiz_str[..]),
        };

        // parse the changes to default config
        let config = config_str.parse()?;

        let mut quiz = Quiz {
            config,
            questions: Vec::new(),
            total_score: 0.0,
        };

        // Questions are separated by newlines
        for q_text in quiz_text
            .split("\r\n\r\n") // handle windows blank lines
            .flat_map(|text| text.split("\n\n")) // handle normal linux blank lines
            .filter(|s| !s.is_empty()) {
            quiz.questions.push(
                match Question::parse_str(&quiz.config, q_text) {
                    Err(QuestionErr::OnlyConfig) => continue, // don't push comment/config blocks
                    // as questions
                    
                    other => other?, // else just return errors / add the question
                }
            );
        }

        // add up the total score of all questions
        for question in quiz.questions.iter() {
            quiz.total_score += question.config.value;
        }
        
        Ok(quiz)
    }
}

impl Quiz {
    pub fn take(&self) -> io::Result<f32> {
        let input = stdin();

        if self.config.tutorial {
            println!("\n\
                Hello, welcome to your quiz!\n\
                I'll ask questions and you give the answers; sound good?\n\
            ");

            println!("\
                Questions that don't present options expect you to type your answer; \
                questions that present options with parenthesis expect a single answer \
                (type the number of the answer); and questions that present options with \
                square brackets expect multiple answers (separate them with spaces, \
                semicolons, periods, or commas).\n\
            ");

            println!("\
                Once you've typed your answer, press enter twice to submit. If you made a \
                mistake, don't worry! Pressing enter only once allows you to restart the \
                answering process with a new answer (the last non-empty line is used), no \
                sweat!\n\
            ");

            println!("Your quiz starts now!\n---");
        }

        let mut score = 0.0;

        let mut questions = Vec::new();
        questions.reserve_exact(self.questions.len());

        let mut ordered_questions = vec![];

        // set the order that questions will be asked in
        for question in &self.questions {
            if question.config.ordered {
                ordered_questions.push(question);
            } else {
                questions.push(question);
            }
        }

        // randomly shuffle questions that desire to be randomly shuffled
        shuffle(&mut questions);

        // append questions that desire to be presented in order (multi-part questions, etc)
        questions.append(&mut ordered_questions);

        for question in questions {
            // ask question
            println!("\n{0}", question.title);

            // prep user input
            let mut user_in = String::new();

            // handle typed-answer questions
            if question.answers.len() == 1 {
                print!("\nYour Answer: ");
                stdout().flush()?;
                input.read_line(&mut user_in)?;

                let mut user_answer = String::new();
                while !user_in.trim().is_empty() {
                    user_answer = user_in.clone();
                    
                    user_in.clear();
                    input.read_line(&mut user_in)?;
                }

                if !question.config.case_sensitive {
                    user_answer = user_answer.to_lowercase()
                }

                let ans = match &question.answers[0] {
                    Answer::Correct(ans) => ans,
                    Answer::Incorrect(ans) => ans,
                };

                if ans == user_answer.trim() {
                    score += question.config.value;
                }

                continue;
            }

            // multiple-choice/answer questions
            let num_correct_answers = question.answers
                .iter()
                .filter(|t| matches!(t, Answer::Correct(_)))
                .count();

            let single_correct = num_correct_answers == 1;

            let mut answers: Vec<&Answer> = question.answers.iter().collect();

            if !question.config.ordered_answers {
                shuffle(&mut answers);
            }

            let mut correct_answer_indicies = vec![];

            // display answers
            for (i, answer) in answers.iter().enumerate() {
                let text = match answer {
                    Answer::Incorrect(text) => text,
                    Answer::Correct(text) => { correct_answer_indicies.push(i); text },
                };

                if single_correct {
                    println!("({i}) {text}");
                } else {
                    println!("[{i}] {text}");
                }
            }

            print!("\nYour Answer{0}: ", if single_correct {""} else {"s"});
            stdout().flush()?;

            correct_answer_indicies.sort();

            input.read_line(&mut user_in)?;

            let mut user_answers = vec![];

            while !user_in.trim().is_empty() {
                user_answers = user_in
                    .split(['.', ' ', ';', ','])
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                    .filter_map(|s| s.parse::<usize>().ok())
                    .collect();
                user_answers.sort();
                
                user_in.clear();
                input.read_line(&mut user_in)?;
            }

            if user_answers == correct_answer_indicies {
                score += question.config.value;
            }

            // BUNCHA PARTIAL CREDIT STUFF I DON'T CARE ABOUT RIGHT NOW
            //// get the value for each part of the question
            //let part_value = question.config.value / answers.len() as f32;
            //
            //// add the score of the incorrect answers (score will be subtracted if user submits an
            //// incorrect answer)
            //score += (answers.len() - num_correct_answers) as f32 * part_value;
            //
            //for ans in user_answers {
            //
            //}
        };

        Ok(score)
    }
}

fn shuffle<T>(vec: &mut [T]) {
    let n = vec.len();
    if n == 0 { return }
    for i in 0..(n - 1) {
        let j = rand() % (n - i) + i;
        vec.swap(i, j);
    }
}

fn rand() -> usize {
    RandomState::new().build_hasher().finish() as usize
}
