pub mod switch;

use std::f32::consts::{FRAC_PI_2, PI};

pub trait Easer: Iterator<Item = f32> {
    fn steps(&self) -> usize;
    /// Start and coordinates is the position in the animation from 0 to 1
    fn new(start: f32, end: f32, amplitude: f32) -> Self;
    fn set_amplitude(&mut self, amplitude: f32);
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Sinus {
    steps: f32,
    amplitude: f32,
    // On the X axis
    start: f32,
    end: f32,
    position: f32,
}

impl Iterator for Sinus {
    type Item = f32;
    fn next(&mut self) -> Option<Self::Item> {
        let v = self.amplitude * (self.position).sin().abs();
        if self.position <= self.end {
            self.position += PI / self.steps;
            Some(v.floor())
        } else {
            self.position = self.start;
            return None;
        }
    }
}

impl Easer for Sinus {
    fn steps(&self) -> usize {
        self.steps as usize
    }
    fn set_amplitude(&mut self, amplitude: f32) {
        self.amplitude = amplitude;
    }
    fn new(start: f32, end: f32, amplitude: f32) -> Self {
        Self {
            amplitude,
            steps: 1000.,
            start: start * PI,
            end: end * PI,
            position: start * PI,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Quadratic {
    steps: f32,
    amplitude: f32,
    end: f32,
    start: f32,
    position: f32,
}

impl Iterator for Quadratic {
    type Item = f32;
    fn next(&mut self) -> Option<Self::Item> {
        let h = self.amplitude.sqrt();
        let v = self.amplitude - (self.position - h).powi(2);
        if self.position < self.end {
            self.position += h * 2. / self.steps;
            Some(v.floor())
        } else {
            self.position = self.start;
            None
        }
    }
}

impl Easer for Quadratic {
    fn new(start: f32, end: f32, amplitude: f32) -> Self {
        let h = 2. * amplitude.sqrt();
        Self {
            amplitude,
            steps: 1000.,
            start: start * h,
            end: end * h,
            position: start,
        }
    }
    fn steps(&self) -> usize {
        self.steps as usize
    }
    fn set_amplitude(&mut self, amplitude: f32) {
        self.amplitude = amplitude;
    }
}
