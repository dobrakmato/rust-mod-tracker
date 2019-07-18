use crate::env::EnvelopeState::{Attack, Sustain, Off, Decay, Release};
use crate::osc::{Osc, Shape};
use crate::midi::note2freq;
use crate::env::Envelope;
use crate::filter::{Mode, Filter};
use rand::Rng;

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

        let a = self.filter_env.next();

        self.filter.cutoff_mod(a * self.filter_envelope_amount);

        return self.filter.next(self.osc.next() * self.env.next() * self.velocity);
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

pub struct Preset {
    pub waveform: Shape,
    pub attack: f64,
    pub decay: f64,
    pub sustain: f64,
    pub release: f64,
    pub filter_mode: Mode,
    pub filter_cutoff: f64,
    pub filter_resonance: f64,
    pub filter_attack: f64,
    pub filter_decay: f64,
    pub filter_sustain: f64,
    pub filter_release: f64,
    pub filter_evn_amount: f64,
}

impl Preset {
    pub(crate) fn random() -> Self {
        let mut rng = rand::thread_rng();

        let release = rng.gen_range(0.05, 2.0);

        Preset {
            waveform: rng.gen(),
            attack: rng.gen_range(0.01, 0.4),
            decay: rng.gen_range(0.3, 0.7),
            sustain: rng.gen_range(0.2, 0.8),
            release,
            filter_mode: rng.gen(),
            filter_cutoff: rng.gen_range(0.01, 0.99),
            filter_resonance: rng.gen_range(0.01, 0.99),
            filter_attack: rng.gen_range(0.01, 0.4),
            filter_decay: rng.gen_range(0.01, 0.7),
            filter_sustain: rng.gen_range(0.2, 0.8),
            filter_release: rng.gen_range(0.05, release),
            filter_evn_amount: rng.gen_range(-0.5, 0.5),
        }
    }
}

impl Synth {
    pub fn new(sample_rate: f64) -> Self {
        Synth {
            voices: Voices::new(sample_rate, 32)
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

    pub fn voices(&self) -> (usize, usize) {
        let mut available = self.voices.voices.len();
        let mut used = self.voices.voices.iter()
            .filter(|x| x.is_active)
            .count();

        return (available, used);
    }

    pub fn apply_preset(&mut self, preset: &Preset) {
        for voice in self.voices.voices.iter_mut() {
            voice.osc.shape = preset.waveform;

            voice.env.attack(preset.attack);
            voice.env.decay(preset.decay);
            voice.env.sustain(preset.sustain);
            voice.env.release(preset.release);

            voice.filter.mode = preset.filter_mode;
            voice.filter.cutoff(preset.filter_cutoff);
            voice.filter.resonance(preset.filter_resonance);

            voice.filter_env.attack(preset.filter_attack);
            voice.filter_env.decay(preset.filter_decay);
            voice.filter_env.sustain(preset.filter_sustain);
            voice.filter_env.release(preset.filter_release);

            voice.filter_envelope_amount = preset.filter_evn_amount;
        }
    }
}