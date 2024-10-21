# todolint

todolint is a lightweight tool designed to collect and process `todo` marked comments across project files. It adheres to a defined [specification](#comment-specification) for comment structure, where comment tagging is a key feature.

## Existing Features
(todo)


## Future Features
Features to be added, with high priority:
- Fixing line analyzer logic (i.e. all tests pass)
- Add additional unit tests and functional tests

Features to be added, in no particular order:
- Support for multi-line comments (e.g. `/* ... */`)
- Basic performance metrics: speed, memory usage, size of output
- Parallelization for improved processing speed
- Using I/O buffers for file reads
- Improved logging (generals logs + errors)
- Support for a user-provided config file
- Support for "flags":
  - Output to terminal or a BSON file or both
  - "Audit" feature for correcting invalid entries
  - Specify location of config file
- For existing scanned files, use of file modification time and last scan time to determine rescanning necessity


## Potential Future Features
Features that have potential but require additional thought:

(todo)


## Output Options
(todo)

## Comment Specification
(todo)

## Dev Details



### Diagrams

High-level diagram of the app's logic.

```mermaid
flowchart TD

subgraph app
    Start[Start]
    Finish[Finish]
end

subgraph proj
    CheckDir{"`Is Directory Empty
    **or**
    Unmodified Since Last Scan?`"}
    NextDir[Go to Next Directory]
    AnalyzeDir[Analyze Directory]
    MoreDirs{More Directories to Scan?}

    Start --> CheckDir
    CheckDir -->|Yes| NextDir
    CheckDir -->|No| AnalyzeDir
    MoreDirs -->|Yes| CheckDir
    NextDir --> MoreDirs
    MoreDirs -->|No| Finish
end

subgraph dir
    CheckFile{"`Is File Empty
    **or**
    Unmodified Since Last Scan?`"}
    NextFile[Go to Next File]
    AnalyzeFile[Analyze File]
    MoreFiles{More Files to Scan?}

    AnalyzeDir --> CheckFile
    CheckFile -->|Yes| NextFile
    CheckFile -->|No| AnalyzeFile
    MoreFiles -->|Yes| CheckFile
    NextFile --> MoreFiles
    MoreFiles -->|No| MoreDirs
end

subgraph line
    CheckLine{Does line pass general regex?}
    ProcessLine[Process Matching Line]
    NextLine[Go to Next Line]
    MoreLines{More Lines?}

    AnalyzeFile --> CheckLine
    CheckLine -->|Yes| ProcessLine
    CheckLine -->|No| NextLine
    NextLine --> MoreLines
    MoreLines -->|Yes| CheckLine
    MoreLines -->|No| MoreFiles
end

subgraph matched_line
    ValidTodo{Is 'todo' content valid?}
    MoreMatchedLines{More matched lines?}
    NextMatchedLine[Go to Next Matched Line]
    ValidOutcome[Process Valid Outcome]
    InvalidOutcome[Process Invalid Outcome]

    ProcessLine --> ValidTodo
    ValidTodo -->|Yes| ValidOutcome
    ValidTodo -->|No| InvalidOutcome
    MoreMatchedLines --> |Yes| ProcessLine
    MoreMatchedLines -->|No| MoreLines
    ValidOutcome --> NextMatchedLine
    InvalidOutcome --> NextMatchedLine
    NextMatchedLine --> MoreMatchedLines
end
```

