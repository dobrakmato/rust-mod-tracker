use crate::env::EnvelopeState::{Attack, Sustain, Off, Decay, Release};

#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
pub enum EnvelopeState {
    Attack,
    Decay,
    Release,
    Sustain,
    Off,
}

impl EnvelopeState {
    fn next(&self) -> EnvelopeState {
        match self {
            Attack => Decay,
            Decay => Sustain,
            Release => Off,
            Sustain => Release,
            Off => Off,
        }
    }
}

#[derive(Copy, Clone)]
pub struct Envelope {
    state: EnvelopeState,
    current_level: f64,
    sample_rate: f64,
    multiplier: f64,
    current_sample_idx: u64,
    next_state_sample_idx: u64,
    pub attack: f64,
    pub decay: f64,
    pub release: f64,
    pub sustain: f64,
}

const MINIMUM_LEVEL: f64 = 0.001;

impl Envelope {
    pub fn new(sample_rate: f64) -> Self {
        return Envelope {
            state: EnvelopeState::Off,
            current_level: 0.0,
            sample_rate,
            multiplier: 1.0,
            current_sample_idx: 0,
            next_state_sample_idx: 0,
            attack: 0.1,
            decay: 0.5,
            sustain: 0.7,
            release: 1.0,
        };
    }

    #[inline]
    pub fn state(&self) -> EnvelopeState {
        self.state
    }

    pub fn next(&mut self) -> f64 {
        /* when off or sustain the level must stay same */
        if self.state == Sustain || self.state == Off {
            return self.current_level;
        }

        if self.current_sample_idx == self.next_state_sample_idx {
            self.enter_state(self.state.next())
        }

        self.current_level *= self.multiplier;
        self.current_sample_idx += 1;

        return self.current_level;
    }

    fn calculate_multiplier(&mut self, start_level: f64, end_level: f64, length_samples: u64) {
        self.multiplier = 1.0 + (end_level.ln() - start_level.ln()) / (length_samples as f64);
    }

    pub fn enter_state(&mut self, state: EnvelopeState) {
        self.state = state;
        self.current_sample_idx = 0;
        self.next_state_sample_idx = (self.sample_rate * match state {
            Attack => self.attack,
            Decay => self.decay,
            Release => self.release,
            Sustain => self.sustain,
            Off => 0.0,
        }) as u64;

        match state {
            Attack => {
                self.current_level = MINIMUM_LEVEL;
                self.calculate_multiplier(self.current_level,
                                          1.0,
                                          self.next_state_sample_idx)
            }
            Decay => {
                self.current_level = 1.0;
                self.calculate_multiplier(self.current_level,
                                          self.sustain.max(MINIMUM_LEVEL),
                                          self.next_state_sample_idx)
            }
            Sustain => {
                self.current_level = self.sustain;
                self.multiplier = 1.0;
            }
            Release => {
                self.calculate_multiplier(self.current_level,
                                          MINIMUM_LEVEL,
                                          self.next_state_sample_idx)
            }
            Off => {
                self.current_level = 0.0;
                self.multiplier = 1.0;
            }
        }
    }
}