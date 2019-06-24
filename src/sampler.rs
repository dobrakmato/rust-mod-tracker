use cpal::SampleRate;

/// Structure representing an audio sample with associated
/// sample rate and channels information.
#[derive(Debug)]
pub struct Sample<'a> {
    buffer: &'a [f32],
    sample_rate: SampleRate,
    channels: u8,
}

impl<'a> Sample<'a> {
    pub fn new(buffer: &'a [f32], sample_rate: SampleRate, channels: u8) -> Self {
        return Sample { buffer, sample_rate, channels };
    }
}

/// Signal source that generates the audio by playing a sample.
pub struct Sampler<'a> {
    sample: &'a Sample<'a>,
    step: usize,
    pub looping: bool,
}

impl<'a> Sampler<'a> {
    pub fn new(sample: &'a Sample) -> Self {
        Sampler {
            sample,
            step: 0,
            looping: true,
        }
    }
}

impl<'a> Iterator for Sampler<'a> {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.looping && self.step >= self.sample.buffer.len() {
            return Some(0.0);
        }

        let value = self.sample.buffer[self.step];
        self.step += 1;

        if self.looping && self.step >= self.sample.buffer.len() {
            self.step = 0;
        }

        return Some(value);
    }
}