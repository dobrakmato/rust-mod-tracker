use crate::osc::{SquareOsc, DetunedSaw, Voice};
use cpal::SampleRate;

#[derive(Copy, Clone)]
struct ADSRState {
    current_time: f32,
    note_off_time: f32,
    note_on: bool,
}

impl ADSRState {
    fn new() -> Self {
        return ADSRState {
            current_time: 0.0,
            note_off_time: 0.0,
            note_on: false,
        };
    }

    fn note_on(&mut self) {
        self.current_time = 0.0;
        self.note_off_time = 0.0;
        self.note_on = true;
    }
}

#[derive(Copy, Clone)]
pub struct ADSR {
    pub attack: f32,
    pub decay: f32,
    pub sustain: f32,
    pub release: f32,
    state: ADSRState,
}

impl ADSR {
    pub fn new(attack: f32, decay: f32, sustain: f32, release: f32) -> Self {
        ADSR {
            attack,
            decay,
            sustain,
            release,
            state: ADSRState::new(),
        }
    }

    pub fn note_on(&mut self) {
        self.state.note_on();
    }

    pub fn note_off(&mut self) {
        self.state.note_on = false;
    }

    pub fn is_silent(&self) -> bool {
        return self.state.note_off_time >= self.release;
    }

    pub fn step(&mut self, time_to_add: f32) -> f32 {
        if self.state.note_on {
            self.state.current_time += time_to_add;

            // we in attack
            if self.state.current_time < self.attack {
                return (self.state.current_time / self.attack).min(1.0);
            }

            // we in decay
            if self.state.current_time < self.decay + self.attack {
                let f = (self.state.current_time - self.attack) / self.decay;
                return (1.0 - f + self.sustain).min(1.0);
            }

            // we are in sustain
            return self.sustain.min(1.0);
        } else {
            self.state.note_off_time += time_to_add;

            // we in release
            let inv = 1.0 / self.release;
            return ((self.release - self.state.note_off_time) * inv * self.sustain).max(0.0);
        }
    }
}

pub trait Playable {
    type Item;
    fn note_on(&mut self, hz: f32, velocity: u8);
    fn note_off(&mut self, hz: f32);
    fn playing(&self) -> usize;
    fn max_voices(&self) -> usize;
    fn next(&mut self) -> Option<Self::Item>;
}

pub struct Synth<T> {
    sample_rate: u32,
    voices: [T; 24],
    velocities: [u8; 24],
    adsr: [ADSR; 24],
    playing: [bool; 24],
    releasing: [bool; 24],
}

impl<T> Synth<T> where T: Voice + Copy {
    pub fn new(sample_rate: SampleRate, adsr: ADSR, voice: T) -> Self {
        Synth {
            sample_rate: sample_rate.0,
            voices: [voice; 24],
            velocities: [0; 24],
            adsr: [adsr; 24],
            playing: [false; 24],
            releasing: [false; 24],
        }
    }
}

impl<T> Playable for Synth<T> where T: Voice {
    type Item = f32;

    fn note_on(&mut self, hz: f32, velocity: u8) {
        let mut idx = 0;
        for i in 0..self.voices.len() {
            if !self.playing[i] {
                idx = i;
                break;
            }
        }

        self.adsr[idx].note_on();
        self.velocities[idx] = velocity;
        self.voices[idx].set_hz(hz);
        self.playing[idx] = true;
    }

    fn note_off(&mut self, hz: f32) {
        for i in 0..self.voices.len() {
            if self.playing[i] && !self.releasing[i] && self.voices[i].get_hz() == hz {
                self.adsr[i].note_off();
                self.releasing[i] = true;
                return;
            }
        }
    }

    fn playing(&self) -> usize {
        return self.playing.iter().filter(|x| **x).count();
    }

    fn max_voices(&self) -> usize {
        self.voices.len()
    }

    fn next(&mut self) -> Option<Self::Item> {
        let mut sum = 0.0;
        for idx in 0..self.voices.len() {
            if self.playing[idx] {
                let note = self.voices[idx].next().unwrap();
                let vel = self.velocities[idx] as f32 / 128.0;
                let env = self.adsr[idx].step(1.0 / self.sample_rate as f32);

                sum += note * vel * env;

                if self.adsr[idx].is_silent() {
                    self.playing[idx] = false;
                    self.releasing[idx] = false;
                }
            }
        }

        return Some(sum);
    }
}