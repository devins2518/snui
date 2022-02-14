//! Additional tools or widgets which aren't necessary

pub mod revealer;
pub mod switch;

#[derive(Copy, Clone, Debug, PartialEq)]
enum Course {
    Running,
    Rest,
}

use std::f32::consts::PI;

pub trait Easer: Iterator<Item = f32> {
    fn steps(&self) -> usize;
    /// Start and coordinates is the position in the animation from 0 to 1
    fn new(start: f32, end: f32, amplitude: f32) -> Self;
    fn set_amplitude(&mut self, amplitude: f32);
}

/// An easer implementing the sine function
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Sinus {
    steps: f32,
    amplitude: f32,
    start: f32,
    end: f32,
    course: Course,
    position: f32,
}

impl Iterator for Sinus {
    type Item = f32;
    fn next(&mut self) -> Option<Self::Item> {
        let v = self.amplitude * (self.position).sin().abs();
        match self.course {
            Course::Running => {
                if self.position <= self.end {
                    self.position += PI / self.steps;
                } else {
                    self.course = Course::Rest;
                }
                Some(v.round())
            }
            Course::Rest => {
                self.course = Course::Running;
                self.position = self.start;
                None
            }
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
            course: Course::Running,
            steps: 1000.,
            start: start * PI,
            end: end * PI,
            position: start * PI,
        }
    }
}

/// An easer with a quadratic cruve
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Quadratic {
    steps: f32,
    amplitude: f32,
    end: f32,
    start: f32,
    course: Course,
    position: f32,
}

impl Iterator for Quadratic {
    type Item = f32;
    fn next(&mut self) -> Option<Self::Item> {
        match self.course {
            Course::Running => {
                let h = self.amplitude.abs().sqrt();
                let v = if self.amplitude.is_sign_positive() {
                    self.amplitude - (self.position - h).powi(2)
                } else {
                    self.amplitude + (self.position - h).powi(2)
                };
                if self.position <= self.end {
                    self.position += h * 2. / self.steps;
                } else {
                    self.course = Course::Rest;
                }
                Some(v.round())
            }
            Course::Rest => {
                self.course = Course::Running;
                self.position = self.start;
                None
            }
        }
    }
}

impl Easer for Quadratic {
    fn new(start: f32, end: f32, amplitude: f32) -> Self {
        let h = 2. * amplitude.abs().sqrt();
        Self {
            amplitude,
            steps: 1000.,
            course: Course::Running,
            start: start * h,
            end: end * h,
            position: start * h,
        }
    }
    fn steps(&self) -> usize {
        self.steps as usize
    }
    fn set_amplitude(&mut self, amplitude: f32) {
        self.amplitude = amplitude;
    }
}
