use ghakuf::messages::*;
use ghakuf::reader::*;
use std::path;

enum Letter { C, CSharp, D, E, F, G, A, H }
struct Note(u8);

impl Note {
    const C4: Note = Note(8);

    fn octave(&self, octave: i8) -> Self {
        return Note((self.0 as i8 + 12 * octave) as u8)
    }
}


fn test() {
    let note = Note::C4;
    let c5 = note.octave(1);
    let c3 = note.octave(-1);
}

pub const MIDI_NOTES: [f32; 128] = [
    8.176, 8.662, 9.177, 9.723, 10.301, 10.913, 11.562, 12.250, 12.978, 13.750, 14.568,
    15.434, 16.352, 17.324, 18.354, 19.445, 20.601, 21.826, 23.124, 24.499, 25.956,
    27.500, 29.135, 30.867, 32.703, 34.648, 36.708, 38.890, 41.203, 43.653, 46.249,
    48.999, 51.913, 55.000, 58.270, 61.735, 65.406, 69.295, 73.416, 77.781, 82.406,
    87.307, 92.499, 97.998, 103.82, 110.00, 116.54, 123.47, 130.81, 138.59, 146.83,
    155.56, 164.81, 174.61, 184.99, 195.99, 207.65, 220.00, 233.08, 246.94, 261.63,
    277.18, 293.66, 311.13, 329.63, 349.23, 369.99, 391.99, 415.31, 440.00, 466.16,
    493.88, 523.25, 554.37, 587.33, 622.25, 659.26, 698.46, 739.99, 783.99, 830.61,
    880.00, 932.32, 987.77, 1046.5, 1108.7, 1174.7, 1244.5, 1318.5, 1396.9, 1480.0,
    1568.0, 1661.2, 1760.0, 1864.7, 1975.5, 2093.0, 2217.5, 2349.3, 2489.0, 2637.0,
    2793.8, 2960.0, 3136.0, 3322.4, 3520.0, 3729.3, 3951.1, 4186.0, 4434.9, 4698.6,
    4978.0, 5274.0, 5587.7, 5919.9, 6271.9, 6644.9, 7040.0, 7458.6, 7902.1, 8372.0,
    8869.8, 9397.3, 9956.1, 10548.1, 11175.3, 11839.8, 12543.9
];

#[derive(Debug)]
pub enum EventKind {
    NoteOn(f32, u8),
    NoteOff(f32),
}

#[derive(Debug)]
pub struct TrackEvent {
    pub kind: EventKind,
    pub time: u128,
}

#[derive(Debug)]
pub struct Track {
    pub events: Vec<TrackEvent>,
    pub idx: u8,
    pub name: String,
}

#[derive(Debug)]
pub struct Song {
    pub tracks: Vec<Track>,
    pub bpm: u8,
}

impl Song {
    pub fn new() -> Self {
        return Song { tracks: vec![], bpm: 120 };
    }

    pub fn remove_events(&mut self, lt_micros: u128) -> Vec<(u8, TrackEvent)> {
        let v = self.tracks.iter_mut().enumerate().flat_map(|(idx, track)| {
            track.events.drain_filter(|x| x.time < lt_micros).map(move |ev| {
                (idx as u8, ev)
            })
        }).collect();

        return v;
    }
}

struct MidiDecoder {
    song: Song,
    current_track_events: Vec<TrackEvent>,
    current_track_idx: usize,
    current_track_name: String,

    total_time: u128,
    mpqn: [u128; 128],
    time_base: u128,
    first_track: bool,
}

impl Handler for MidiDecoder {
    fn header(&mut self, format: u16, track: u16, time_base: u16) {
        println!("header {} {} {}", format, track, time_base);
        self.time_base = time_base as u128;
    }
    fn meta_event(&mut self, delta_time: u32, event: &MetaEvent, data: &Vec<u8>) {
        match event {
            MetaEvent::SequenceOrTrackName => {
                self.current_track_name = String::from_utf8_lossy(&data).to_string();
            }
            MetaEvent::SetTempo => {
                let mpqn = ((data[0] as u64) << 16 | (data[1] as u64) << 8 | data[2] as u64) as u128;

                for i in self.current_track_idx..128 {
                    self.mpqn[i] = mpqn;
                }

                let bpm = 60_000_000 / self.mpqn[self.current_track_idx];
                self.song.bpm = bpm as u8;

                println!("tempo ({}) mpqn={} bpm={}", self.current_track_idx, self.mpqn[self.current_track_idx], bpm);
                self.current_track_idx += 1;
            }
            MetaEvent::TimeSignature => {
                println!("{} {:?}", event, data)
            }
            MetaEvent::EndOfTrack => {}
            _ => println!("{} {:?}", event, data)
        }
    }
    fn midi_event(&mut self, delta_time: u32, event: &MidiEvent) {
        let tick = self.mpqn[self.current_track_idx] as f32 / self.time_base as f32;
        let tick = tick as f32;

        self.total_time += (delta_time as f32 * tick) as u128;

        let e = match event {
            MidiEvent::NoteOff { note, velocity, ch } => {
                if *ch == 9u8 { None } else {
                    Some(TrackEvent {
                        time: self.total_time,
                        kind: EventKind::NoteOff(MIDI_NOTES[*note as usize]),
                    })
                }
            }
            MidiEvent::NoteOn { note, velocity, ch } => {
                if *ch == 9u8 { None } else {
                    Some(TrackEvent {
                        time: self.total_time,
                        kind: if *velocity == 0u8 { EventKind::NoteOff(MIDI_NOTES[*note as usize]) } else { EventKind::NoteOn(MIDI_NOTES[*note as usize], *velocity) },
                    })
                }
            }
            MidiEvent::ProgramChange { ch, program } => {
                println!("channel {} changed program to {}", ch, program);
                None
            }
            MidiEvent::ControlChange { ch, control, data } => {
                println!("channel {} control change {} = {}", ch, control, data);
                None
            }
            MidiEvent::PolyphonicKeyPressure { ch, note, velocity } => {
                println!("channel {} aftertouch note {} = {}", ch, note, velocity);
                None
            }
            MidiEvent::PitchBendChange { ch, data } => {
                println!("channel {} pitch bend {}", ch, data);
                None
            }
            MidiEvent::ChannelPressure { ch, pressure } => {
                println!("channel {} aftertouch all notes = {}", ch, pressure);
                None
            }
            _ => None
        };
        if let Some(t) = e {
            self.current_track_events.push(t);
        }
    }
    fn sys_ex_event(&mut self, delta_time: u32, event: &SysExEvent, data: &Vec<u8>) {
        println!("{}", event);
    }
    fn track_change(&mut self) {
        self.total_time = 0;
        if self.first_track {
            self.current_track_idx = 0;
        }
        self.current_track_idx += 1;

        if self.current_track_events.len() != 0 {
            let idx = self.song.tracks.len() as u8;
            let track = Track {
                idx,
                events: std::mem::replace(&mut self.current_track_events, vec![]),
                name: std::mem::replace(&mut self.current_track_name, format!("track #{}", idx)),
            };
            self.song.tracks.push(track);
        }
    }
}

pub fn load_midi(path: &str) -> Song {
    let path = path::Path::new(path);
    let mut handler = MidiDecoder {
        song: Song::new(),
        current_track_events: vec![],
        total_time: 0,
        mpqn: [500000; 128],
        time_base: 0,
        current_track_idx: 0,
        current_track_name: "track #0".to_string(),
        first_track: true,
    };
    let mut reader = Reader::new(&mut handler, &path).unwrap();
    let _ = reader.read();

    handler.track_change();

    return handler.song;
}