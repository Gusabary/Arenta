use chrono::offset::Local;
use chrono::{DateTime, Duration, NaiveDate};
use colored::{Color, Colorize};

use crate::command::{DateFilterOp, ListOption};

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum TaskStatus {
    Backlog,
    Planned,
    Overdue,
    Ongoing,
    Complete,
}

#[derive(Debug)]
pub struct Task {
    pub description: String,
    pub planned_start: Option<DateTime<Local>>,
    pub planned_complete: Option<DateTime<Local>>,
    pub actual_start: Option<DateTime<Local>>,
    pub actual_complete: Option<DateTime<Local>>,
    pub status: TaskStatus,
    pub is_deleted: bool,
}

impl Task {
    pub fn new_immediate_task(description: &str) -> Self {
        Task {
            description: description.to_string(),
            planned_start: None,
            planned_complete: None,
            actual_start: Some(Local::now()),
            actual_complete: None,
            status: TaskStatus::Ongoing,
            is_deleted: false,
        }
    }

    pub fn new_planned_task(
        description: &str,
        start: DateTime<Local>,
        end: DateTime<Local>,
    ) -> Self {
        Task {
            description: description.to_string(),
            planned_start: Some(start),
            planned_complete: Some(end),
            actual_start: None,
            actual_complete: None,
            status: if Local::now() > start {
                TaskStatus::Overdue
            } else {
                TaskStatus::Planned
            },
            is_deleted: false,
        }
    }

    pub fn new_backlog_task(description: &str) -> Self {
        Task {
            description: description.to_string(),
            planned_start: None,
            planned_complete: None,
            actual_start: None,
            actual_complete: None,
            status: TaskStatus::Backlog,
            is_deleted: false,
        }
    }

    pub fn start(&mut self) {
        self.actual_start = Some(Local::now());
        self.status = TaskStatus::Ongoing;
    }

    pub fn complete(&mut self) {
        self.actual_complete = Some(Local::now());
        self.status = TaskStatus::Complete;
    }

    pub fn delete(&mut self) {
        self.is_deleted = true;
    }

    pub fn update_status(&mut self) {
        self.status = {
            let now = Local::now();
            if self.actual_complete.map(|dt| dt < now).unwrap_or(false) {
                TaskStatus::Complete
            } else if self.actual_start.map(|dt| dt < now).unwrap_or(false) {
                TaskStatus::Ongoing
            } else if self.planned_start.map(|dt| dt < now).unwrap_or(false) {
                TaskStatus::Overdue
            } else if self.planned_start.is_some() {
                TaskStatus::Planned
            } else {
                TaskStatus::Backlog
            }
        };
    }

    pub fn satisfy(&self, option: &ListOption) -> bool {
        match self.status {
            TaskStatus::Backlog => option.include_backlog,
            _ => {
                let (op, date) = &option.date_filter;
                compare_date(&self.planned_start, *op, date)
                    || compare_date(&self.planned_complete, *op, date)
                    || compare_date(&self.actual_start, *op, date)
                    || compare_date(&self.actual_complete, *op, date)
            }
        }
    }

    pub fn has_higher_priority_than(&self, task: &Task) -> bool {
        match self.status {
            TaskStatus::Overdue => {
                if task.status == TaskStatus::Overdue {
                    self.planned_start.unwrap() < task.planned_start.unwrap()
                } else {
                    true
                }
            }
            TaskStatus::Ongoing => {
                if task.status == TaskStatus::Ongoing {
                    self.actual_start.unwrap() > task.actual_start.unwrap()
                } else {
                    task.status != TaskStatus::Overdue
                }
            }
            TaskStatus::Planned => {
                if task.status == TaskStatus::Planned {
                    self.planned_start.unwrap() < task.planned_start.unwrap()
                } else {
                    task.status == TaskStatus::Complete || task.status == TaskStatus::Backlog
                }
            }
            TaskStatus::Complete => {
                if task.status == TaskStatus::Complete {
                    self.actual_complete.unwrap() > task.actual_complete.unwrap()
                } else {
                    task.status == TaskStatus::Backlog
                }
            }
            TaskStatus::Backlog => false,
        }
    }

    pub fn render(&self, index: usize, timeline_index: Option<char>, is_verbose: bool) {
        if let Some(timeline_index) = timeline_index {
            print!("{}({}). ", index, timeline_index);
        } else {
            print!("{}. ", index);
        }

        if self.is_deleted {
            println!(
                "{}",
                "(deleted)".color(Color::TrueColor {
                    r: 100,
                    g: 100,
                    b: 100
                })
            );
            return;
        }

        if is_verbose {
            self.render_time_verbose();
        } else {
            self.render_time_simple();
        }
        println!("{}", self.description.bold());
    }

    pub fn render_time_simple(&self) {
        print!("{}  ", self.get_render_status_string());
    }

