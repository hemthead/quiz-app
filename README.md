# Quiz App
Yep, that's a generic name.

What does it do? Allow you to take QUIZZES!

Develop your own custom quizzes in this phenomenal new product never before seen in
western OSS markets. Using only your wits, carve and compete in endless challenges of your
own derivation! Quiz App allows you to unleash your inner Quizling and develop the quizzes
you wish your teacher would've given you. Want a quiz about the types of rocks? It's 
yours, my friend, as long as you have enough ~rupies~ *questions*! How about a vocab quiz
of everyone's favorite language, toki pona? Yep, that too! And what if you want to hone
your programming skills with a quiz on C's Structures? Whatever you desire, my friend!

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
You'll likely need to set a custom command such as `<terminal-emulator> 
<path-to-quiz-app>`.

#### NixOS

By virtue of it being my OS, NixOS (and nix flake users in-gen.) gets special treatment.
You can download the standalone executable, or use nix flakes to download/install/run the
program.

## Usage

### Developing Quizzes

The template at `examples/template.qz` has everything you need to get a grip on the
configuration and development of custom quizzes. Don't hesistate to contact me if
something's not immediately clear.

The config language is rather basic and the file structure consits of the following:
- Optional file-wide config options, followed by a line consisting of `---` (think yaml)
- Questions formatted as such:
    - Config options and comments; config starting with `;` and comments starting with `#`
    - Followed by a line starting with `?` that begins the question. The question spans 
    until
    - One of the answer identifiers is met on a new line: `-` for incorrect answers and 
    `+` for correct answers

All config options are explained in the template file, as well as multiple examples of how
to format questions.

#### Quiz Errors

When developing a quiz you're liable to run into some inconvenient errors. Perhaps you
forgot the name of a config option or neglected to put a `?` delimiter before your
question. In this case, the program will fail to read the quiz and print the error and
where it occured.

All quiz errors you encounter should print out the number of the line that errored so that
you can quickly fix the issues. They'll also include a description of the error and some
context as to what caused the error. Question errors will include a snippet of the
question so you can more easily find it.

Errors with questions will typically point to the start of the question (right after
question-specific config) as the error position because it's not clear *where* you'll want
to add the fix (especially in the case of 'no correct answer'). I've found that's it's
usually more desireable to know the question than the specific spot.

Unfortunately, as of now, only one error will be reported at a time, so you'll have to
iteratively fix them one-by-one until there are no more left. In the future I plan to list
all the errors so you can fix them at once, but that's for the future, sorry!

### Taking Quizzes

Taking quizzes is as simple as running the executable with the path to the quiz's file as
the first argument (ex. `$ ./quiz_app examples/template.qz` from a shell in this 
directory).

## Issues

Please report any issues you have either directly to me or through github issues. I'll try
to address them as quickly as possible.
