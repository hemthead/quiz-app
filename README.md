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

All config options are explained in the template file, as well as multiple examples of how to format questions.

### Taking Quizzes

Taking quizzes is as simple as running the executable with the path to the quiz's file as the first argument 
(ex. `$ ./quiz_app examples/template.qz` from a shell in this directory).

## Issues

Please report any issues you have either directly to me or through github issues. I'll try to address them as quickly as possible.
