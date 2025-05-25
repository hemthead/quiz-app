use std::io::{self, Write, stdin, stdout};

use std::cmp;

use std::hash::{BuildHasher, Hasher, RandomState};

/* consider doing something like this
enum ConfigValue {
    F32(String),
    Bool(String),
}
impl ConfigValue {
    fn parse
}
*/

#[derive(Debug)]
pub enum ConfigValueParseError {
    ParseIntError(std::num::ParseIntError),
    ParseFloatError(std::num::ParseFloatError),
    ParseBoolError(std::str::ParseBoolError),
}
impl std::fmt::Display for ConfigValueParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ParseIntError(e) => write!(f, "{e}"),
            Self::ParseFloatError(e) => write!(f, "{e}"),
            Self::ParseBoolError(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for ConfigValueParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(match self {
            Self::ParseFloatError(e) => e,
            Self::ParseIntError(e) => e,
            Self::ParseBoolError(e) => e,
        })
    }
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
pub struct ConfigError {
    /// the string that failed to parse
    context: String,
    /// how many lines of config were parsed before this error came up
    lines_parsed: usize,
    /// what kind of error this is 
    kind: ConfigErrorKind,
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "failed to parse '{0}': {1}", self.context, self.kind)
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            ConfigErrorKind::InvalidValue(err) => {
                err.source()
            },
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum ConfigErrorKind {
    /// The option specified in the config is invalid
    InvalidOption,
    /// The value for the config option is invalid
    InvalidValue(ConfigValueParseError),
    /// The comment/config is missing an appropriate delimiter (`;`, `:`, or `#`)
    MissingDelimiter,
}

impl std::fmt::Display for ConfigErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidOption => write!(f, "invalid config option"),
            Self::MissingDelimiter => write!(f, "missing `;` delimiter"),
            Self::InvalidValue(e) => write!(f, "{e}"),
        }
    }
}

impl From<ConfigValueParseError> for ConfigErrorKind {
    fn from(value: ConfigValueParseError) -> Self {
        ConfigErrorKind::InvalidValue(value)
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
    fn convert_parsed<T>(parsed: Result<T, ConfigValueParseError>, value: String, lines_parsed: usize) -> Result<T, ConfigError> {
        match parsed {
            Err(e) => return Err(ConfigError{
                kind: ConfigErrorKind::from(e),
                context: value,
                lines_parsed,
            }),
            Ok(v) => Ok(v),
        }
    }

    fn parse_str(base_config: &Config, config_str: &str) -> Result<Self, ConfigError> {
        let mut config = base_config.clone();

        for (line_num, cfg) in config_str.lines().map(|l| l.trim()).enumerate() {
            if cfg.starts_with('#') || cfg.is_empty() { continue; } // skip comments and blanks
            
            if !cfg.starts_with(';') { return Err(ConfigError{
                kind: ConfigErrorKind::MissingDelimiter,
                lines_parsed: line_num,
                context: cfg.to_owned(),
            })}

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
 //                "value" => config.value = match value.parse() {
 //                    Err(e) => return Err(ConfigError{
 //                        kind: ConfigErrorKind::from(e),
 //                        context: value,
 //                        lines_parsed: line_num,
 //                    }),
 //                    Ok(v) => v,
 //                },
                // Ok, we're gonna pull an ugly, but this lets us add context so, *shrug*
                "value" => config.value = Self::convert_parsed(value.parse::<f32>().map_err(|e| e.into()), value, line_num)?,
                "casesensitive" => config.case_sensitive = Self::convert_parsed(value.parse::<bool>().map_err(|e| e.into()), value, line_num)?,
                "ordered" => config.ordered = Self::convert_parsed(value.parse::<bool>().map_err(|e| e.into()), value, line_num)?,
                "orderedanswers" => config.ordered_answers = Self::convert_parsed(value.parse::<bool>().map_err(|e| e.into()), value, line_num)?,
                "tutorial" => config.tutorial = Self::convert_parsed(value.parse::<bool>().map_err(|e| e.into()), value, line_num)?,
                _ => return Err(ConfigError { 
                    kind: ConfigErrorKind::InvalidOption,
                    lines_parsed: line_num,
                    context: name,
                }),
            };
        };

        Ok(config)
    }
}
impl std::str::FromStr for Config {
    type Err = ConfigError;
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
pub struct QuestionError {
    kind: QuestionErrorKind,
    lines_parsed: usize,
    context: String,
}

impl std::fmt::Display for QuestionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "failed to parse question '{0}': {1}", self.context, self.kind)
    }
}

