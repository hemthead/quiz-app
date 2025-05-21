# Quiz App
Yep, that's a generic name.

What does it do? Allow you to take QUIZZES!

Develop your own custom quizzes in this phenomenal new product never before seen in
western OSS markets. Using only your wits, carve and compete in endless challenges of
your own derivation! Quiz App allows you to unleash your inner Quizling and develop
the quizzes you wish your teacher would've given you. Want a quiz about the types of
rocks? It's yours, my friend, as long as you have enough ~rupies~ *questions*! How
about a vocab quiz of everyone's favorite language, toki pona? Yep, that too! And what
if you want to hone your programming skills with a quiz on C's Structures? Whatever you
desire, my friend!

Ok, enough with the marketing...

## Installation

### Windows

Download the executable from a release and double click! (or you can always compile for
yourself if you have rust set up) A terminal window will automatically pop up and ask 
for the path to a quiz to take.

If you want, you can set `.qz` files to open with the executable by right clicking a `.qz`
file, selecting `Open With` and browsing to the executable. Then there's a checkbox to set
it as default.

### Linux

Download the executable and run from a terminal emulator. The first and only argument
should be the path to the quiz to take (or you can forgoe any arguments to be prompted).

Consult your file-manager for how to set the app as the default way to open `.qz` files.
You'll likely need to set a custom command such as `<terminal-emulator> <path-to-quiz-app>`.

#### NixOS

By virtue of it being my OS, NixOS (and nix flake users in-gen.) gets special treatment. You
can download the standalone executable, or use nix flakes to download/install/run the program.

## Usage

### Developing Quizzes

The template at `examples/template.qz` has everything you need to get a grip on the
configuration and development of custom quizzes. Don't hesistate to contact me if
something's not immediately clear.

The config language is rather basic and the file structure consits of the following:
- Optional file-wide config options, followed by a line consisting of `---` (think yaml)
- Questions formatted as such:
    - Config options and comments; config starting with `;` and comments starting with `#`
    - Followed by a line starting with `?` that begins the question. The question spans until
    - One of the answer identifiers is met on a new line: `-` for incorrect answers and `+` for correct answers
    - (Note that for questions with a single answer it will be assuemed that that is the intended correct answer.)

All config options are explained in the template file, as well as multiple examples of
how to format questions.

#### Quiz Errors

When developing a quiz you're liable to run into some inconvenient errors. Perhaps you
forgot the name of a config option or neglected to put a `?` delimiter before your
question. In this case, the program will fail to read the quiz and print the error before
exiting. In the future, I plan to include line-numbers and other context in this printed
output, but currently only the name of the error is printed out.

There are a multitude of possible errors, but most have names that are (hopefully) self
explanatory as to what the problem is. The following is a table of errors that may result
from improper quiz formatting/syntax.

| Error | Description |
| --- | --- |
| `QuestionErr(MissingDelimiter)` | The parser sees a config block that has an invalid config line, you probably meant to ask a question and forgot the `?` delimiter. |
| `QuestionErr(NoCorrectAnswer)` | The parser sees a question that has no correct answer. |
| `QuestionErr(OnlyConfig)` | The parser sees a block that has no question, but also probably wasn't intended to be a question, i.e. a comment block. This error will never show up as comment blocks are allowed. |
| `QuestionErr(ConfigErr(...))` | Refer to the following `ConfigErr`s. |
| `ConfigErr(InvalidOption)` | The parser sees an invalid config option name (you tried to set an option that doesn't exist). |
| `ConfigErr(InvalidValue(...))` | The parser sees a config option with an unparseable value (ex. you set something like `value: true`, which doesn't make sense). |
| `ConfigErr(MissingDelimiter)` | The parser sees a config line that is missing the `;` or `#` delimiter. |

If the errors seem unhelpful, that's because they are right now. I am working on a revamp
of the error system that will make everything much simpler and user-friendly!

### Taking Quizzes

Taking quizzes is as simple as running the executable with the path to the quiz's file as
the first argument (ex. `$ ./quiz_app examples/template.qz` from a shell in this directory).

## Issues

Please report any issues you have either directly to me or through github issues. I'll try to
address them as quickly as possible.
