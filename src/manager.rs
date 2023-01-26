use crate::command::{parse_command, print_command_usage, Command, DateFilterOp, ListOption};
use crate::task::{Task, TaskStatus};
use crate::timeline::Timeline;
use chrono::{DateTime, Duration, Local, NaiveDateTime, NaiveTime, TimeZone};
use csv::{ReaderBuilder, StringRecord, Writer};
use inquire::error::InquireResult;
use inquire::{
    ui::{RenderConfig, Styled},
    CustomType, DateSelect, Select, Text,
};
use std::cmp::Ordering;
use std::path::PathBuf;

pub struct Manager {
    tasks: Vec<Task>,
}

fn get_arenta_file_path() -> PathBuf {
    let mut arenta_file = dirs::home_dir().unwrap();
    arenta_file.push(".arenta");
    arenta_file
}

fn load_tasks_from_file() -> Vec<Task> {
    let reader = ReaderBuilder::new()
        .has_headers(false)
        .from_path(get_arenta_file_path().as_path());
    if reader.is_err() {
        return vec![];
    }
    fn record_to_task(record: StringRecord) -> Task {
        assert_eq!(record.len(), 5);
        let planned_start = datetime_opt_from_string(record.get(1).unwrap());
        let planned_complete = datetime_opt_from_string(record.get(2).unwrap());
        let actual_start = datetime_opt_from_string(record.get(3).unwrap());
        let actual_complete = datetime_opt_from_string(record.get(4).unwrap());
        if planned_start.is_some() != planned_complete.is_some() {
            panic!("planned start and complete should always come in pair");
        }
        if planned_start.is_some() && planned_start.unwrap() > planned_complete.unwrap() {
            panic!("planned start shouldn't be later than planned complete");
        }
        if actual_start.is_some()
            && actual_complete.is_some()
            && actual_start.unwrap() > actual_complete.unwrap()
        {
            panic!("actual start shouldn't be later than actual complete");
        }
        Task {
            description: record.get(0).unwrap().to_string(),
            planned_start,
            planned_complete,
            actual_start,
            actual_complete,
            status: TaskStatus::Planned,
        }
    }
    reader
        .unwrap()
        .records()
        .map(|result| record_to_task(result.unwrap()))
        .collect::<Vec<_>>()
}

impl Manager {
    pub fn new() -> Self {
        Manager {
            tasks: load_tasks_from_file(),
        }
    }

    pub fn start_loop(&mut self) {
        inquire::set_global_render_config(get_render_config());
        self.update_status_of_all_tasks();
        loop {
            let command = Text::new("arenta>").prompt();
            if command.is_err() {
                eprintln!("command error, exit");
                break;
            }
            let command = parse_command(&command.unwrap());
            if command.is_none() {
                println!("invalid command, type `h` to show usage");
                continue;
            }
            match self.dispatch_command(&command.unwrap()) {
                Err(..) => {
                    eprintln!("command error, exit");
                    break;
                }
                Ok(true) => break,
                Ok(false) => (),
            }
        }
    }

    fn dispatch_command(&mut self, command: &Command) -> InquireResult<bool> {
        match command {
            Command::Empty => return Ok(false),
            Command::Quit => return Ok(true),
            Command::Help => print_command_usage(),
            Command::New => self.new_task()?,
            Command::Sort => self.sort_tasks(),
            Command::Start(index) => self.start_task(*index),
            Command::Complete(index) => self.complete_task(*index),
            Command::Delete(index) => self.delete_task(*index),
            Command::Edit(index) => self.edit_task(*index)?,
            Command::List(list_option) => match list_option.has_timeline {
                true => self.list_tasks_with_timeline(list_option),
                false => self.list_tasks(list_option),
            },
        }
        Ok(false)
    }

    fn new_task(&mut self) -> InquireResult<()> {
        let description = Text::new("description:").prompt()?;
        let options = vec!["start immediately", "put in backlog", "plan to..."];
        let option = Select::new("how to arrange this task", options)
            .without_help_message()
            .prompt()?;
        match option {
            "start immediately" => self.tasks.push(Task::new_immediate_task(&description)),
            "put in backlog" => self.tasks.push(Task::new_backlog_task(&description)),
            "plan to..." => {
                let (planned_start, planned_complete) = get_planned_pair()?;
                self.tasks.push(Task::new_planned_task(
                    &description,
                    planned_start.unwrap(),
                    planned_complete.unwrap(),
                ));
            }
            _ => unreachable!(),
        }
        self.dump_tasks();
        println!("task {} created", self.tasks.len() - 1);
        Ok(())
    }

