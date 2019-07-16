use cpal::{EventLoop, default_output_device, StreamData, UnknownTypeOutputBuffer};
use device_query::{Keycode, DeviceState, DeviceQuery};
use crate::midi::{load_midi, Kind, note2freq, Player, Event};
use std::time::Instant;
use std::cmp::max;
use std::io;
use std::io::Write;
use core::borrow::Borrow;
use crate::osc::Osc;
use std::process::exit;
use std::fs::File;
use std::path::Path;
use std::ffi::OsStr;
use crate::synth::Synth;


mod osc;
mod sampler;
mod math;
mod filter;
mod midi;
mod effects;
mod env;
mod synth;


fn main() {
    let event_loop = EventLoop::new();
    let out = default_output_device().expect("no output device");
    let format = out.default_output_format().expect("no output format");
    println!("out_device={}", out.name());
    println!("out_channels={}", format.channels);
    println!("out_sample_rate={}", format.sample_rate.0);
    println!("out_data_type={:?}", format.data_type);

    let mut synth = Synth::new(format.sample_rate.0 as f64);

    //for x in std::fs::read_dir(".").unwrap() {
    //    let x = x.unwrap().path();
    //    if x.extension().is_some() && (x.extension().unwrap() == OsStr::new("mid") || x.extension().unwrap() == OsStr::new("MID")) {
    //        load_midi(x.as_path());
    //    }
    //}

    let mut midi = load_midi(Path::new("tet.mid"));
    let mut player = Player::new(&midi);
    // println!("{:#?}", midi);

    let stream = event_loop
        .build_output_stream(&out, &format)
        .expect("cannot create output stream");
    event_loop.play_stream(stream);

    let mut export: Vec<f32> = vec![];
    let start = Instant::now();

    event_loop.run(|_stream_id, _stream_data| {
        /* playback */
        let now = start.elapsed().as_micros();
        for event in player.get_events(now as f64) {
            match event.kind {
                Kind::NoteOn { ch, note, velocity } => {
                    // println!("note_on {} {}", note, velocity);
                    synth.note_on(note, velocity)
                }
                Kind::NoteOff { ch, note } => {
                    // println!("note_off {}", note);
                    synth.note_off(note)
                }
                _ => {}
            }
        }

        /* generate data */
        match _stream_data {
            StreamData::Output { buffer: UnknownTypeOutputBuffer::F32(mut buffer) } => {
                for elem in buffer.chunks_mut(2) {
                    let v = synth.next();

                    /* exporting file */
                    //export.push(v as f32);
                    if export.len() == format.sample_rate.0 as usize {
                        let mut f = File::create("export.raw").unwrap();
                        for num in export.iter() {
                            let i = num.to_bits();
                            #[inline]
                            fn u32tou8abe(v: u32) -> [u8; 4] {
                                [
                                    (v >> 24) as u8,
                                    (v >> 16) as u8,
                                    (v >> 8) as u8,
                                    v as u8,
                                ]
                            }

                            f.write(&u32tou8abe(i));
                        }
                        f.flush();
                        exit(0);
                    }

                    elem[0] = v as f32;
                    elem[1] = v as f32;
                }
            }
            _ => (),
        }
    });
}
