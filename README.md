# Arenta

![](https://img.shields.io/badge/version-v1.0.1-9cf) ![](https://img.shields.io/badge/license-MIT-blue)

A terminal-based daily task management tool with minimal overhead.

## Demo

![](./asset/demo.gif)

## Features

+ Complete CRUD support of daily tasks with intuitive syntax
+ Visualized task schedule with colorful timeline
+ A single executable binary and naturally terminal based
+ Retentiveness by storing tasks in local file

## Install

Several alternatives:

+ **Recommended**: `cargo install arenta`
+ `cargo install --git https://github.com/Gusabary/Arenta.git`
+ Download binary from [Release](https://github.com/Gusabary/Arenta/releases) page and put it into `$PATH`
+ Clone the repo and build from source

## Usage

Use `arenta -h` to show help messages and `arenta -v` to check the version.

Use `arenta` to start an interactive session, in which you could manage daily tasks easily.

### Task Definition

**Task** is the core concept in Arenta, which consists of description, planned start/complete time, actual start/complete time and status.

The description and time part could be set and edited with Arenta commands, while the status is managed by Arenta in such rules:

|              | planned start            | planned complete | actual start | actual complete |
| ------------ | ------------------------ | ---------------- | ------------ | --------------- |
| **Backlog**  | unset                    | unset            | unset        | unset           |
| **Planned**  | set and later than now   | set              | unset        | unset           |
| **Overdue**  | set and earlier than now | set              | unset        | unset           |
| **Ongoing**  | set / unset              | set / unset      | set          | unset           |
| **Complete** | set / unset              | set / unset      | set          | set             |

### Command Syntax

The interactive session accepts valid Arenta commands:

+ Type in  `n` or `new` to create a new task in an interactive way:

  ```
  arenta> n
   description: a new task
   how to arrange this task
  > start immediately
    put in backlog
    plan to...
  ```

+ Type in `s` or `start` plus a task index to start it:

  ```
  arenta> s 0
  task 0 started
  ```

+ Type in `c` or `complete` plus a task index to complete it:

  ```
  arenta> c 0
  task 0 completed
  ```

+ Type in `ls` or `ll` with `[date_filter]` and `[flags]` to list all tasks in specified date range.

  + `ll` will render a visualized timeline as well, in which the planned period is represented as `-` while actual period is `=`
  + `[date_filter]` is in format of `<op><date>`
    + `<op>` could be `<`, `<=`, `>`, `>=` or empty, which indicates `==`. Note that for `ll`, `<op>` must be empty.     
    + `<date>` could take format of `mm-dd`, `yyyy-mm-dd` or just an integer, which indicates offset to today.
  + `[flags]` could contain `b` or `v`
    + `b` flag to display backlog tasks as well
    + `v` flag to display in verbose mode
  
  ```
  # list today's tasks
  > ls
  
  # list tomorrow's tasks with timeline
  > ll +1
  
  # list all historical tasks in verbose mode
  > ls <0 v
  
  # list yesterday's tasks including backlog and timeline in verbose mode
  > ll -1 bv
  ```
  
+ Type in `h` or `help` to show the complete usage of all Arenta commands

## Todos

+ [usability] make the Arenta interactive session more shell-like, e.g. can use up arrow key to pop up last command
+ [scalability] take a more scalable approach to save all tasks to local file
+ [customizability] expose some settings as configurable, e.g. length of timeline, color of status, task display pattern, etc.

## License

[MIT](https://github.com/Gusabary/Arenta/blob/master/LICENSE)

