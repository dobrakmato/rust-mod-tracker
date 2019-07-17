use crate::synth::Preset;
use crate::osc::Shape;
use crate::filter::Mode;

pub const PIANO: Preset = Preset {
    waveform: Shape::Square,
    attack: 0.01,
    decay: 0.15,
    sustain: 0.4,
    release: 0.9,
    filter_mode: Mode::Lowpass,
    filter_cutoff: 1.0,
    filter_resonance: 0.3,
    filter_attack: 0.1,
    filter_decay: 0.1,
    filter_sustain: 0.3,
    filter_release: 1.5,
    filter_evn_amount: -0.3,
};

pub const ORGAN: Preset = Preset {
    waveform: Shape::Saw,
    attack: 0.01,
    decay: 0.01,
    sustain: 1.0,
    release: 0.3,
    filter_mode: Mode::Lowpass,
    filter_cutoff: 0.6,
    filter_resonance: 0.0,
    filter_attack: 0.1,
    filter_decay: 0.1,
    filter_sustain: 0.3,
    filter_release: 1.5,
    filter_evn_amount: 0.0,
};

pub const GUITAR: Preset = Preset {
    waveform: Shape::Saw,
    attack: 0.01,
    decay: 0.4,
    sustain: 0.6,
    release: 1.0,
    filter_mode: Mode::Highpass,
    filter_cutoff: 0.5,
    filter_resonance: 0.0,
    filter_attack: 0.1,
    filter_decay: 0.1,
    filter_sustain: 0.3,
    filter_release: 1.5,
    filter_evn_amount: 0.0,
};


pub const BASS: Preset = Preset {
    waveform: Shape::Square,
    attack: 0.01,
    decay: 0.1,
    sustain: 0.7,
    release: 0.5,
    filter_mode: Mode::Lowpass,
    filter_cutoff: 0.05,
    filter_resonance: 0.8,
    filter_attack: 0.1,
    filter_decay: 0.1,
    filter_sustain: 0.3,
    filter_release: 1.5,
    filter_evn_amount: 0.0,
};


pub const STRINGS: Preset = Preset {
    waveform: Shape::Saw,
    attack: 0.01,
    decay: 0.1,
    sustain: 0.4,
    release: 1.5,
    filter_mode: Mode::Lowpass,
    filter_cutoff: 1.0,
    filter_resonance: 0.4,
    filter_attack: 0.01,
    filter_decay: 0.3,
    filter_sustain: 0.4,
    filter_release: 1.5,
    filter_evn_amount: -0.3,
};


pub const GENERIC: Preset = Preset {
    waveform: Shape::Square,
    attack: 0.1,
    decay: 0.2,
    sustain: 0.6,
    release: 0.8,
    filter_mode: Mode::Lowpass,
    filter_cutoff: 1.0,
    filter_resonance: 0.0,
    filter_attack: 0.01,
    filter_decay: 0.3,
    filter_sustain: 0.4,
    filter_release: 1.5,
    filter_evn_amount: 0.0,
};