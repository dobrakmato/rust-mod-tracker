use std::f64::consts::PI;
use rand::Rng;

const TWO_PI: f64 = std::f64::consts::PI * 2.0;

#[derive(Copy, Clone, Rand)]
pub enum Shape { Sine, Saw, Square, Triangle, Noise }

#[derive(Copy, Clone)]
pub struct Osc {
    pub shape: Shape,
    frequency: f64,
    phase: f64,
    pitch_mod: f64,
    sample_rate: f64,
    phase_increment: f64,
    last_output: f64,
}

impl Osc {
    pub fn new(sample_rate: f64) -> Self {
        let mut osc = Osc {
            shape: Shape::Square,
            frequency: 880.0 * 2.0,
            pitch_mod: 0.0,
            phase: 0.0,
            sample_rate,
            phase_increment: 0.0,
            last_output: 0.0,
        };
        osc.update_phase_increment();
        osc
    }

    fn update_phase_increment(&mut self) {
        let modulated_freq = (self.pitch_mod.abs() * 14.0).powf(2.0) - 1.0;
        if self.pitch_mod < 0.0 { let freq = -modulated_freq; }

        let actual_freq = (self.frequency + modulated_freq).clamp(0.0, self.sample_rate / 2.0);

        self.phase_increment = actual_freq * 2.0 * std::f64::consts::PI / self.sample_rate;
    }

    pub fn poly_blep(&self, t: f64) -> f64 {
        let dt = self.phase_increment / TWO_PI;
        // 0 <= t < 1
        if t < dt {
            let t = t / dt;
            return t + t - t * t - 1.0;
        }
        // -1 < t < 0
        else if t > 1.0 - dt {
            let t = (t - 1.0) / dt;
            return t * t + t + t + 1.0;
        }
        // 0 otherwise
        else {
            return 0.0;
        }
    }

    pub fn reset(&mut self) {
        self.phase = 0.0
    }

    pub fn pitch_mod(&mut self, pitch_mod: f64) {
        self.pitch_mod = pitch_mod;
        self.update_phase_increment();
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
        let t = self.phase / TWO_PI;

        let value = match self.shape {
            Shape::Sine => self.next_aliased(Shape::Sine),
            Shape::Saw => self.next_aliased(Shape::Saw) - self.poly_blep(t),
            Shape::Noise => self.next_aliased(Shape::Noise),
            _ => {
                let mut v = self.next_aliased(Shape::Square)
                    + self.poly_blep(t)
                    - self.poly_blep((t + 0.5) % 1.0);
                if let Shape::Triangle = self.shape {
                    v = self.phase_increment * v + (1.0 - self.phase_increment) * self.last_output;
                    self.last_output = v;
                }
                v
            }
        };

        self.phase += self.phase_increment;
        while self.phase > TWO_PI {
            self.phase -= TWO_PI;
        }
        value
    }

    fn next_aliased(&mut self, shape: Shape) -> f64 {
        match shape {
            Shape::Sine => self.phase.sin(),
            Shape::Saw => (2.0 * self.phase / TWO_PI) - 1.0,
            Shape::Square => if self.phase <= PI { 1.0 } else { -1.0 }
            Shape::Triangle => 2.0 * (((2.0 * self.phase / TWO_PI) - 1.0).abs() - 0.5),
            Shape::Noise => rand::thread_rng().gen_range(-1.0, 1.0),
        }
    }
}