impl std::error::Error for QuestionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            QuestionErrorKind::ConfigError(e) => Some(e),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum QuestionErrorKind {
    /// No `?` delimiter was found marking the start of a question
    /// This may be either a comment block or improperly formmated question
    MissingDelimiter,
    ConfigError(ConfigError),
    /// Question has no correct answer
    NoCorrectAnswer,
    /// There is only config/comments, this is likely a comment block
    OnlyConfig,
}

impl std::fmt::Display for QuestionErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingDelimiter => write!(f, "missing `?` delimiter before question"),
            Self::ConfigError(e) => write!(f, "{e}"),
            Self::NoCorrectAnswer => write!(f, "no correct answer"),
            Self::OnlyConfig => write!(f, "only config (likely a comment)"),
        }
    }
}

fn to_context_string(s: &str) -> String {
    let max_len = 32;
    let indx = cmp::min(max_len,
        cmp::min(
            s.find('\n').unwrap_or(std::usize::MAX),
            s.find('\r').unwrap_or(std::usize::MAX),
        ),
    );

    if let Some(substr) = s.get(..indx) {
        substr.to_owned()
    } else {
        s[..].to_owned()
    }
}

impl Question {
    fn parse_str(base_config: &Config, q_text: &str) -> Result<Self, QuestionError> {
        let mut question = Question::new();

        let mut lines_parsed = 0;

        // parse configs
        // split the quiz by the '?' separator between config and quiz
        let (config_str, q_text) = match q_text.split_once("\n?") {
            Some((cfg, qz)) => {
                lines_parsed = 1; // to account for the newline we just got rid of
                (cfg, qz.trim())
            },
            None => {
                // if text starts with `?` (no newline) it's just a question with no config
                if q_text.starts_with('?') {
                    ("", q_text[..].trim_start_matches('?').trim())
                // else, everything is config/comment
                } else {
                    (&q_text[..], "") // questions MUST start with `?` marker
                }
            },
            //None => ("",q_text[..].trim_start_matches('?')), // makes comment blocks harder
        };

        // set up the context to return when the user
        let question_context = to_context_string(q_text);

        let config = Config::parse_str(base_config, config_str);

        lines_parsed += config_str.matches('\n').count();

        question.config = match config {
            // if the config errors that it's missing a delimiter *and* we know there's no
            // question, there's a good chance that the quiz *meant* to put in a question (as
            // opposed to a comment block) and forgot the delimiter `?`
            Err(cfg_err) if matches!(cfg_err.kind, ConfigErrorKind::MissingDelimiter) && q_text.is_empty() => return Err(QuestionError {
                kind: QuestionErrorKind::MissingDelimiter,
                lines_parsed: cfg_err.lines_parsed, // we errored before the config was
                // over, so it knows the actual line number
                context: question_context, // show the start
                // of the would-be question
            }),
            // propogate other errors
            Err(cfg_err) => return Err(QuestionError {
                lines_parsed: cfg_err.lines_parsed,
                kind: QuestionErrorKind::ConfigError(cfg_err),
                context: question_context,
            }),
            // else just handle it normally
            Ok(cfg) => cfg,
        };

        if q_text.is_empty() {
            return Err(QuestionError {
                kind: QuestionErrorKind::OnlyConfig,
                context: to_context_string(q_text),
                lines_parsed,
            });
        }

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

        // err if there are no correct answers
        if question.answers.iter().filter(|ans| matches!(ans, Answer::Correct(_))).count() == 0 {
            return Err(QuestionError {
                kind: QuestionErrorKind::NoCorrectAnswer,
                lines_parsed: lines_parsed,
                context: question_context,
            });
        }

        Ok(question)
    }
}