    pub fn render_time_verbose(&self) {
        print!("{: <56}", self.get_render_status_string());
        print!("{}", self.get_render_status_padding());
        fn datetime_opt_to_str(datetime_opt: &Option<DateTime<Local>>) -> String {
            match datetime_opt {
                Some(dt) => dt.format("%F %R").to_string(),
                None => "unset".to_string(),
            }
        }
        print!("{: <18}", datetime_opt_to_str(&self.planned_start));
        print!("{: <18}", datetime_opt_to_str(&self.planned_complete));
        print!("{: <18}", datetime_opt_to_str(&self.actual_start));
        print!("{: <18}", datetime_opt_to_str(&self.actual_complete));
    }

    fn get_render_status_string(&self) -> String {
        match self.status {
            TaskStatus::Backlog => format!("in {}", "backlog".color(self.color_of_status())),
            TaskStatus::Planned => {
                let gap = get_duration(&Local::now(), &self.planned_start.unwrap());
                format!(
                    "{} to start in {} minutes",
                    "planned".color(self.color_of_status()),
                    gap.num_minutes()
                )
            }
            TaskStatus::Overdue => {
                let gap = get_duration(&self.planned_start.unwrap(), &Local::now());
                format!(
                    "{} minutes {}",
                    gap.num_minutes(),
                    "overdue".color(self.color_of_status())
                )
            }
            TaskStatus::Ongoing => {
                let gap = get_duration(&self.actual_start.unwrap(), &Local::now());
                format!(
                    "{} for {} minutes",
                    "ongoing".color(self.color_of_status()),
                    gap.num_minutes()
                )
            }
            TaskStatus::Complete => {
                let gap = get_duration(&self.actual_complete.unwrap(), &Local::now());
                format!(
                    "{} {} minutes ago",
                    "complete".color(self.color_of_status()),
                    gap.num_minutes()
                )
            }
        }
    }

    fn get_render_status_padding(&self) -> String {
        match self.status {
            TaskStatus::Backlog | TaskStatus::Overdue | TaskStatus::Ongoing => "  ".to_string(),
            TaskStatus::Planned => " ".to_string(),
            TaskStatus::Complete => "".to_string(),
        }
    }

    pub fn color_of_status(&self) -> Color {
        const COLOR_GREY: Color = Color::TrueColor {
            r: 128,
            g: 128,
            b: 128,
        };
        const COLOR_CYAN: Color = Color::TrueColor {
            r: 51,
            g: 255,
            b: 255,
        };
        const COLOR_RED: Color = Color::TrueColor {
            r: 255,
            g: 102,
            b: 102,
        };
        const COLOR_YELLOW: Color = Color::TrueColor {
            r: 255,
            g: 255,
            b: 102,
        };
        const COLOR_GREEN: Color = Color::TrueColor {
            r: 51,
            g: 255,
            b: 51,
        };
        match self.status {
            TaskStatus::Backlog => COLOR_GREY,
            TaskStatus::Planned => COLOR_CYAN,
            TaskStatus::Overdue => COLOR_RED,
            TaskStatus::Ongoing => COLOR_YELLOW,
            TaskStatus::Complete => COLOR_GREEN,
        }
    }
}

fn compare_date(self_dt: &Option<DateTime<Local>>, op: DateFilterOp, date: &NaiveDate) -> bool {
    self_dt.is_some()
        && match op {
            DateFilterOp::Earlier => self_dt.unwrap().date_naive() < *date,
            DateFilterOp::EarlierEqual => self_dt.unwrap().date_naive() <= *date,
            DateFilterOp::Equal => self_dt.unwrap().date_naive() == *date,
            DateFilterOp::Later => self_dt.unwrap().date_naive() > *date,
            DateFilterOp::LaterEqual => self_dt.unwrap().date_naive() >= *date,
        }
}

