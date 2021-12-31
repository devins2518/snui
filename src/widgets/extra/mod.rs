pub mod switch;

use std::f32::consts::{PI, FRAC_PI_2};

// A simple easer with a few curves
pub enum Curve {
    Quadratic,
    Linear,
    Sinus,
}

pub enum Start {
    Min,
    Middle,
    Max,
}

pub struct Easer {
    cursor: f32,
    max: f32,
    time: u32,
    frame_time: u32,
    curve: Curve,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Trend {
    Positive,
    Neutral,
    Negative
}

impl Iterator for Easer {
    type Item = f32;
    fn next(&mut self) -> Option<Self::Item> {
        let position;
        if self.time == 0 {
            return None;
        }
        let frame = self.time / self.frame_time.max(1);
        match self.curve {
            Curve::Sinus => {
                self.cursor += PI / frame as f32;
                if self.cursor > PI {
                    position = self.max * (PI).sin().abs();
                    self.time = 0;
                } else {
                    position = self.max * (self.cursor).sin().abs();
                }
            }
            Curve::Linear => {
                self.cursor += self.max / frame as f32;
                if self.cursor > self.max {
                    position = self.max;
                    self.time = 0;
                } else {
                    position = self.cursor;
                }
            }
            Curve::Quadratic => {
                let b = self.max;
                let h = b.sqrt();
                self.cursor += h * 2. / frame as f32;
                if self.cursor > 2. * h {
                    position = self.max - (2. * h - h).powi(2);
                    self.time = 0;
                } else {
                    position = self.max - (self.cursor - h).powi(2);
                }
            }
        }
        Some(position)
    }
}

impl Easer {
    pub fn new(start: Start, max: f32, time: u32, curve: Curve) -> Self {
        let cursor = match start {
            Start::Max => match curve {
                Curve::Linear => max,
                Curve::Quadratic => max.sqrt(),
                Curve::Sinus => FRAC_PI_2
            }
            Start::Min => 0.,
            Start::Middle => match curve {
                Curve::Linear => max / 2.,
                Curve::Quadratic => max.sqrt(),
                Curve::Sinus => FRAC_PI_2
            }
        };
        Easer {
            cursor,
            max,
            frame_time: 1,
            time,
            curve,
        }
    }
    pub fn set_max(&mut self, max: f32) {
        self.max = max;
    }
    pub fn frame_time(&mut self, frame_time: u32) {
        self.frame_time = frame_time;
    }
    pub fn trend(&self) -> Trend {
        match self.curve {
            Curve::Sinus => {
                if self.cursor > 0. && self.cursor < FRAC_PI_2 {
                    Trend::Positive
                } else if self.cursor > FRAC_PI_2 && self.cursor < PI {
                    Trend::Negative
                } else {
                    Trend::Neutral
                }
            }
            Curve::Linear => {
                Trend::Positive
            }
            Curve::Quadratic => {
                let b = self.max;
                let h = b.sqrt();
                if self.cursor > 0. && self.cursor < h {
                    return Trend::Positive
                } else if self.cursor > h && self.cursor < 2. * h {
                    Trend::Negative
                } else {
                    Trend::Neutral
                }
            }
        }
    }
    pub fn position(&self) -> f32 {
        match self.curve {
            Curve::Sinus => {
                self.max * (self.cursor).sin().abs()
            }
            Curve::Linear => {
                self.cursor
            }
            Curve::Quadratic => {
                let b = self.max;
                let h = b.sqrt();
                self.max - (2. * h - h).powi(2)
            }
        }
    }
    pub fn reset(&mut self, time: u32) {
        self.time = time;
        self.frame_time = 10;
        self.cursor = 0.;
    }
}
