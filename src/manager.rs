use crate::command::{parse_command, print_command_usage, Command, ListOption};
use crate::task::{Task, TaskStatus};
use crate::timeline::Timeline;
use chrono::{DateTime, Duration, Local, NaiveDateTime, NaiveTime, TimeZone};
use csv::{ReaderBuilder, StringRecord, Writer};
use inquire::{
    ui::{RenderConfig, Styled},
    Confirm, CustomType, DateSelect, Text,
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
        Task {
            description: record.get(0).unwrap().to_string(),
            planned_start: datetime_opt_from_string(record.get(1).unwrap()),
            planned_complete: datetime_opt_from_string(record.get(2).unwrap()),
            actual_start: datetime_opt_from_string(record.get(3).unwrap()),
            actual_complete: datetime_opt_from_string(record.get(4).unwrap()),
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
            if self.dispatch_command(&command.unwrap()) {
                break;
            }
        }
    }

    fn dispatch_command(&mut self, command: &Command) -> bool {
        match command {
            Command::Empty => return false,
            Command::Quit => return true,
            Command::Help => print_command_usage(),
            Command::New => self.new_task(),
            Command::Sort => self.sort_tasks(),
            Command::Start(index) => self.start_task(*index),
            Command::Complete(index) => self.complete_task(*index),
            Command::Delete(index) => self.delete_task(*index),
            Command::Edit(index) => self.edit_task(*index),
            Command::List(list_option) => match list_option {
                ListOption::Naive => self.list_tasks_simple(0),
                ListOption::HistoricalDays(n) => self.list_tasks_simple(*n),
                ListOption::Timeline => self.list_tasks_with_timeline(),
            },
        }
        false
    }

    fn new_task(&mut self) {
        let description = Text::new("description:").prompt().unwrap();
        let start_immediately = Confirm::new("start immediately?")
            .with_default(true)
            .with_placeholder("Yes")
            .prompt()
            .unwrap();
        if start_immediately {
            self.tasks.push(Task::new_immediate_task(&description));
        } else {
            let planned_start = get_datetime_input("planned start");
            let planned_duration = CustomType::<usize>::new("planned time to take (in minutes):")
                .prompt()
                .unwrap();
            let planned_complete = planned_start + Duration::minutes(planned_duration as i64);
            self.tasks.push(Task::new_planned_task(
                &description,
                planned_start,
                planned_complete,
            ));
        }
        self.dump_tasks();
        println!("task {} created", self.tasks.len() - 1);
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

    fn edit_task(&mut self, index: usize) {
        if self.tasks.len() <= index {
            eprintln!("index out of range");
        } else {
            let task = &mut self.tasks[index];
            let new_description = Text::new("description:")
                .with_placeholder(&task.description)
                .with_help_message("press enter if don't update description")
                .prompt()
                .unwrap();
            if !new_description.is_empty() {
                task.description = new_description
            }
            check_and_update_datetime(&mut task.planned_start, "planned start");
            check_and_update_datetime(&mut task.planned_complete, "planned complete");
            check_and_update_datetime(&mut task.actual_start, "actual start");
            check_and_update_datetime(&mut task.actual_complete, "planned complete");
            task.update_status();
            self.dump_tasks();
            println!("task {} edited", index);
        }
    }

    fn list_tasks_simple(&mut self, ndays: usize) {
        self.update_status_of_all_tasks();
        self.tasks
            .iter()
            .filter(|&task| task.is_in_recent_n_days(ndays))
            .enumerate()
            .for_each(|(index, task)| task.render_simple(index));
    }

    fn list_tasks_with_timeline(&mut self) {
        self.update_status_of_all_tasks();
        let tasks: Vec<&Task> = self
            .tasks
            .iter()
            .filter(|&task| task.is_in_recent_n_days(0))
            .take(10)
            .collect();
        Timeline::new(&tasks).draw();
        println!();
        tasks
            .iter()
            .enumerate()
            .for_each(|(index, task)| task.render_simple(index));
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

fn get_datetime_input(hint: &str) -> DateTime<Local> {
    let date = DateSelect::new(&format!("{} date:", hint))
        .with_help_message("select a date")
        .prompt()
        .unwrap();
    let time = CustomType::<NaiveTime>::new(&format!("{} time:", hint))
        .with_parser(&|time| NaiveTime::parse_from_str(time, "%H:%M").map_err(|_| ()))
        .with_formatter(&|time| time.format("%H:%M").to_string())
        .with_error_message("please type a valid time.")
        .with_help_message("time in %H:%M format")
        .prompt()
        .unwrap();
    let datetime = NaiveDateTime::new(date, time);
    Local.from_local_datetime(&datetime).unwrap()
}

fn check_and_update_datetime(datetime: &mut Option<DateTime<Local>>, hint: &str) {
    let update = Confirm::new(&format!("update {}?", hint))
        .with_default(false)
        .with_placeholder("No")
        .prompt()
        .unwrap();
    if update {
        let reset = Confirm::new(&format!("reset {}?", hint))
            .with_default(false)
            .with_placeholder("No")
            .prompt()
            .unwrap();
        if reset {
            *datetime = None
        } else {
            *datetime = Some(get_datetime_input(hint));
        }
    }
}