fn get_duration(t0: &DateTime<Local>, t1: &DateTime<Local>) -> Duration {
    assert!(*t1 > *t0);
    *t1 - *t0
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::naive::Days;

    #[test]
    fn test_new_immediate_task() {
        let task = Task::new_immediate_task("immediate task");
        assert_eq!(task.description, "immediate task");
        assert!(task.planned_start.is_none());
        assert!(task.planned_complete.is_none());
        assert!(task.actual_start.is_some());
        assert!(task.actual_complete.is_none());
        assert_eq!(task.status, TaskStatus::Ongoing);
    }

    #[test]
    fn test_new_planned_task() {
        let format = "%Y-%m-%d %H:%M:%S%z";
        let future_start = DateTime::parse_from_str("2099-01-01 00:00:00+0000", format)
            .unwrap()
            .with_timezone(&Local);
        let future_end = DateTime::parse_from_str("2099-01-01 01:00:00+0000", format)
            .unwrap()
            .with_timezone(&Local);
        let past_start = DateTime::parse_from_str("1999-01-01 00:00:00+0000", format)
            .unwrap()
            .with_timezone(&Local);
        let past_end = DateTime::parse_from_str("1999-01-01 01:00:00+0000", format)
            .unwrap()
            .with_timezone(&Local);

        let task = Task::new_planned_task("planned task future", future_start, future_end);
        assert_eq!(task.description, "planned task future");
        assert_eq!(task.planned_start.unwrap(), future_start);
        assert_eq!(task.planned_complete.unwrap(), future_end);
        assert!(task.actual_start.is_none());
        assert!(task.actual_complete.is_none());
        assert_eq!(task.status, TaskStatus::Planned);

        let task = Task::new_planned_task("planned task past", past_start, past_end);
        assert_eq!(task.description, "planned task past");
        assert_eq!(task.planned_start.unwrap(), past_start);
        assert_eq!(task.planned_complete.unwrap(), past_end);
        assert!(task.actual_start.is_none());
        assert!(task.actual_complete.is_none());
        assert_eq!(task.status, TaskStatus::Overdue);
    }

    fn task_template() -> Task {
        Task {
            description: "".to_string(),
            planned_start: None,
            planned_complete: None,
            actual_start: None,
            actual_complete: None,
            status: TaskStatus::Planned,
            is_deleted: false,
        }
    }

    #[test]
    fn test_render_simple() {
        let task = Task {
            description: "planned but no schedule".to_string(),
            status: TaskStatus::Backlog,
            ..task_template()
        };
        task.render(0, None, false);

        let task = Task {
            description: "planned but no schedule".to_string(),
            planned_start: Local::now().checked_add_days(Days::new(1)),
            ..task_template()
        };
        task.render(1, None, false);

        let task = Task {
            description: "overdue".to_string(),
            planned_start: Local::now().checked_sub_days(Days::new(1)),
            status: TaskStatus::Overdue,
            ..task_template()
        };
        task.render(2, None, false);

        let task = Task {
            description: "ongoing".to_string(),
            actual_start: Local::now().checked_sub_days(Days::new(1)),
            status: TaskStatus::Ongoing,
            ..task_template()
        };
        task.render(3, None, false);

        let task = Task {
            description: "complete".to_string(),
            actual_complete: Local::now().checked_sub_days(Days::new(1)),
            status: TaskStatus::Complete,
            ..task_template()
        };
        task.render(4, None, false);
    }

    #[test]
    fn test_compare_date() {
        let dt = Some(
            DateTime::parse_from_str("2023-01-26 09:00:00+0800", "%F %H:%M:%S%z")
                .unwrap()
                .with_timezone(&Local),
        );
        let yesterday = NaiveDate::parse_from_str("2023-01-25", "%F").unwrap();
        let today = NaiveDate::parse_from_str("2023-01-26", "%F").unwrap();
        let tomorrow = NaiveDate::parse_from_str("2023-01-27", "%F").unwrap();
        assert!(!compare_date(&dt, DateFilterOp::Earlier, &today));
        assert!(compare_date(&dt, DateFilterOp::Earlier, &tomorrow));
        assert!(compare_date(&dt, DateFilterOp::EarlierEqual, &today));
        assert!(compare_date(&dt, DateFilterOp::EarlierEqual, &tomorrow));
        assert!(compare_date(&dt, DateFilterOp::Equal, &today));
        assert!(!compare_date(&dt, DateFilterOp::Equal, &tomorrow));
        assert!(compare_date(&dt, DateFilterOp::Later, &yesterday));
        assert!(!compare_date(&dt, DateFilterOp::Later, &today));
        assert!(compare_date(&dt, DateFilterOp::LaterEqual, &yesterday));
        assert!(compare_date(&dt, DateFilterOp::LaterEqual, &today));
    }

    #[test]
    #[cfg_attr(target_os = "windows", ignore)]
    // on windows this test case can panic at
    // 'SystemTimeToFileTime failed with: The parameter is incorrect. (os error 87)'
    // don't know why.. let's ignore it for now
    fn test_has_higher_priority_than() {
        fn overdue_task(gap: i64) -> Task {
            Task {
                status: TaskStatus::Overdue,
                planned_start: Some(Local::now() + Duration::minutes(gap)),
                ..task_template()
            }
        }
        fn ongoing_task(gap: i64) -> Task {
            Task {
                status: TaskStatus::Ongoing,
                actual_start: Some(Local::now() + Duration::minutes(gap)),
                ..task_template()
            }
        }
        fn planned_task(gap: i64) -> Task {
            Task {
                status: TaskStatus::Planned,
                planned_start: Some(Local::now() + Duration::minutes(gap)),
                ..task_template()
            }
        }
        fn done_task(gap: i64) -> Task {
            Task {
                status: TaskStatus::Complete,
                actual_complete: Some(Local::now() + Duration::minutes(gap)),
                ..task_template()
            }
        }
        fn backlog_task() -> Task {
            Task {
                status: TaskStatus::Backlog,
                ..task_template()
            }
        }
        assert!(overdue_task(-2).has_higher_priority_than(&overdue_task(-1)));
        assert!(ongoing_task(-1).has_higher_priority_than(&ongoing_task(-2)));
        assert!(planned_task(1).has_higher_priority_than(&planned_task(2)));
        assert!(done_task(-1).has_higher_priority_than(&done_task(-2)));
        assert!(overdue_task(-2).has_higher_priority_than(&ongoing_task(-1)));
        assert!(ongoing_task(-1).has_higher_priority_than(&planned_task(2)));
        assert!(planned_task(1).has_higher_priority_than(&done_task(-2)));
        assert!(done_task(-1).has_higher_priority_than(&backlog_task()));
    }
}
