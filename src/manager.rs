use crate::command::{parse_command, Command, ListOption};
use crate::task::Task;
use chrono::{DateTime, Duration, Local, NaiveDateTime, NaiveTime, TimeZone};
use inquire::{
    ui::{RenderConfig, Styled},
    Confirm, CustomType, DateSelect, Text,
};

pub struct Manager {
    tasks: Vec<Task>,
}

impl Manager {
    pub fn new() -> Self {
        Manager { tasks: vec![] }
    }

    pub fn start_loop(&mut self) {
        inquire::set_global_render_config(get_render_config());
        loop {
            let command = Text::new("arenta>").prompt().unwrap();
            let command = parse_command(&command);
            if command.is_none() {
                print_command_usage();
                continue;
            }
            match command.unwrap() {
                Command::Empty => continue,
                Command::Quit => break,
                Command::Help => print_command_usage(),
                Command::New => self.new_task(),
                Command::Start(index) => self.start_task(index),
                Command::Complete(index) => self.complete_task(index),
                Command::Delete(index) => self.delete_task(index),
                Command::Edit(index) => self.edit_task(index),
                Command::List(list_option) => match list_option {
                    ListOption::Naive => self.list_tasks_simple(0),
                    ListOption::HistoricalDays(n) => self.list_tasks_simple(n),
                    ListOption::Timeline => self.list_tasks_with_timeline(),
                },
            }
        }
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
    }

    fn start_task(&mut self, index: usize) {
        if self.tasks.len() <= index {
            eprintln!("index out of range");
        } else {
            self.tasks[index].start();
        }
    }

    fn complete_task(&mut self, index: usize) {
        if self.tasks.len() <= index {
            eprintln!("index out of range");
        } else {
            self.tasks[index].complete();
        }
    }

    fn delete_task(&mut self, index: usize) {
        if self.tasks.len() <= index {
            eprintln!("index out of range");
        } else {
            self.tasks.remove(index);
            println!("task {} removed", index);
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
        }
    }

    fn list_tasks_simple(&mut self, ndays: usize) {
        self.tasks.iter_mut().for_each(|task| task.update_status());
        self.tasks
            .iter()
            .filter(|&task| task.is_in_recent_n_days(ndays))
            .enumerate()
            .for_each(|(index, task)| task.render_simple(index));
    }

    fn list_tasks_with_timeline(&mut self) {}
}

fn get_render_config() -> RenderConfig {
    RenderConfig {
        prompt_prefix: Styled::new(""),
        answered_prompt_prefix: Styled::new(""),
        ..RenderConfig::default()
    }
}

fn print_command_usage() {
    println!("TODO: help");
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
