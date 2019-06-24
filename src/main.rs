#![feature(drain_filter)]

use cpal::{EventLoop, default_output_device, StreamData, UnknownTypeOutputBuffer};
use crate::synth::{Synth, ADSR, Playable};
use device_query::{Keycode, DeviceState, DeviceQuery};
use crate::midi::{MIDI_NOTES, EventKind, load_midi};
use crate::effects::Echo;
use std::time::Instant;
use std::cmp::max;
use std::io;
use std::io::Write;
use crate::osc::{SineOsc, SquareOsc, DetunedSaw, Combined, SawOsc};
use core::borrow::Borrow;


mod osc;
mod sampler;
mod math;
mod midi;
mod synth;
mod effects;

trait AsF32Slice {
    fn as_f32_slice(&self) -> &[f32];
}

impl<T> AsF32Slice for Vec<T> {
    fn as_f32_slice(&self) -> &[f32] {
        unsafe {
            std::slice::from_raw_parts::<f32>(
                self.as_ptr() as *const f32,
                self.len() / std::mem::size_of::<f32>(),
            )
        }
    }
}

#[derive(Debug)]
enum Event {
    On(Keycode),
    Off(Keycode),
}

struct Input {
    last_keys: Vec<Keycode>,
    state: DeviceState,
}

impl Input {
    pub fn new() -> Self {
        Input {
            state: DeviceState::new(),
            last_keys: vec![],
        }
    }

    fn poll(&mut self) -> Vec<Event> {
        let mut events = vec![];
        let keys: Vec<Keycode> = self.state.get_keys();

        for key in keys.iter() {
            if !self.last_keys.contains(key) {
                events.push(Event::On(key.clone()));
            }
        }

        for key in self.last_keys.iter() {
            if !keys.contains(key) {
                events.push(Event::Off(key.clone()));
            }
        }

        self.last_keys = keys;
        return events;
    }
}

fn keycode_to_note(k: Keycode) -> Option<f32> {
    match k {
        Keycode::Z => Some(MIDI_NOTES[60]),
        Keycode::S => Some(MIDI_NOTES[61]),
        Keycode::X => Some(MIDI_NOTES[62]),
        Keycode::D => Some(MIDI_NOTES[63]),
        Keycode::C => Some(MIDI_NOTES[64]),
        Keycode::V => Some(MIDI_NOTES[65]),
        Keycode::G => Some(MIDI_NOTES[66]),
        Keycode::B => Some(MIDI_NOTES[67]),
        Keycode::H => Some(MIDI_NOTES[68]),
        Keycode::N => Some(MIDI_NOTES[69]),
        Keycode::J => Some(MIDI_NOTES[70]),
        Keycode::M => Some(MIDI_NOTES[71]),

        Keycode::Q => Some(MIDI_NOTES[72]),
        Keycode::W => Some(MIDI_NOTES[74]),
        Keycode::E => Some(MIDI_NOTES[76]),
        Keycode::R => Some(MIDI_NOTES[77]),
        Keycode::T => Some(MIDI_NOTES[79]),
        Keycode::Y => Some(MIDI_NOTES[81]),
        Keycode::U => Some(MIDI_NOTES[83]),
        Keycode::I => Some(MIDI_NOTES[84]),
        Keycode::O => Some(MIDI_NOTES[86]),
        Keycode::P => Some(MIDI_NOTES[88]),

        _ => None
    }
}

fn main() {
    let mut input = Input::new();
    let event_loop = EventLoop::new();
    let out = default_output_device().expect("no output device");
    let format = out.default_output_format().expect("no output format");
    println!("out_device={}", out.name());
    println!("out_channels={}", format.channels);
    println!("out_sample_rate={}", format.sample_rate.0);
    println!("out_data_type={:?}", format.data_type);

    let mut song = load_midi("Pac_Man.mid");

    let stream = event_loop.build_output_stream(&out, &format).expect("cannot create output stream");
    event_loop.play_stream(stream);

    let piano = Combined::new(SquareOsc::new(format.sample_rate), SawOsc::new(format.sample_rate));

    let mut effect: Echo = Echo::new(0.2, 0.3, format.sample_rate);
    let mut synths: Vec<Box<dyn Playable<Item=f32> + Send>> = vec![
        Box::new(Synth::new(format.sample_rate, ADSR::new(0.00, 0.0, 1.0, 0.0001), SquareOsc::new(format.sample_rate))),
        Box::new(Synth::new(format.sample_rate, ADSR::new(0.00, 0.0, 1.0, 0.0001), SquareOsc::new(format.sample_rate))),
        Box::new(Synth::new(format.sample_rate, ADSR::new(0.3, 0.0, 1.0, 0.3), SineOsc::new(format.sample_rate))),
        Box::new(Synth::new(format.sample_rate, ADSR::new(0.05, 0.0, 1.0, 0.1), SquareOsc::new(format.sample_rate))),
        Box::new(Synth::new(format.sample_rate, ADSR::new(0.05, 0.0, 1.0, 0.1), SineOsc::new(format.sample_rate))),
    ];

    let volume = [
        0.2,
        0.2,
        0.1,
        0.4,
        0.2
    ];

    for t in song.tracks.iter() {
        println!("[{}] {} events", t.name, t.events.len());
        //let synth = Synth::new(format.sample_rate);
        //synths.push(synth);
    }

    let mut max_playing = 0;
    let mut frames = 0;
    let mut total_lag = 0.0;
    let mut count = 0.0;
    let start = Instant::now();

    event_loop.run(|_stream_id, _stream_data| {
        /* handle input */
        let events = input.poll();
        for e in events {
            match e {
                Event::On(k) => {
                    if let Some(f) = keycode_to_note(k) {
                        let i = &mut synths[0];
                        (**i).note_on(f, 120);
                    }
                }
                Event::Off(k) => {
                    if let Some(f) = keycode_to_note(k) {
                        synths[0].note_off(f);
                    }
                }
            }
        }


        /* handle track playing */
        let now = start.elapsed().as_micros();
        let events = song.remove_events(now);
        for (idx, e) in events {
            count += 1.0;
            total_lag += ((now - e.time) as f32).abs();
            match e.kind {
                EventKind::NoteOn(f, v) => synths[idx as usize].note_on(f, v),
                EventKind::NoteOff(f) => synths[idx as usize].note_off(f),
            }
        }

        let playing = synths.iter().fold(0, |a, e| a + e.playing());
        let max_voices = synths.iter().fold(0, |a, e| a + e.max_voices());
        max_playing = max(playing, max_playing);

        if frames % 10 == 0 {
            print!("\rbpm={}\t\ttracks={}\t\tvoices={}/{}/{}\t\tlag={}      ",
                   song.bpm,
                   song.tracks.len(),
                   playing,
                   max_playing,
                   max_voices,
                   (total_lag / count as f32) / 1000.0
            );
            io::stdout().flush().ok().expect("Could not flush stdout");
        }

        frames += 1;

        /* generate data */
        match _stream_data {
            StreamData::Output { buffer: UnknownTypeOutputBuffer::F32(mut buffer) } => {
                for elem in buffer.chunks_mut(2) {
                    let sum = synths.iter_mut().enumerate().fold(0.0, |a, (idx, e)| a + e.next().unwrap() * volume[idx]);

                    let v = effect.next(sum).unwrap() * 0.3;
                    elem[0] = v;
                    elem[1] = v;
                }
            }
            _ => (),
        }
    });
}
