use cpal::SampleRate;

pub struct Echo {
    buffer: Vec<f32>,
    idx: usize,
    falloff: f32,
}

impl Echo {
    pub fn new(delay: f32, falloff: f32, sample_rate: SampleRate) -> Self {
        let capacity = (delay * sample_rate.0 as f32) as usize;
        let mut buff = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            buff.push(0.0);
        }
        Echo { buffer: buff, idx: 0, falloff }
    }

    pub fn next(&mut self, sample: f32) -> Option<f32> {
        let d = sample;
        let len = self.buffer.len();
        let w = self.buffer[(self.idx + 1) % len];
        self.buffer[(self.idx) % len] = (d + w) * self.falloff;

        self.idx += 1;

        return Some(d + w);
    }
}