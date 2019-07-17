use ghakuf::messages::*;
use ghakuf::reader::*;
use std::path;
use std::path::Path;
use ghakuf::formats::Format;
use std::slice::Iter;
use std::iter::Peekable;
use crate::synth::{Preset, Synth};
use crate::presets::{PIANO, ORGAN, GUITAR, BASS, STRINGS, GENERIC};

pub fn note2freq(note: Note) -> f64 {
    return 440.0 * 2.0f64.powf((note as f64 - 69.0) / 12.0);
}

#[derive(Debug, Copy, Clone)]
pub enum GMFamily {
    Piano,
    ChromaticPercussion,
    Organ,
    Guitar,
    Bass,
    Strings,
    Ensemble,
    Brass,
    Reed,
    Pipe,
    SynthLead,
    SynthPad,
    SynthEffects,
    Ethnic,
    Percussive,
    SoundEffects,
}

#[derive(Debug, Copy, Clone)]
pub struct GMInstrument {
    family: GMFamily,
    program_number: u8,
}

impl GMInstrument {
    fn new(program_number: u8) -> Self {
        GMInstrument {
            program_number,
            family: match program_number {
                0...7 => GMFamily::Piano,
                8...15 => GMFamily::ChromaticPercussion,
                16...23 => GMFamily::Organ,
                14...31 => GMFamily::Guitar,
                32...39 => GMFamily::Bass,
                40...47 => GMFamily::Strings,
                48...55 => GMFamily::Ensemble,
                56...63 => GMFamily::Brass,
                64...71 => GMFamily::Reed,
                72...79 => GMFamily::Pipe,
                80...88 => GMFamily::SynthLead,
                88...95 => GMFamily::SynthPad,
                96...103 => GMFamily::SynthEffects,
                104...111 => GMFamily::Ethnic,
                112...119 => GMFamily::Percussive,
                120...127 => GMFamily::SoundEffects,
                _ => panic!("invalid program number {}", program_number)
            },
        }
    }

    #[inline]
    pub fn program_number(&self) -> u8 {
        return self.program_number;
    }

    fn preset(&self) -> Preset {
        match self.family {
            GMFamily::Piano => PIANO,
            GMFamily::Organ => ORGAN,
            GMFamily::Guitar => GUITAR,
            GMFamily::Bass => BASS,
            GMFamily::Strings => STRINGS,
            _ => GENERIC
        }
    }
}

#[derive(Debug)]
pub struct Midi {
    pub tracks: Vec<Track>,
    pub time_division: u16,
    pub mpqn: u128,
    pub tick_length: f64,
    pub total_time: f64,
    pub name: String,
    pub format: Format,
}

impl Midi {
    pub fn new(name: String) -> Self {
        Midi {
            format: Format::Unknown,
            mpqn: 500000,
            tick_length: 0.0,
            total_time: 0.0,
            time_division: 0,
            tracks: vec![],
            name,
        }
    }
}

pub type Channel = u8;
pub type Note = u8;
pub type Velocity = u8;

#[derive(Debug)]
pub enum Kind {
    NoteOn {
        ch: Channel,
        note: Note,
        velocity: Velocity,
    },
    NoteOff {
        ch: Channel,
        note: Note,
    },
    Instrument {
        ch: Channel,
        instrument: GMInstrument,
    },
}

#[derive(Debug)]
pub struct Event {
    pub kind: Kind,
    pub time: f64,
}

#[derive(Debug)]
pub struct Track {
    name: Option<String>,
    id: usize,
    pub events: Vec<Event>,
}

struct MidiReader {
    midi: Midi
}

impl Handler for MidiReader {
    fn header(&mut self, format: u16, tracks: u16, time_division: u16) {
        println!("name={} format={} tracks={} time_division={}", self.midi.name, format, tracks, time_division);

        self.midi.time_division = time_division; // ppqn
        self.midi.tick_length = self.midi.mpqn as f64 / self.midi.time_division as f64;
        self.midi.format = Format::new(format);

        // If bit 15 of <time_division> is a one, delta times in a file correspond to
        // subdivisions of a second, in a way consistent with SMPTE and MIDI Time Code.
        if time_division & 0x8000 == 0x8000 {
            eprintln!("SMPTE not supported!");
        }

        // the file contains one or more sequentially independent single-track patterns
        if format == 2 {
            eprintln!("Format 2 Midi files are not supported!");
        }
    }

