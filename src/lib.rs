// NOTE: AFAIK, there's no way to make sure that the FromStr implementation covers all these cases,
// so make sure you fix the implementation each time this enum changes
pub enum ConfigOption {
    RandomQuestionOrder(bool),
    RandomAnswerOrder(bool),
    CaseSensitive(bool),
    Value(f32),
}

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

pub enum ConfigErr {
    /// The option specified in the config is invalid
    InvalidOption,
    /// The value for the config option is invalid
    InvalidValue(ConfigValueParseError),
    /// The comment/config is missing an appropriate delimiter (`;`, `:`, or `#`)
    MissingDelimiter,
    /// The config is a comment
    Comment,
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

impl std::str::FromStr for ConfigOption {
    type Err = ConfigErr;
    fn from_str(cfg: &str) -> Result<Self, Self::Err> {
        if cfg.starts_with('#') { return Err(ConfigErr::Comment) } // skip comments
        
        if !cfg.starts_with(';') { return Err(ConfigErr::MissingDelimiter) }

        let cfg = cfg[1..].trim().replace(['-','_',' '], "").to_lowercase();

        let (name, value) = match cfg.split_once(':') {
            Some(t) => t,
            None => (&cfg[..], ""),
        };

        return Ok(match name {
            "value" => ConfigOption::Value(value.parse()?),
            "casesensitive" => ConfigOption::CaseSensitive(value.parse()?),
            "randomqorder" => ConfigOption::RandomQuestionOrder(value.parse()?),
            "randomaorder" => ConfigOption::RandomAnswerOrder(value.parse()?),
            _ => return Err(ConfigErr::InvalidOption),
        })
    }
}

pub struct Config(Vec<ConfigOption>);
impl Config {
    fn new() -> Config {
        Config(Vec::new())
    }
}

impl std::str::FromStr for Config {
    type Err = ConfigErr;
    fn from_str(config_str: &str) -> Result<Self, Self::Err> {
        let mut config = Config::new();

        for cfg in config_str.lines() {
            config.0.push(cfg.parse()?);
        };

        Ok(config)
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
            config: Config::new(),
        }
    }
}

pub enum QuestionErr {
    /// No `?` delimiter was found marking the start of a question
    /// This may be either a comment block or improperly formmated question
    MissingDelimiter,
    ConfigErr(ConfigErr),
}
impl From<ConfigErr> for QuestionErr {
    fn from(value: ConfigErr) -> Self {
        QuestionErr::ConfigErr(value)
    }
}

impl std::str::FromStr for Question {
    type Err = QuestionErr;
    fn from_str(q_text: &str) -> Result<Self, Self::Err> {
        let mut question = Question::new();

        // parse configs
        // split the quiz by the '?' separator between config and quiz
        let (config_str, q_text) = match q_text.split_once("\n?") {
            Some((cfg, qz)) => (cfg, qz),
            None => (&q_text[..], ""), // questions MUST start with `?` marker
            //None => ("",q_text[..].trim_start_matches('?')), // makes comment blocks harder
        };
        let config = config_str.parse();

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

        Ok(question)
    }
}

pub struct Quiz {
    config: Config,
    questions: Vec<Question>,
}

enum QuizErr {
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
        let mut quiz = Quiz {
            config: Config::new(),
            questions: Vec::new(),
        };

        // split the quiz by the '---' separator between config and quiz
        let (config_str, quiz_text) = match quiz_str.split_once("\n---") {
            Some((cfg, qz)) => (cfg, qz),
            None => ("", &quiz_str[..]),
        };

        // TODO: implement default config
        quiz.config = config_str.parse()?;

        // Questions are separated by newlines
        for q_text in quiz_text.split("\n\n").filter(|s| !s.is_empty()) {
            quiz.questions.push(q_text.parse()?);
        }
        
        Ok(quiz)
    }
}
