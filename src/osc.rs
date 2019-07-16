
use std::f64::consts::PI;

const TWO_PI: f64 = std::f64::consts::PI * 2.0;

#[derive(Copy, Clone)]
pub enum Shape { Sine, Saw, Square, Triangle }

#[derive(Copy, Clone)]
pub struct Osc {
    pub shape: Shape,
    frequency: f64,
    phase: f64,
    sample_rate: f64,
    phase_increment: f64,
    pub is_muted: bool,
}

impl Osc {
    pub fn new(sample_rate: f64) -> Self {
        let mut osc = Osc {
            is_muted: false,
            shape: Shape::Square,
            frequency: 880.0 * 2.0,
            phase: 0.0,
            sample_rate,
            phase_increment: 0.0,
        };
        osc.update_phase_increment();
        osc
    }

    fn update_phase_increment(&mut self) {
        self.phase_increment = self.frequency * 2.0 * std::f64::consts::PI / self.sample_rate;
    }

    pub fn frequency(&mut self, frequency: f64) {
        self.frequency = frequency;
        self.update_phase_increment();
    }

    pub fn sample_rate(&mut self, sample_rate: f64) {
        self.sample_rate = sample_rate;
        self.update_phase_increment();
    }

    pub fn next(&mut self) -> f64 {
        if self.is_muted { return 0.0; }

        let value = match self.shape {
            Shape::Sine => self.phase.sin(),
            Shape::Saw => 1.0 - (2.0 * self.phase / TWO_PI),
            Shape::Square => if self.phase <= PI { 1.0 } else { -1.0 }
            Shape::Triangle => 2.0 * (((2.0 * self.phase / TWO_PI) - 1.0).abs() - 0.5)
        };

        self.phase += self.phase_increment;
        while self.phase > TWO_PI {
            self.phase -= TWO_PI;
        }
        value
    }
}