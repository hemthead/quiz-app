use std::io::stdin;
use std::io;

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
    random_q_order: bool,
    random_a_order: bool,
}
impl std::default::Default for Config {
    fn default() -> Self {
        Self {
            value: 1.0,
            case_sensitive: false,
            random_q_order: false,
            random_a_order: true,
        }
    }
}

impl Config {
    ///
    fn parse_str(base_config: &Config, config_str: &str) -> Result<Self, ConfigErr> {
        let mut config = base_config.clone();

        for cfg in config_str.lines().map(|l| l.trim()) {
            if cfg.starts_with('#') { continue; } // skip comments
            
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
                "randomqorder" => config.random_q_order = value.parse()?,
                "randomaorder" => config.random_a_order = value.parse()?,
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
                    (&q_text[..], "") // questions MUST start with `?` marker
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
                "+" => question.answers.push(Answer::Correct(to_parse[1..].trim().to_owned())),
                "-" => question.answers.push(Answer::Incorrect(to_parse[1..].trim().to_owned())),
                _ => question.title = to_parse.trim().to_owned(),
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
    pub config: Config,
    pub questions: Vec<Question>,
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
        };

        // Questions are separated by newlines
        for q_text in quiz_text.split("\n\n").filter(|s| !s.is_empty()) {
            quiz.questions.push(Question::parse_str(&quiz.config, q_text)?);
        }
        
        Ok(quiz)
    }
}

impl Quiz {
    pub fn take(&self) -> io::Result<f32> {
        let input = stdin();

        let mut score = 0.0;
        let mut total_score = 0.0;

        let questions = &self.questions;
        for question in questions {
            total_score += question.config.value;

            // ask question
            println!("{0}", question.title);

            // prep user input
            let mut user_in = String::new();

            // handle typed-answer questions
            if question.answers.len() == 1 {
                input.read_line(&mut user_in)?;

                if !question.config.case_sensitive {
                    user_in = user_in.to_lowercase()
                }

                let ans = match &question.answers[0] {
                    Answer::Correct(ans) => ans,
                    Answer::Incorrect(ans) => ans,
                };

                if ans == &user_in {
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

            let answers = &question.answers;

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

            input.read_line(&mut user_in)?;

            let mut user_answers = vec![];

            while user_in != "n\n" {
                user_answers = user_in
                    .split(['.', ' ', ';', ','])
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                    .filter_map(|s| s.parse::<usize>().ok())
                    .collect()
                ;
                
                user_in.clear();
                input.read_line(&mut user_in)?;
            }

            println!("input: {user_answers:?}");

            if user_answers.sort() == correct_answer_indicies.sort() {
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

        Ok(score / total_score)
    }
}
