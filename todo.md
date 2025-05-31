# Active Tasks
- Default values for config options (typing `;case-sensitive` would set the default value)
- more config options
    - ?justified: ask user for justification, log to ...
    - ?ask-inverse: ask user the question to their answer later
    - ?allow-review: if missed, allow user to manually mark correct, log to...

# Future Tasks
- Tons of Documentation
- Bug Hunting
- Binary/Precompiled Quizzes
    - Allow quizzes to be compiled into the Quiz object ahead of time and 
    printed into a file (careful byte-order) as binary to be read by the
    quiz taker at runtime (hides answers / more secure, very dumbly though)
    - .qz / .txt distinction? Or .qzb
