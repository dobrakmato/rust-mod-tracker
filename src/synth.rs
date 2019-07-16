use crate::env::EnvelopeState::{Attack, Sustain, Off, Decay, Release};
use crate::osc::{Osc, Shape};
use crate::midi::note2freq;
use crate::env::Envelope;
use crate::filter::{Mode, Filter};

pub struct Preset {
    waveform: Shape,
    attack: f64,
    decay: f64,
    sustain: f64,
    release: f64,
    filter_mode: Mode,
    filter_cutoff: f64,
    filter_resonance: f64,
    filter_attack: f64,
    filter_decay: f64,
    filter_sustain: f64,
    filter_release: f64,
    filter_evn_amount: f64,
}

#[derive(Copy, Clone)]
pub struct Voice {
    pub osc: Osc,
    pub env: Envelope,
    pub filter: Filter,
    pub filter_env: Envelope,
    pub velocity: f64,
    pub note: u8,
    pub filter_envelope_amount: f64,
    pub is_active: bool,
}

impl Voice {
    fn new(sample_rate: f64) -> Self {
        Voice {
            osc: Osc::new(sample_rate),
            filter: Filter::new(0.1),
            env: Envelope::new(sample_rate),
            filter_env: Envelope::new(sample_rate),
            filter_envelope_amount: 1.0,
            velocity: 1.0,
            note: 0,
            is_active: false,
        }
    }

    pub fn next(&mut self) -> f64 {
        if self.env.state() == Off { self.is_active = false; }

        self.filter.cutoff_mod(self.filter_env.next() * self.filter_envelope_amount);

        return self.filter.next(self.osc.next() * self.env.next() * self.velocity)
    }
}

pub struct Voices {
    voices: Vec<Voice>
}

impl Voices {
    fn new(sample_rate: f64, polyphony: usize) -> Self {
        Voices { voices: vec![Voice::new(sample_rate); polyphony] }
    }

    pub fn note_on(&mut self, note: u8, velocity: u8) {
        for v in self.voices.iter_mut() {
            if !v.is_active {
                v.is_active = true;
                v.note = note;
                v.velocity = velocity as f64 / 127.0;
                v.osc.frequency(note2freq(note));
                v.env.enter_state(Attack);
                v.filter_env.enter_state(Attack);
                break;
            }
        }
    }

    pub fn note_off(&mut self, note: u8) {
        for v in self.voices.iter_mut() {
            if v.is_active && v.note == note {
                v.env.enter_state(Release);
                v.filter_env.enter_state(Release);
            }
        }
    }

    pub fn next(&mut self) -> f64 {
        /* sum active voices */
        self.voices.iter_mut()
            .filter(|v| v.is_active)
            .map(|v| v.next())
            .sum()
    }
}

pub struct Synth {
    voices: Voices
}

impl Synth {
    pub fn new(sample_rate: f64) -> Self {
        Synth {
            voices: Voices::new(sample_rate, 96)
        }
    }

    pub fn note_on(&mut self, note: u8, velocity: u8) {
        self.voices.note_on(note, velocity)
    }

    pub fn note_off(&mut self, note: u8) {
        self.voices.note_off(note)
    }

    pub fn next(&mut self) -> f64 {
        self.voices.next()
    }

    pub fn apply_preset(&mut self, preset: &Preset) {}
}