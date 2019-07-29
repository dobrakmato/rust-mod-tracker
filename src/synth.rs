use crate::env::EnvelopeState::{Attack, Sustain, Off, Decay, Release};
use crate::osc::{Osc, Shape};
use crate::midi::note2freq;
use crate::env::Envelope;
use crate::filter::{Mode, Filter};
use rand::Rng;

#[derive(Copy, Clone)]
pub struct Voice {
    pub osc1: Osc,
    pub osc2: Osc,
    pub osc1_pitch_mod: f64,
    pub osc2_pitch_mod: f64,
    pub osc_mix: f64,
    pub env: Envelope,
    pub filter: Filter,
    pub filter_env: Envelope,
    pub velocity: f64,
    pub note: u8,
    pub filter_envelope_amount: f64,
    pub is_active: bool,
}

pub type Semitone = f64;

impl Voice {
    fn new(sample_rate: f64) -> Self {
        Voice {
            osc1: Osc::new(sample_rate),
            osc2: Osc::new(sample_rate),
            osc1_pitch_mod: 0.0,
            osc2_pitch_mod: 0.0,
            osc_mix: 0.5,
            filter: Filter::new(0.1),
            env: Envelope::new(sample_rate),
            filter_env: Envelope::new(sample_rate),
            filter_envelope_amount: 1.0,
            velocity: 1.0,
            note: 0,
            is_active: false,
        }
    }

    pub fn next(&mut self, lfo_value: f64, lfo_filter_amount: f64) -> f64 {
        if self.env.state() == Off { self.is_active = false; }

        let osc1 = self.osc1.next();
        let osc2 = self.osc2.next();
        let mix = ((1.0 - self.osc_mix) * osc1) + (self.osc_mix * osc2);

        self.filter.cutoff_mod(self.filter_env.next() * self.filter_envelope_amount + lfo_value * lfo_filter_amount);

        self.osc1.pitch_mod(lfo_value * self.osc1_pitch_mod);
        self.osc2.pitch_mod(lfo_value * self.osc2_pitch_mod);

        return self.filter.next(mix * self.env.next() * self.velocity);
    }

    pub fn reset(&mut self) {
        self.velocity = 0.0;
        self.osc1.reset();
        self.osc2.reset();
        self.env.reset();
        self.filter_env.reset();
        self.filter.reset();
    }
}

pub struct Voices {
    voices: Vec<Voice>,
    lfo: Osc,
    lfo_filter_amount: f64,
    osc1_tuning: Semitone,
    osc2_tuning: Semitone,
}

impl Voices {
    fn new(sample_rate: f64, polyphony: usize) -> Self {
        Voices {
            voices: vec![Voice::new(sample_rate); polyphony],
            lfo: Osc::new(sample_rate),
            lfo_filter_amount: 0.0,
            osc1_tuning: 0.0,
            osc2_tuning: 0.0,
        }
    }

    pub fn note_on(&mut self, note: u8, velocity: u8) {
        for v in self.voices.iter_mut() {
            if !v.is_active {
                v.reset();
                v.is_active = true;
                v.note = note;
                v.velocity = velocity as f64 / 127.0;
                v.osc1.frequency(note2freq(note as f64 + self.osc1_tuning));
                v.osc2.frequency(note2freq(note as f64 + self.osc2_tuning));
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
        let lfo_value = self.lfo.next();
        let lfo_filter_amount = self.lfo_filter_amount;

        /* sum active voices */
        self.voices.iter_mut()
            .filter(|v| v.is_active)
            .map(|v| v.next(lfo_value, lfo_filter_amount))
            .sum()
    }
}

pub struct Preset {
    pub osc1_waveform: Shape,
    pub osc1_pitch_mod: f64,
    pub osc1_tuning: f64,
    pub osc2_waveform: Shape,
    pub osc2_pitch_mod: f64,
    pub osc2_tuning: f64,
    pub osc_mix: f64,
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
    pub lfo_waveform: Shape,
    pub lfo_frequency: f64,
    pub lfo_filter_mod_amount: f64,
}

impl Preset {
    pub(crate) fn random() -> Self {
        let mut rng = rand::thread_rng();

        let release = rng.gen_range(0.05, 2.0);

        Preset {
            osc1_waveform: rng.gen(),
            osc2_waveform: rng.gen(),
            osc1_pitch_mod: rng.gen_range(0.0, 0.1),
            osc2_pitch_mod: rng.gen_range(0.0, 0.1),
            osc1_tuning: rng.gen_range(-12.0, 12.0),
            osc2_tuning: rng.gen_range(-12.0, 12.0),
            osc_mix: rng.gen_range(0.0, 1.5),
            attack: rng.gen_range(0.01, 0.2),
            decay: rng.gen_range(0.3, 0.7),
            sustain: rng.gen_range(0.2, 0.8),
            release,
            filter_mode: rng.gen(),
            filter_cutoff: rng.gen_range(0.01, 0.99),
            filter_resonance: rng.gen_range(0.01, 0.9),
            filter_attack: rng.gen_range(0.01, 0.4),
            filter_decay: rng.gen_range(0.01, 0.7),
            filter_sustain: rng.gen_range(0.2, 0.8),
            filter_release: rng.gen_range(0.05, release),
            filter_evn_amount: rng.gen_range(-0.5, 0.5),
            lfo_waveform: rng.gen(),
            lfo_frequency: (rng.gen_range(0.0, 1000.0) as f64).ln(),
            lfo_filter_mod_amount: rng.gen_range(0.0, 0.2),
        }
    }
}

pub struct Synth {
    voices: Voices
}


impl Synth {
    pub fn new(sample_rate: f64) -> Self {
        Synth {
            voices: Voices::new(sample_rate, 128)
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
            voice.osc1.shape = preset.osc1_waveform;
            voice.osc2.shape = preset.osc2_waveform;
            voice.osc1_pitch_mod = preset.osc1_pitch_mod;
            voice.osc2_pitch_mod = preset.osc2_pitch_mod;
            voice.osc_mix = preset.osc_mix;

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

        self.voices.lfo_filter_amount = preset.lfo_filter_mod_amount;
        self.voices.lfo.frequency(preset.lfo_frequency);
        self.voices.lfo.shape = preset.lfo_waveform;

        self.voices.osc1_tuning = preset.osc1_tuning;
        self.voices.osc2_tuning = preset.osc2_tuning;
    }
}