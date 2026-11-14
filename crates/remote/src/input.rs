use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
pub enum Input {
    Down { x: f64, y: f64 },
    Move { x: f64, y: f64 },
    Up { x: f64, y: f64 },
    Wheel { x: f64, y: f64, delta: f64 },
    Text { text: String },
    Key { key: String },
    Cancel,
}

impl Input {
    pub fn valid(&self) -> bool {
        let point =
            |x: f64, y: f64| x.is_finite() && y.is_finite() && (0.0..=1.0).contains(&x) && (0.0..=1.0).contains(&y);
        match self {
            Self::Down { x, y } | Self::Move { x, y } | Self::Up { x, y } => point(*x, *y),
            Self::Wheel { x, y, delta } => point(*x, *y) && delta.is_finite() && delta.abs() <= 1200.0,
            Self::Text { text } => text.len() <= 1024 && !text.contains('\0'),
            Self::Key { key } => matches!(key.as_str(), "backspace" | "enter" | "escape"),
            Self::Cancel => true,
        }
    }
}

#[derive(Default)]
pub struct InputQueue {
    queue: std::collections::VecDeque<Input>,
    blocked: bool,
}
impl InputQueue {
    pub fn push(&mut self, input: Input) {
        if !input.valid() {
            return;
        }
        if matches!(input, Input::Cancel) {
            self.queue.clear();
            self.blocked = false;
        } else if self.blocked {
            if matches!(input, Input::Up { .. }) {
                self.blocked = false;
                self.queue.push_back(Input::Cancel);
            }
            return;
        }
        if self.queue.len() >= 128 {
            self.queue.clear();
            self.queue.push_back(Input::Cancel);
            self.blocked = true;
            return;
        }
        self.queue.push_back(input);
    }
    pub fn take(&mut self) -> Vec<Input> {
        self.queue.drain(..).collect()
    }
}
