pub mod switch;

use std::f32::consts::{FRAC_PI_2, PI};

pub struct Sinus {
    steps: f32,
    amplitude: f32,
    // On the X axis
    start: f32,
    end: f32,
    position: f32
}

impl Iterator for Sinus {
    type Item = f32;
    fn next(&mut self) -> Option<Self::Item> {
        let v = self.amplitude * (self.position).sin().abs();
        if self.position < self.end {
            self.position += PI / self.steps;
            Some(v)
        } else {
            self.position = self.start;
            return None
        }
    }
}

impl Sinus {
    /// Start and end in radian
    pub fn new(start: f32, end: f32, amplitude: f32) -> Self {
        Self {
            amplitude,
            steps: 1000.,
            start,
            end,
            position: start
        }
    }
    pub fn steps(&self) -> usize {
        self.steps as usize
    }
    fn set_amplitude(&mut self, amplitude: f32) {
        self.amplitude = amplitude;
    }
}

pub struct Quadratic {
    steps: f32,
    amplitude: f32,
    end: f32,
    start: f32,
    position: f32
}

impl Iterator for Quadratic {
    type Item = f32;
    fn next(&mut self) -> Option<Self::Item> {
        let h = self.amplitude.sqrt();
        let v = self.amplitude - (self.position - h).powi(2).round();
        if (self.position / h / 2.) < self.end {
            self.position += h * 2. / self.steps;
            Some(v)
        } else {
            self.position = self.start;
            None
        }
    }
}

impl Quadratic {
    /// Start and coordinates is the position in the animation from 0 to 1
    pub fn new(start: f32, end: f32, amplitude: f32) -> Self {
        Self {
            amplitude,
            steps: 1000.,
            start,
            end,
            position: start
        }
    }
    pub fn steps(&self) -> usize {
        self.steps as usize
    }
    fn set_amplitude(&mut self, amplitude: f32) {
        self.amplitude = amplitude;
    }
}