    fn sort_tasks(&mut self) {
        self.update_status_of_all_tasks();
        self.tasks.sort_by(|ta, tb| {
            if ta.has_higher_priority_than(tb) {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        });
        self.dump_tasks();
        println!("all tasks sorted");
    }

    fn start_task(&mut self, index: usize) {
        if self.tasks.len() <= index {
            eprintln!("index out of range");
        } else {
            self.tasks[index].start();
            self.dump_tasks();
            println!("task {} started", index);
        }
    }

    fn complete_task(&mut self, index: usize) {
        if self.tasks.len() <= index {
            eprintln!("index out of range");
        } else {
            self.tasks[index].complete();
            self.dump_tasks();
            println!("task {} completed", index);
        }
    }

    fn delete_task(&mut self, index: usize) {
        if self.tasks.len() <= index {
            eprintln!("index out of range");
        } else {
            self.tasks.remove(index);
            self.dump_tasks();
            println!("task {} deleted", index);
        }
    }

    fn edit_task(&mut self, index: usize) -> InquireResult<()> {
        if self.tasks.len() <= index {
            eprintln!("index out of range");
        } else {
            let task = &mut self.tasks[index];
            let new_description = Text::new("description:")
                .with_placeholder(&task.description)
                .with_help_message("press enter if don't update description")
                .prompt()?;
            if !new_description.is_empty() {
                task.description = new_description
            }
            match get_edit_operation("planned start and complete time") {
                EditOperation::Ignore => (),
                EditOperation::Reset => (task.planned_start, task.planned_complete) = (None, None),
                EditOperation::Update => {
                    (task.planned_start, task.planned_complete) = get_planned_pair()?
                }
            }
            match get_edit_operation("actual start time") {
                EditOperation::Ignore => (),
                EditOperation::Reset => task.actual_start = None,
                EditOperation::Update => {
                    task.actual_start = Some(get_datetime_input("actual start")?)
                }
            }
            match get_edit_operation("actual complete time") {
                EditOperation::Ignore => (),
                EditOperation::Reset => task.actual_complete = None,
                EditOperation::Update => {
                    task.actual_complete = Some(get_datetime_input("actual complete")?)
                }
            }
            task.update_status();
            self.dump_tasks();
            println!("task {} edited", index);
        }
        Ok(())
    }

    fn list_tasks(&mut self, option: &ListOption) {
        self.update_status_of_all_tasks();
        self.tasks
            .iter()
            .enumerate()
            .filter(|(_, task)| task.satisfy(option))
            .for_each(|(index, task)| task.render(index, None, option.is_verbose));
    }

    fn list_tasks_with_timeline(&mut self, option: &ListOption) {
        self.update_status_of_all_tasks();
        let tasks: Vec<(usize, &Task)> = self
            .tasks
            .iter()
            .enumerate()
            .filter(|(_, task)| task.satisfy(option))
            .take(26)
            .collect();
        let (op, date) = option.date_filter;
        assert_eq!(op, DateFilterOp::Equal);
        Timeline::new(&tasks, date).draw();
        println!();
        tasks
            .iter()
            .enumerate()
            .for_each(|(timeline_index, &(index, task))| {
                task.render(
                    index,
                    Some(timeline_index_to_char(timeline_index)),
                    option.is_verbose,
                )
            });
    }

    fn update_status_of_all_tasks(&mut self) {
        self.tasks.iter_mut().for_each(|task| task.update_status());
    }

    fn dump_tasks(&mut self) {
        let mut writer = Writer::from_path(get_arenta_file_path().as_path()).unwrap();
        self.tasks.iter().for_each(|task| {
            writer
                .write_record([
                    &task.description,
                    &datetime_opt_to_string(&task.planned_start),
                    &datetime_opt_to_string(&task.planned_complete),
                    &datetime_opt_to_string(&task.actual_start),
                    &datetime_opt_to_string(&task.actual_complete),
                ])
                .unwrap()
        });
        writer.flush().unwrap();
    }
}

pub fn timeline_index_to_char(index: usize) -> char {
    char::from_u32('a' as u32 + index as u32).unwrap()
}

fn datetime_opt_to_string(datetime_opt: &Option<DateTime<Local>>) -> String {
    datetime_opt.map_or("".to_string(), |dt| dt.to_rfc3339())
}

fn datetime_opt_from_string(s: &str) -> Option<DateTime<Local>> {
    if s.is_empty() {
        None
    } else {
        Some(
            DateTime::parse_from_rfc3339(s)
                .unwrap()
                .with_timezone(&Local),
        )
    }
}

fn get_render_config() -> RenderConfig {
    RenderConfig {
        prompt_prefix: Styled::new(""),
        answered_prompt_prefix: Styled::new(""),
        ..RenderConfig::default()
    }
}

fn get_datetime_input(hint: &str) -> InquireResult<DateTime<Local>> {
    let date = DateSelect::new(&format!("{} date:", hint))
        .with_help_message("select a date")
        .prompt()?;
    let time = CustomType::<NaiveTime>::new(&format!("{} time:", hint))
        .with_parser(&|time| NaiveTime::parse_from_str(time, "%H:%M").map_err(|_| ()))
        .with_formatter(&|time| time.format("%H:%M").to_string())
        .with_error_message("please type a valid time.")
        .with_help_message("time in %H:%M format")
        .prompt()?;
    let datetime = NaiveDateTime::new(date, time);
    Ok(Local.from_local_datetime(&datetime).unwrap())
}

type PlannedPairResult = InquireResult<(Option<DateTime<Local>>, Option<DateTime<Local>>)>;
fn get_planned_pair() -> PlannedPairResult {
    let start_dt = get_datetime_input("planned start")?;
    let duration = CustomType::<usize>::new("planned time to take (in minutes):").prompt()?;
    let complete_dt = start_dt + Duration::minutes(duration as i64);
    Ok((Some(start_dt), Some(complete_dt)))
}

enum EditOperation {
    Ignore,
    Reset,
    Update,
}

fn get_edit_operation(hint: &str) -> EditOperation {
    let options = vec!["don't update", "reset", "update to..."];
    let option = Select::new(&format!("update {}?", hint), options)
        .without_help_message()
        .prompt()
        .unwrap();
    match option {
        "don't update" => EditOperation::Ignore,
        "reset" => EditOperation::Reset,
        "update to..." => EditOperation::Update,
        _ => unreachable!(),
    }
}
