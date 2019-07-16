#[derive(Copy, Clone)]
pub enum Mode {
    Lowpass,
    Highpass,
    Bandpass,
}

#[derive(Copy, Clone)]
pub struct Filter {
    cutoff: f64,
    cutoff_mod: f64,
    resonance: f64,
    mode: Mode,
    feedback_amount: f64,
    buf: [f64; 4],
}

impl Filter {
    pub fn new(cutoff: f64) -> Self {
        let mut filter = Filter {
            cutoff,
            cutoff_mod: 0.0,
            resonance: 0.0,
            mode: Mode::Lowpass,
            feedback_amount: 0.0,
            buf: [0.0; 4],
        };
        filter.calculate_feedback_amount();
        filter
    }

    pub fn cutoff_mod(&mut self, cutoff_mod: f64) {
        self.cutoff_mod = cutoff_mod;
        self.calculate_feedback_amount();
    }

    fn calculate_feedback_amount(&mut self) {
        self.feedback_amount = self.resonance + self.resonance / (1.0 - self.calculate_cutoff());
    }

    #[inline]
    fn calculate_cutoff(&self) -> f64 {
        (self.cutoff + self.cutoff_mod).clamp(0.01, 0.99)
    }

    pub fn next(&mut self, input: f64) -> f64 {
        if input == 0.0 { return input; };

        let calculated_cutoff = self.calculate_cutoff();

        self.buf[0] += calculated_cutoff * (input - self.buf[0]);
        self.buf[1] += calculated_cutoff * (self.buf[0] - self.buf[1]);
        self.buf[2] += calculated_cutoff * (self.buf[1] - self.buf[2]);
        self.buf[3] += calculated_cutoff * (self.buf[2] - self.buf[3]);

        match self.mode {
            Mode::Lowpass => self.buf[3],
            Mode::Highpass => input - self.buf[3],
            Mode::Bandpass => self.buf[0] - self.buf[3],
        }
    }
}