    /// Fired when meta event has found.
    fn meta_event(&mut self, delta_time: u32, event: &MetaEvent, data: &Vec<u8>) {
        match event {
            MetaEvent::SequenceOrTrackName => self.midi.tracks.last_mut().unwrap().name = Some(String::from_utf8_lossy(data).to_string()),
            MetaEvent::SetTempo => {
                // todo: tracks can change tempo during playing multiple times

                let mpqn = ((data[0] as u64) << 16 | (data[1] as u64) << 8 | data[2] as u64) as u128;
                let bpm = 60_000_000 / mpqn;

                self.midi.mpqn = mpqn;
                self.midi.tick_length = self.midi.mpqn as f64 / self.midi.time_division as f64;

                println!("set_tempo {} {}bpm", delta_time, bpm)
            }
            MetaEvent::SMTPEOffset => eprintln!("SMTPEOffset is not supported!"),

            MetaEvent::CuePoint => {} /* ignored */
            MetaEvent::Lyric => {} /* ignored */
            MetaEvent::Marker => {} /* ignored */
            MetaEvent::InstrumentName => {} /* ignored */
            MetaEvent::MIDIChannelPrefix => {} /* used only with instrument name, ignored */
            MetaEvent::TextEvent => {} /* ignored */
            MetaEvent::CopyrightNotice => {} /* ignored */
            MetaEvent::SequenceNumber => {} /* ignored */
            MetaEvent::SequencerSpecificMetaEvent => {} /* ignored */
            MetaEvent::KeySignature => {} /* ignored */
            MetaEvent::TimeSignature => {} /* ignored */
            MetaEvent::EndOfTrack => {} /* silent */
            MetaEvent::Unknown { event_type } => {} /* silent */
        }
    }

    /// Fired when MIDI event has found.
    fn midi_event(&mut self, delta_time: u32, event: &MidiEvent) {
        self.midi.total_time += (delta_time as f64 * self.midi.tick_length);
        let track = self.midi.tracks.last_mut().unwrap();

        match event {
            MidiEvent::NoteOff { ch, note, velocity } | MidiEvent::NoteOn { ch, note, velocity: velocity @ 0 } => {
                track.events.push(Event {
                    time: self.midi.total_time,
                    kind: Kind::NoteOff { note: *note, ch: *ch },
                })
            }
            MidiEvent::NoteOn { ch, note, velocity } => {
                track.events.push(Event {
                    time: self.midi.total_time,
                    kind: Kind::NoteOn {
                        ch: *ch,
                        note: *note,
                        velocity: *velocity,
                    },
                })
            }
            MidiEvent::ProgramChange { ch, program } => {
                track.events.push(Event {
                    time: self.midi.total_time,
                    kind: Kind::Instrument {
                        ch: *ch,
                        instrument: GMInstrument::new(*program),
                    },
                })
            }
            MidiEvent::ControlChange { ch, control, data } => {}
            MidiEvent::PitchBendChange { ch, data } => {}
            MidiEvent::ChannelPressure { ch, pressure } => {} /* unsupported */
            MidiEvent::PolyphonicKeyPressure { ch, note, velocity } => {} /* unsupported */
            MidiEvent::Unknown { .. } => {} /* silent */
        }
    }

    /// Fired when track has changed.
    fn track_change(&mut self) {
        self.midi.total_time = 0.0;
        self.midi.tracks.push(Track {
            name: None,
            id: self.midi.tracks.len(),
            events: vec![],
        })
    }
}

pub fn load_midi(path: &Path) -> Midi {
    let mut handler = MidiReader { midi: Midi::new(path.file_name().unwrap().to_str().unwrap().to_owned()) };
    let mut reader = Reader::new(&mut handler, &path).unwrap();
    let _ = reader.read();
    return handler.midi;
}

pub struct Player<'a> {
    iterators: Vec<Peekable<Iter<'a, Event>>>
}

impl<'a> Player<'a> {
    pub fn new(midi: &'a Midi) -> Self {
        Player {
            iterators: midi.tracks.iter().map(|t| t.events.iter().peekable()).collect(),
        }
    }

    pub fn get_events(&mut self, time_micros: f64) -> Vec<&Event> {
        let mut result = vec![];

        for x in self.iterators.iter_mut() {
            let mut should_play = {
                let mut last = x.peek();
                last.is_some() && last.unwrap().time <= time_micros
            };

            while should_play {
                result.push(x.next().unwrap());
                should_play = {
                    let mut last = x.peek();
                    last.is_some() && last.unwrap().time <= time_micros
                };
            }
        }

        return result;
    }
}

pub struct MidiChannel {
    synth: Synth,
}

impl MidiChannel {
    pub fn new(sample_rate: f64) -> Self {
        MidiChannel {
            synth: Synth::new(sample_rate),
        }
    }

    pub fn next(&mut self) -> f64 {
        self.synth.next()
    }
}


pub struct MidiPlayback {
    channels: Vec<MidiChannel>
}

impl MidiPlayback {
    pub fn new(sample_rate: f64) -> Self {
        MidiPlayback {
            channels: vec![0; 16].into_iter().map(|x| MidiChannel::new(sample_rate)).collect()
        }
    }

    pub fn note_on(&mut self, ch: Channel, note: Note, velocity: Velocity) {
        self.channels[ch as usize].synth.note_on(note, velocity)
    }

    pub fn note_off(&mut self, ch: Channel, note: Note) {
        self.channels[ch as usize].synth.note_off(note)
    }

    pub fn set_instrument(&mut self, ch: Channel, instrument: GMInstrument) {
        self.channels[ch as usize].synth.apply_preset(&instrument.preset())
    }

    pub fn next(&mut self) -> f64 {
        self.channels.iter_mut()
            .enumerate()
            .filter(|(i, x)| *i != 9)
            .map(|(i, x)| x.next())
            .sum()
    }
}

