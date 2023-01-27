use std::vec;

use crate::{manager::timeline_index_to_char, task::Task};
use chrono::{DateTime, Local, NaiveDate};
use colored::{Color, Colorize};

const UI_MAX_WIDTH: usize = 73;

#[derive(Clone, Debug, Copy)]
struct Pixel {
    content: char,
    color: Option<Color>,
}

impl Pixel {
    fn default() -> Self {
        Self::new(' ', None)
    }

    fn new(content: char, color: Option<Color>) -> Self {
        Pixel { content, color }
    }

    fn set_if_empty(&mut self, pixel: Pixel) {
        if self.is_empty() {
            *self = pixel;
        }
    }

    fn is_empty(&self) -> bool {
        self.content == ' '
    }

    fn render(&self) {
        if let Some(color) = self.color {
            print!("{}", self.content.to_string().color(color));
        } else {
            print!("{}", self.content);
        }
    }
}

pub struct Timeline<'a> {
    tasks: &'a Vec<(usize, &'a Task)>,
    canvas: Vec<Vec<Pixel>>,
    date: NaiveDate,
    pos_of_now: Option<i64>,
}

impl<'a> Timeline<'a> {
    pub fn new(tasks: &'a Vec<(usize, &'a Task)>, date: NaiveDate) -> Self {
        assert!(tasks.len() <= 26);
        let pos_of_now = if Local::now().date_naive() == date {
            Some(get_pos_in_row(&Local::now()))
        } else {
            None
        };
        Timeline {
            tasks,
            canvas: vec![],
            date,
            pos_of_now,
        }
    }

    pub fn draw(&mut self) {
        self.tasks
            .iter()
            .enumerate()
            .for_each(|(timeline_index, &(_, task))| {
                self.populate_task(task, timeline_index_to_char(timeline_index))
            });
        self.populate_scale_line();
        self.populate_now_cursor();
        println!("{}", self.date.format("%F").to_string().bold().underline());
        self.canvas.iter().for_each(|row| {
            row.iter().for_each(|p| p.render());
            println!();
        });
    }

    fn populate_scale_line(&mut self) {
        self.canvas.insert(
            0,
            "8     9     10    11    12    13    14    15    16    17    18    19    20"
                .chars()
                .map(|content| Pixel::new(content, None))
                .collect(),
        );
        self.canvas.insert(
            1,
            "|-----|-----|-----|-----|-----|-----|-----|-----|-----|-----|-----|-----|"
                .chars()
                .map(|content| Pixel::new(content, None))
                .collect(),
        );
        self.canvas.insert(
            self.canvas.len(),
            "|-----|-----|-----|-----|-----|-----|-----|-----|-----|-----|-----|-----|"
                .chars()
                .map(|content| Pixel::new(content, None))
                .collect(),
        );
        self.canvas.insert(
            self.canvas.len(),
            "8     9     10    11    12    13    14    15    16    17    18    19    20"
                .chars()
                .map(|content| Pixel::new(content, None))
                .collect(),
        );
    }

    fn populate_now_cursor(&mut self) {
        if self.pos_of_now.is_none() {
            return;
        }
        let pos = self.pos_of_now.unwrap().clamp(0, UI_MAX_WIDTH as i64 - 1) as usize;
        let bottom = self.canvas.len() - 2;
        self.canvas[1][pos] = Pixel::new('v', Some(Color::Red));
        self.canvas[bottom][pos] = Pixel::new('^', Some(Color::Red));
        self.canvas[2..=bottom]
            .iter_mut()
            .for_each(|row| row[pos].set_if_empty(Pixel::new('|', Some(Color::Red))));
    }

    fn populate_task(&mut self, task: &Task, index: char) {
        if task.is_deleted {
            return;
        }
        if self.date_includes(&task.planned_start) && self.date_includes(&task.planned_complete) {
            let start_pos = get_pos_in_row(&task.planned_start.unwrap());
            let end_pos = get_pos_in_row(&task.planned_complete.unwrap());
            self.populate_index_and_line(
                start_pos,
                end_pos,
                index,
                Pixel::new('-', Some(task.color_of_status())),
            );
        }
        if self.date_includes(&task.actual_start) {
            let start_pos = get_pos_in_row(&task.actual_start.unwrap());
            let end_pos = task
                .actual_complete
                .map_or(self.pos_of_now.unwrap_or(UI_MAX_WIDTH as i64 - 1), |dt| {
                    get_pos_in_row(&dt)
                });
            self.populate_index_and_line(
                start_pos,
                end_pos,
                index,
                Pixel::new('=', Some(task.color_of_status())),
            );
        } else if self.date_includes(&task.actual_complete) {
            let end_pos = get_pos_in_row(&task.actual_complete.unwrap());
            self.populate_index_and_line(
                1,
                end_pos,
                index,
                Pixel::new('=', Some(task.color_of_status())),
            );
        }
    }

    fn date_includes(&self, datetime: &Option<DateTime<Local>>) -> bool {
        datetime.is_some() && datetime.unwrap().date_naive() == self.date
    }

    fn populate_index_and_line(&mut self, start_pos: i64, end_pos: i64, index: char, pixel: Pixel) {
        let start_pos = start_pos.clamp(1, UI_MAX_WIDTH as i64 - 1) as usize;
        let end_pos = end_pos.clamp(1, UI_MAX_WIDTH as i64 - 1) as usize;
        let row_opt = self
            .canvas
            .iter()
            .position(|row| can_put_in_row(row, start_pos, end_pos));
        let row = row_opt.unwrap_or_else(|| self.new_row());
        self.put_in_row(row, start_pos, end_pos, pixel);
        assert!(start_pos >= 1);
        self.canvas[row][start_pos - 1] = Pixel::new(index, pixel.color);
    }

    fn new_row(&mut self) -> usize {
        self.canvas.push(vec![Pixel::default(); UI_MAX_WIDTH]);
        self.canvas.len() - 1
    }

    fn put_in_row(&mut self, row: usize, start_pos: usize, end_pos: usize, pixel: Pixel) {
        self.canvas[row].splice(start_pos..=end_pos, vec![pixel; end_pos - start_pos + 1]);
    }
}

fn get_pos_in_row(dt: &DateTime<Local>) -> i64 {
    const START_HOUR: u32 = 8;
    const TIMELINE_TICK: usize = 10;
    let start_of_day = Local::now()
        .date_naive()
        .and_hms_opt(START_HOUR, 0, 0)
        .unwrap();
    let offset = dt.naive_local() - start_of_day;
    offset.num_minutes() / TIMELINE_TICK as i64
}

fn can_put_in_row(row: &[Pixel], start_pos: usize, end_pos: usize) -> bool {
    row[start_pos - 1..=end_pos]
        .iter()
        .all(|pixel| pixel.is_empty())
}
