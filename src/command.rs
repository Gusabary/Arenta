use chrono::{Datelike, Days, Local, NaiveDate};

#[derive(Debug, PartialEq)]
pub struct ListOption {
    pub date_filter: (DateFilterOp, NaiveDate),
    pub include_backlog: bool,
    pub is_verbose: bool,
    pub has_timeline: bool,
}

impl ListOption {
    fn default() -> Self {
        ListOption {
            date_filter: (DateFilterOp::Equal, Local::now().date_naive()),
            include_backlog: false,
            is_verbose: false,
            has_timeline: false,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum DateFilterOp {
    Earlier,
    EarlierEqual,
    Equal,
    Later,
    LaterEqual,
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
        let args: Vec<&str> = cmd.split_whitespace().collect();
        if args[0] == "ls" || args[0] == "ll" {
            try_parse_list_option(&args).map(Command::List)
        } else if args.len() < 2 {
            None
        } else if args[0] == "s" || args[0] == "start" {
            args[1].parse::<usize>().ok().map(Command::Start)
        } else if args[0] == "c" || args[0] == "complete" {
            args[1].parse::<usize>().ok().map(Command::Complete)
        } else if args[0] == "delete" {
            args[1].parse::<usize>().ok().map(Command::Delete)
        } else if args[0] == "e" || args[0] == "edit" {
            args[1].parse::<usize>().ok().map(Command::Edit)
        } else {
            None
        }
    }
}

fn try_parse_list_option(args: &[&str]) -> Option<ListOption> {
    let mut option = ListOption::default();
    option.has_timeline = if args[0] == "ls" {
        false
    } else if args[0] == "ll" {
        true
    } else {
        return None;
    };
    if let Some(&arg) = args.get(1) {
        if let Some(filter) = try_parse_date_filter(arg) {
            option.date_filter = filter;
        } else if let Some((include_backlog, is_verbose)) = try_parse_bv(arg) {
            option.include_backlog = include_backlog;
            option.is_verbose = is_verbose;
        } else {
            return None;
        }
    }
    if let Some(&arg) = args.get(2) {
        if let Some((include_backlog, is_verbose)) = try_parse_bv(arg) {
            option.include_backlog = include_backlog;
            option.is_verbose = is_verbose;
        } else if let Some(filter) = try_parse_date_filter(arg) {
            option.date_filter = filter;
        } else {
            return None;
        }
    }
    if option.has_timeline && option.date_filter.0 != DateFilterOp::Equal {
        println!("cannot specify <op> when using `ll`");
        None
    } else {
        Some(option)
    }
}

fn try_parse_date_filter(arg: &str) -> Option<(DateFilterOp, NaiveDate)> {
    assert!(!arg.is_empty());
    if let Some(date) = arg.strip_prefix(">=") {
        try_parse_date(date).map(|date| (DateFilterOp::LaterEqual, date))
    } else if let Some(date) = arg.strip_prefix('>') {
        try_parse_date(date).map(|date| (DateFilterOp::Later, date))
    } else if let Some(date) = arg.strip_prefix("<=") {
        try_parse_date(date).map(|date| (DateFilterOp::EarlierEqual, date))
    } else if let Some(date) = arg.strip_prefix('<') {
        try_parse_date(date).map(|date| (DateFilterOp::Earlier, date))
    } else {
        try_parse_date(arg).map(|date| (DateFilterOp::Equal, date))
    }
}

fn try_parse_date(arg: &str) -> Option<NaiveDate> {
    if arg.len() == 5 && arg.chars().nth(2).unwrap() == '-' {
        let date = format!("{}-{}", Local::now().year(), arg);
        if let Ok(date) = NaiveDate::parse_from_str(&date, "%F") {
            return Some(date);
        }
    }
    if let Ok(date) = NaiveDate::parse_from_str(arg, "%F") {
        Some(date)
    } else if let Ok(offset) = arg.parse::<i32>() {
        if offset.is_positive() {
            Local::now()
                .date_naive()
                .checked_add_days(Days::new(offset as u64))
        } else if offset.is_negative() {
            Local::now()
                .date_naive()
                .checked_sub_days(Days::new(offset.unsigned_abs() as u64))
        } else {
            Some(Local::now().date_naive())
        }
    } else {
        None
    }
}

fn try_parse_bv(arg: &str) -> Option<(bool, bool)> {
    if arg.chars().any(|c| c != 'b' && c != 'v') {
        None
    } else {
        Some((arg.contains('b'), arg.contains('v')))
    }
}

pub fn print_command_usage() {
    println!("commands: ");
    println!("  q / quit               quit arenta");
    println!("  h / help               show this message");
    println!("  n / new                create a new task");
    println!("  s / start <index>      start task");
    println!("  c / complete <index>   complete task");
    println!("  e / edit <index>       edit task");
    println!("  delete <index>         delete task");
    println!("  sort                   sort all the tasks");
    println!("  ls [date_filter] [bv]  list tasks, without timeline");
    println!("  ll [date_filter] [bv]  list tasks, with timeline");
    println!("    [date_filter] is in format of `<op><date>`");
    println!("      <op> could be <, <=, >, >= or empty, which indicates `==`, note that for `ll`, <op> must use empty");
    println!("      <date> could in format of mm-dd, yyyy-mm-dd or an integer, which indicates offset to today");
    println!("    if `b` flag specified, it means display backlog tasks as well");
    println!("    if `v` flag specified, it means display in verbose mode");
    println!("    some examples:");
    println!("      ls, ls b, ls +1 v, ll, ll -1, ll 2023-01-26 bv");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_parse_date() {
        assert_eq!(
            try_parse_date("01-26"),
            NaiveDate::from_ymd_opt(Local::now().year(), 1, 26)
        );
        assert_eq!(try_parse_date("01-32"), None);
        assert_eq!(
            try_parse_date("2023-01-27"),
            NaiveDate::from_ymd_opt(2023, 1, 27)
        );
        let today = Local::now().date_naive();
        assert_eq!(try_parse_date("+2"), today.checked_add_days(Days::new(2)));
        assert_eq!(try_parse_date("3"), today.checked_add_days(Days::new(3)));
        assert_eq!(try_parse_date("-1"), today.checked_sub_days(Days::new(1)));
    }

    #[test]
    fn test_try_parse_date_filter() {
        let today = Local::now().date_naive();
        assert_eq!(
            try_parse_date_filter(">=1"),
            Some((
                DateFilterOp::LaterEqual,
                today.checked_add_days(Days::new(1)).unwrap()
            ))
        );
        assert_eq!(
            try_parse_date_filter(">-2"),
            Some((
                DateFilterOp::Later,
                today.checked_sub_days(Days::new(2)).unwrap()
            ))
        );
        assert_eq!(
            try_parse_date_filter("0"),
            Some((DateFilterOp::Equal, today))
        );
        assert_eq!(
            try_parse_date_filter("<4"),
            Some((
                DateFilterOp::Earlier,
                today.checked_add_days(Days::new(4)).unwrap()
            ))
        );
        assert_eq!(
            try_parse_date_filter("<=-1"),
            Some((
                DateFilterOp::EarlierEqual,
                today.checked_sub_days(Days::new(1)).unwrap()
            ))
        );
    }

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
        assert_eq!(
            parse_command("ls"),
            Some(Command::List(ListOption::default()))
        );
        let date_filter = (
            DateFilterOp::LaterEqual,
            Local::now()
                .date_naive()
                .checked_add_days(Days::new(1))
                .unwrap(),
        );
        assert_eq!(
            parse_command("ls >=1"),
            Some(Command::List(ListOption {
                date_filter,
                ..ListOption::default()
            }))
        );
        assert_eq!(
            parse_command("ls >=1 b"),
            Some(Command::List(ListOption {
                date_filter,
                include_backlog: true,
                ..ListOption::default()
            }))
        );
        assert_eq!(
            parse_command("ls bv >=1"),
            Some(Command::List(ListOption {
                date_filter,
                include_backlog: true,
                is_verbose: true,
                ..ListOption::default()
            }))
        );
        assert_eq!(
            parse_command("ll"),
            Some(Command::List(ListOption {
                has_timeline: true,
                ..ListOption::default()
            }))
        );
        assert_eq!(parse_command("ll >=1"), None);
        assert_eq!(
            parse_command("ll b 1"),
            Some(Command::List(ListOption {
                date_filter: (
                    DateFilterOp::Equal,
                    Local::now()
                        .date_naive()
                        .checked_add_days(Days::new(1))
                        .unwrap(),
                ),
                include_backlog: true,
                has_timeline: true,
                ..ListOption::default()
            }))
        );
        assert_eq!(
            parse_command("ll vb"),
            Some(Command::List(ListOption {
                include_backlog: true,
                is_verbose: true,
                has_timeline: true,
                ..ListOption::default()
            }))
        );
    }
}
