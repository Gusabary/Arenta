use chrono::offset::Local;
use chrono::DateTime;

#[derive(Debug, PartialEq)]
enum TaskStatus {
    Planned,
    Overdue,
    Ongoing,
    Done,
}

#[derive(Debug)]
pub struct Task {
    description: String,
    planned_start: Option<DateTime<Local>>,
    planned_end: Option<DateTime<Local>>,
    actual_start: Option<DateTime<Local>>,
    actual_end: Option<DateTime<Local>>,
    status: TaskStatus,
}

impl Task {
    pub fn new_immediate_task(description: &str) -> Self {
        Task {
            description: description.to_string(),
            planned_start: None,
            planned_end: None,
            actual_start: Some(Local::now()),
            actual_end: None,
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
            planned_end: Some(end),
            actual_start: None,
            actual_end: None,
            status: if Local::now() > start {
                TaskStatus::Overdue
            } else {
                TaskStatus::Planned
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_immediate_task() {
        let task = Task::new_immediate_task("immediate task");
        assert_eq!(task.description, "immediate task");
        assert!(task.planned_start.is_none());
        assert!(task.planned_end.is_none());
        assert!(task.actual_start.is_some());
        assert!(task.actual_end.is_none());
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
        assert_eq!(task.planned_end.unwrap(), future_end);
        assert!(task.actual_start.is_none());
        assert!(task.actual_end.is_none());
        assert_eq!(task.status, TaskStatus::Planned);

        let task = Task::new_planned_task("planned task past", past_start, past_end);
        assert_eq!(task.description, "planned task past");
        assert_eq!(task.planned_start.unwrap(), past_start);
        assert_eq!(task.planned_end.unwrap(), past_end);
        assert!(task.actual_start.is_none());
        assert!(task.actual_end.is_none());
        assert_eq!(task.status, TaskStatus::Overdue);
    }
}