impl std::str::FromStr for Question {
    type Err = QuestionError;
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
pub struct QuizError {
    kind: QuizErrorKind,
    /// note that lines_parsed is *not* the current line / error line, it's the number of the last
    /// line parsed (or index of the error line)
    lines_parsed: usize,
}

impl std::fmt::Display for QuizError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error on line {0}: {1}", self.lines_parsed + 1, self.kind)
    }
}

impl std::error::Error for QuizError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(match &self.kind {
            QuizErrorKind::ConfigError(e) => e,
            QuizErrorKind::QuestionError(e) => e,
        })
    }
}

#[derive(Debug)]
pub enum QuizErrorKind {
    ConfigError(ConfigError),
    QuestionError(QuestionError),
}
impl std::fmt::Display for QuizErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ConfigError(e) => write!(f, "{e}"),
            Self::QuestionError(e) => write!(f, "{e}"),
        }
    }
}


impl From<ConfigError> for QuizErrorKind {
    fn from(value: ConfigError) -> Self {
        QuizErrorKind::ConfigError(value)
    }
}
impl From<QuestionError> for QuizErrorKind {
    fn from(value: QuestionError) -> Self {
        QuizErrorKind::QuestionError(value)
    }
}

impl std::str::FromStr for Quiz {
    type Err = QuizError;
    fn from_str(quiz_str: &str) -> Result<Self, Self::Err> {
        // split the quiz by the '---' separator between config and quiz
        let (config_str, quiz_text) = match quiz_str.split_once("\n---") {
            Some((cfg, qz)) => (cfg, qz),
            None => ("", &quiz_str[..]),
        };

        // parse the changes to default config
        let config: Config = match config_str.parse() {
            Ok(cfg) => cfg,
            Err(cfg_err) => return Err(QuizError {
                lines_parsed: cfg_err.lines_parsed,
                kind: cfg_err.into(),
            }),
        };

        let mut quiz = Quiz {
            config,
            questions: Vec::new(),
            total_score: 0.0,
        };

        // count the newlines in the config
        let mut lines_parsed = config_str.matches('\n').count();
        if config_str.len() != 0 {
            lines_parsed += 1; // add the line from `\n---`
        }

        // Questions are separated by newlines
        for q_text in quiz_text
            .split("\r\n\r\n") // handle windows blank lines
            .flat_map(|text| text.split("\n\n")) // handle normal linux blank lines
            {
            if !q_text.is_empty() { 
                match Question::parse_str(&quiz.config, q_text) {
                    Err(question_err) if matches!(question_err.kind, QuestionErrorKind::OnlyConfig) => (), // don't push comment/config blocks
                    // as questions

                    Err(e) => return Err(QuizError {
                        lines_parsed: lines_parsed + e.lines_parsed, // where the question is +
                        // where the error is in the question
                        kind: QuizErrorKind::QuestionError(e),
                    }),
                    
                    Ok(question) => quiz.questions.push(question), // else just return errors / add the question
                }
            }

            // 2 for the two newlines before each question-block + the newlines in the question
            lines_parsed += 2 + q_text.matches('\n').count();
            if q_text.is_empty() { // account for blank lines
                //lines_parsed += 1;
            }
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

                let mut ans = match &question.answers[0] {
                    Answer::Correct(ans) => ans,
                    Answer::Incorrect(_) => unreachable!(), // this question would fail to parse
                    // with `NoCorrectAnswer`
                }.to_owned();

                if !question.config.case_sensitive {
                    user_answer = user_answer.to_lowercase();
                    ans = ans.to_lowercase();
                }

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
