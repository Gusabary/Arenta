use chrono::offset::Local;
use chrono::{DateTime, Duration};
use termion::color;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum TaskStatus {
    Planned,
    Overdue,
    Ongoing,
    Done,
}

#[derive(Debug)]
pub struct Task {
    pub description: String,
    pub planned_start: Option<DateTime<Local>>,
    pub planned_complete: Option<DateTime<Local>>,
    pub actual_start: Option<DateTime<Local>>,
    pub actual_complete: Option<DateTime<Local>>,
    pub status: TaskStatus,
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
        }
    }

    pub fn start(&mut self) {
        self.actual_start = Some(Local::now());
        self.status = TaskStatus::Ongoing;
    }

    pub fn complete(&mut self) {
        self.actual_complete = Some(Local::now());
        self.status = TaskStatus::Done;
    }

    pub fn update_status(&mut self) {
        self.status = {
            let now = Local::now();
            if self.actual_complete.map(|dt| dt < now).unwrap_or(false) {
                TaskStatus::Done
            } else if self.actual_start.map(|dt| dt < now).unwrap_or(false) {
                TaskStatus::Ongoing
            } else if self.planned_start.map(|dt| dt < now).unwrap_or(false) {
                TaskStatus::Overdue
            } else {
                TaskStatus::Planned
            }
        };
    }

    pub fn is_in_recent_n_days(&self, n: usize) -> bool {
        let since = Local::now() - Duration::days(n as i64);
        let since = since.date_naive().and_hms_opt(0, 0, 0).unwrap();
        self.actual_complete
            .map(|dt| dt.naive_local() > since)
            .unwrap_or(true)
    }

    pub fn render_simple(&self, index: usize) {
        print!(
            "{}: {}{}{}  - ",
            index,
            termion::style::Bold,
            self.description,
            termion::style::Reset
        );
        match self.status {
            TaskStatus::Planned => self.render_planned(),
            TaskStatus::Overdue => self.render_overdue(),
            TaskStatus::Ongoing => self.render_ongoing(),
            TaskStatus::Done => self.render_done(),
        }
    }

    fn render_planned(&self) {
        if self.planned_start.is_none() {
            println!(
                "{}planned{} but not in schedule yet",
                color::Fg(color::Cyan),
                color::Fg(color::Reset)
            );
        } else {
            let gap = get_duration(&Local::now(), &self.planned_start.unwrap());
            println!(
                "{}planned{} to start in {} minutes",
                color::Fg(color::Cyan),
                color::Fg(color::Reset),
                gap.num_minutes()
            );
        }
    }

    fn render_overdue(&self) {
        let gap = get_duration(&self.planned_start.unwrap(), &Local::now());
        println!(
            "{} minutes {}overdue{}",
            gap.num_minutes(),
            color::Fg(color::LightRed),
            color::Fg(color::Reset)
        );
    }

    fn render_ongoing(&self) {
        let gap = get_duration(&self.actual_start.unwrap(), &Local::now());
        println!(
            "{}ongoing{} for {} minutes",
            color::Fg(color::LightYellow),
            color::Fg(color::Reset),
            gap.num_minutes()
        );
    }

    fn render_done(&self) {
        let gap = get_duration(&self.actual_complete.unwrap(), &Local::now());
        println!(
            "{}done{} {} minutes ago",
            color::Fg(color::LightGreen),
            color::Fg(color::Reset),
            gap.num_minutes()
        );
    }
}

fn get_duration(t0: &DateTime<Local>, t1: &DateTime<Local>) -> Duration {
    assert!(*t1 > *t0);
    *t1 - *t0
}

#[cfg(test)]
mod tests {
    use chrono::naive::Days;

    use super::*;

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
        }
    }

    #[test]
    fn test_render_simple() {
        let task = Task {
            description: "planned but no schedule".to_string(),
            ..task_template()
        };
        task.render_simple(0);

        let task = Task {
            description: "planned but no schedule".to_string(),
            planned_start: Local::now().checked_add_days(Days::new(1)),
            ..task_template()
        };
        task.render_simple(1);

        let task = Task {
            description: "overdue".to_string(),
            planned_start: Local::now().checked_sub_days(Days::new(1)),
            status: TaskStatus::Overdue,
            ..task_template()
        };
        task.render_simple(2);

        let task = Task {
            description: "ongoing".to_string(),
            actual_start: Local::now().checked_sub_days(Days::new(1)),
            status: TaskStatus::Ongoing,
            ..task_template()
        };
        task.render_simple(3);

        let task = Task {
            description: "done".to_string(),
            actual_complete: Local::now().checked_sub_days(Days::new(1)),
            status: TaskStatus::Done,
            ..task_template()
        };
        task.render_simple(4);
    }

    #[test]
    fn test_is_in_recent_n_days() {
        assert!(task_template().is_in_recent_n_days(5));
        let task = Task {
            actual_complete: Some(Local::now() - Duration::days(2)),
            ..task_template()
        };
        assert!(!task.is_in_recent_n_days(0));
        assert!(!task.is_in_recent_n_days(1));
        assert!(task.is_in_recent_n_days(2));
        assert!(task.is_in_recent_n_days(3));
    }

    #[test]
    fn test_A() {
        let format = "%Y-%m-%d %H:%M:%S%z";
        let now = DateTime::parse_from_str("2023-01-20 07:00:00+0000", format)
            .unwrap()
            .with_timezone(&Local);
        let x = DateTime::parse_from_str("2023-01-18 07:00:00+0000", format)
            .unwrap()
            .with_timezone(&Local);

        // println!("{:?}", now);

        let past = now - Duration::days(2);
        println!("{:?}", past);
        println!("{:?}", past.naive_local());
        let past = past.date_naive().and_hms_opt(0, 0, 0).unwrap();
        println!("{:?}", past);
        println!("{:?}", x.naive_local());
        println!("{:?}", x.naive_local() > past);

        // let gap = now.date_naive(
        // let aa = now + Duration::minutes(60);
        // // let aa = DateTime::parse_from_str("2023-01-23 16:55:00+0800", "%Y-%m-%d %H:%M:%S%z")
        // //     .unwrap()
        // //     .with_timezone(&Local);
        // // let duration = aa-now;
        // println!("{:?}", now);
        // println!("{:?}", aa);
    }
}
