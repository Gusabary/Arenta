#[derive(Debug, PartialEq)]
pub enum ListOption {
    Naive,
    HistoricalDays(usize),
    Timeline,
}

#[derive(Debug, PartialEq)]
pub enum Command {
    Empty,
    Quit,
    Help,
    New,
    Sort,
    Start(usize),
    Complete(usize),
    Delete(usize),
    Edit(usize),
    List(ListOption),
}

pub fn parse_command(cmd: &str) -> Option<Command> {
    let cmd = cmd.trim();
    if cmd.is_empty() {
        Some(Command::Empty)
    } else if cmd == "q" || cmd == "quit" {
        Some(Command::Quit)
    } else if cmd == "h" || cmd == "help" {
        Some(Command::Help)
    } else if cmd == "n" || cmd == "new" {
        Some(Command::New)
    } else if cmd == "sort" {
        Some(Command::Sort)
    } else {
        let args: Vec<&str> = cmd.split(' ').collect();
        if args[0] == "ls" {
            if args.len() == 1 {
                Some(Command::List(ListOption::Naive))
            } else if args.len() == 2 && args[1] == "-t" {
                Some(Command::List(ListOption::Timeline))
            } else if args.len() == 3 && args[1] == "-d" {
                if let Ok(days) = args[2].parse::<usize>() {
                    Some(Command::List(ListOption::HistoricalDays(days)))
                } else {
                    None
                }
            } else {
                None
            }
        } else if args.len() < 2 {
            None
        } else if args[0] == "s" || args[0] == "start" {
            if let Ok(index) = args[1].parse::<usize>() {
                Some(Command::Start(index))
            } else {
                None
            }
        } else if args[0] == "c" || args[0] == "complete" {
            if let Ok(index) = args[1].parse::<usize>() {
                Some(Command::Complete(index))
            } else {
                None
            }
        } else if args[0] == "delete" {
            if let Ok(index) = args[1].parse::<usize>() {
                Some(Command::Delete(index))
            } else {
                None
            }
        } else if args[0] == "e" || args[0] == "edit" {
            if let Ok(index) = args[1].parse::<usize>() {
                Some(Command::Edit(index))
            } else {
                None
            }
        } else {
            None
        }
    }
}

pub fn print_command_usage() {
    println!("commands: ");
    println!("  q / quit              quit arenta");
    println!("  h / help              show this message");
    println!("  n / new               create a new task");
    println!("  s / start <index>     start task");
    println!("  c / complete <index>  complete task");
    println!("  e / edit <index>      edit task");
    println!("  delete <index>        delete task");
    println!("  ls [-d <n>]           list tasks of recent <n> days");
    println!("  ls -t                 list today's tasks with timeline");
    println!("  sort                  sort all the tasks");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_command() {
        assert_eq!(parse_command(""), Some(Command::Empty));
        assert_eq!(parse_command("  "), Some(Command::Empty));
        assert_eq!(parse_command("q "), Some(Command::Quit));
        assert_eq!(parse_command(" quit "), Some(Command::Quit));
        assert_eq!(parse_command("h"), Some(Command::Help));
        assert_eq!(parse_command(" help"), Some(Command::Help));
        assert_eq!(parse_command("n"), Some(Command::New));
        assert_eq!(parse_command("new"), Some(Command::New));
        assert_eq!(parse_command("sort "), Some(Command::Sort));
        assert_eq!(parse_command("s 1"), Some(Command::Start(1)));
        assert_eq!(parse_command("start 2"), Some(Command::Start(2)));
        assert_eq!(parse_command("start a"), None);
        assert_eq!(parse_command("c 1"), Some(Command::Complete(1)));
        assert_eq!(parse_command("complete 2"), Some(Command::Complete(2)));
        assert_eq!(parse_command("complete a"), None);
        assert_eq!(parse_command("d 1"), None);
        assert_eq!(parse_command("delete 2"), Some(Command::Delete(2)));
        assert_eq!(parse_command("delete a"), None);
        assert_eq!(parse_command("e 1"), Some(Command::Edit(1)));
        assert_eq!(parse_command("edit 2"), Some(Command::Edit(2)));
        assert_eq!(parse_command("edit a"), None);
        assert_eq!(parse_command("ls"), Some(Command::List(ListOption::Naive)));
        assert_eq!(parse_command("lss"), None);
        assert_eq!(
            parse_command("ls -t"),
            Some(Command::List(ListOption::Timeline))
        );
        assert_eq!(
            parse_command("ls -d 1"),
            Some(Command::List(ListOption::HistoricalDays(1)))
        );
        assert_eq!(parse_command("ls -d n"), None);
        assert_eq!(parse_command("ls -d 1 3"), None);
    }
}
