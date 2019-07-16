
#[derive(Copy, Clone)]
pub enum Mode {
    Lowpass,
    Highpass,
    Bandpass,
}

#[derive(Copy, Clone)]
pub struct Filter {
    cutoff: f64,
    resonance: f64,
    mode: Mode,
    feedback_amount: f64,
    buf: [f64; 2],
}

impl Filter {
    pub fn new() -> Self {
        let mut filter = Filter {
            cutoff: 0.99,
            resonance: 0.0,
            mode: Mode::Lowpass,
            feedback_amount: 0.0,
            buf: [0.0; 2],
        };
        filter.calculate_feedback_amount();
        filter
    }

    fn calculate_feedback_amount(&mut self) {
        self.feedback_amount = self.resonance + self.resonance / (1.0 - self.cutoff);
    }

    pub fn next(&mut self, input: f64) -> f64 {
        self.buf[0] += self.cutoff * (input - self.buf[0]);
        self.buf[1] += self.cutoff * (self.buf[0] - self.buf[1]);

        match self.mode {
            Mode::Lowpass => self.buf[1],
            Mode::Highpass => input - self.buf[1],
            Mode::Bandpass => self.buf[0] - self.buf[1],
        }
    }
}

