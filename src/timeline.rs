use std::vec;

use crate::task::Task;
use chrono::{DateTime, Local};
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
    tasks: &'a Vec<&'a Task>,
    ui: Vec<Vec<Pixel>>,
    pos_of_now: i64,
}

impl<'a> Timeline<'a> {
    pub fn new(tasks: &'a Vec<&'a Task>) -> Self {
        assert!(tasks.len() < 10);
        Timeline {
            tasks,
            ui: vec![],
            pos_of_now: get_pos_in_row(&Local::now()),
        }
    }

    pub fn draw(&mut self) {
        self.tasks
            .iter()
            .enumerate()
            .for_each(|(index, task)| self.populate_task(task, index));
        self.populate_scale_line();
        self.populate_now_cursor();
        self.ui.iter().for_each(|row| {
            row.iter().for_each(|p| p.render());
            println!();
        });
    }

    fn populate_scale_line(&mut self) {
        self.ui.insert(
            0,
            "8     9     10    11    12    13    14    15    16    17    18    19    20"
                .chars()
                .map(|content| Pixel::new(content, None))
                .collect(),
        );
        self.ui.insert(
            1,
            "|-----|-----|-----|-----|-----|-----|-----|-----|-----|-----|-----|-----|"
                .chars()
                .map(|content| Pixel::new(content, None))
                .collect(),
        );
        self.ui.insert(
            self.ui.len(),
            "|-----|-----|-----|-----|-----|-----|-----|-----|-----|-----|-----|-----|"
                .chars()
                .map(|content| Pixel::new(content, None))
                .collect(),
        );
        self.ui.insert(
            self.ui.len(),
            "8     9     10    11    12    13    14    15    16    17    18    19    20"
                .chars()
                .map(|content| Pixel::new(content, None))
                .collect(),
        );
    }

    fn populate_now_cursor(&mut self) {
        let pos = self.pos_of_now.clamp(0, UI_MAX_WIDTH as i64 - 1) as usize;
        let bottom = self.ui.len() - 2;
        self.ui[1][pos] = Pixel::new('v', Some(Color::Red));
        self.ui[bottom][pos] = Pixel::new('^', Some(Color::Red));
        self.ui[2..=bottom]
            .iter_mut()
            .for_each(|row| row[pos].set_if_empty(Pixel::new('|', Some(Color::Red))));
    }

    fn populate_task(&mut self, task: &Task, index: usize) {
        if task.planned_start.is_some() && task.planned_complete.is_some() {
            let start_pos = get_pos_in_row(&task.planned_start.unwrap());
            let end_pos = get_pos_in_row(&task.planned_complete.unwrap());
            self.populate_line(
                start_pos,
                end_pos,
                index,
                Pixel::new('-', Some(task.color_of_status())),
            );
        }
        if task.actual_start.is_some() {
            let start_pos = get_pos_in_row(&task.actual_start.unwrap());
            let end_pos = task
                .actual_complete
                .map_or(self.pos_of_now, |dt| get_pos_in_row(&dt));
            self.populate_line(
                start_pos,
                end_pos,
                index,
                Pixel::new('=', Some(task.color_of_status())),
            );
        }
    }

    fn populate_line(&mut self, start_pos: i64, end_pos: i64, index: usize, pixel: Pixel) {
        let start_pos = start_pos.clamp(1, UI_MAX_WIDTH as i64 - 1) as usize;
        let end_pos = end_pos.clamp(1, UI_MAX_WIDTH as i64 - 1) as usize;
        let row_opt = self
            .ui
            .iter()
            .position(|row| can_put_in_row(row, start_pos, end_pos));
        let row = row_opt.unwrap_or_else(|| self.new_row());
        self.put_in_row(row, start_pos, end_pos, pixel);
        assert!(start_pos >= 1);
        assert!(index < 10);
        let content = index.to_string().chars().next().unwrap();
        self.ui[row][start_pos - 1] = Pixel::new(content, pixel.color);
    }

    fn new_row(&mut self) -> usize {
        self.ui.push(vec![Pixel::default(); UI_MAX_WIDTH]);
        self.ui.len() - 1
    }

    fn put_in_row(&mut self, row: usize, start_pos: usize, end_pos: usize, pixel: Pixel) {
        self.ui[row].splice(start_pos..=end_pos, vec![pixel; end_pos - start_pos + 1]);
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
