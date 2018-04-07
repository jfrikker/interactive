# interactive

Interactive is a simple utility that adds an interactive mode to any command-line tool.

Example:

```shell
> interactive find .
> find . -name *.rs
./src/escape.rs
./src/main.rs
> find . -name src
./src
> find . ++ -type f
> find . -type f -name src
> find . -type f - type
> find .
```

## How it works

The tool maintains a "base command", which is reflected in the prompt. The base command is initially set via command line arguments, although it can be modified later. At each prompt, the user types additional command arguments. These arguments are combined with the base command, and a new subprocess is spawned.

## Modifying the base command

The tool recognizes a few special commands which can be used to modify the base command.

`+ <option> [<option> ...]`: Adds a new option to the base command. The option string can start with a `-`, in which case it will be appended as-is. Otherwise, either `-` or `--` are prepended, depending on whether the option is one character, or more than one character. Multiple options can be specified at once.

`- <option> [<option> ...]`: The opposite of `+`. This command removes one or more options from the base command. If an option is followed by a non-option argument (no leading `-`), that argument is removed as well. Multiple options can be specified at once.

`++ <option> <arg>`: Adds an option with an argument. Only supports adding one option at a time. If the option in question is already part of the base command, it will be replaced.

## History

This tool tracks command history, and supports conveniences like history searching. It does *not* yet persist history across executions.
