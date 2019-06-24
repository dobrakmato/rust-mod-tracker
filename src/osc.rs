use cpal::SampleRate;

const TWO_PI: f32 = std::f32::consts::PI * 2.0;

pub trait Voice {
    #[inline]
    fn set_hz(&mut self, hz: f32);

    #[inline]
    fn get_hz(&self) -> f32;

    fn next(&mut self) -> Option<f32>;
}

/// Oscillator generating sine wave.
#[derive(Copy, Clone, Debug)]
pub struct SineOsc {
    hz: f32,
    sample_rate: SampleRate,
    step: u32,
}

impl SineOsc {
    pub fn new(sample_rate: SampleRate) -> Self {
        SineOsc {
            step: 0,
            sample_rate,
            hz: 0.0,
        }
    }
}

impl Voice for SineOsc {
    fn set_hz(&mut self, hz: f32) {
        self.hz = hz;
    }

    fn get_hz(&self) -> f32 {
        return self.hz;
    }

    fn next(&mut self) -> Option<f32> {
        let f = self.step as f32 / self.sample_rate.0 as f32;
        self.step += 1;
        Some((f * TWO_PI * self.hz).sin())
    }
}

/// Oscillator generating square wave.
#[derive(Copy, Clone, Debug)]
pub struct SquareOsc {
    hz: f32,
    sample_rate: SampleRate,
    step: u32,
}

impl SquareOsc {
    pub fn new(sample_rate: SampleRate) -> Self {
        SquareOsc {
            step: 0,
            sample_rate,
            hz: 0.0,
        }
    }
}

impl Voice for SquareOsc {
    fn set_hz(&mut self, hz: f32) {
        self.hz = hz;
    }

    fn get_hz(&self) -> f32 {
        return self.hz;
    }

    fn next(&mut self) -> Option<f32> {
        let period = self.sample_rate.0 as f32 / self.hz;
        let f = self.step as f32 / period;
        self.step += 1;

        if self.step > period as u32 {
            self.step = 1;
        }

        Some(if f < 0.5 { 0.5 } else { -0.5 })
    }
}


/// Oscillator generating square wave.
#[derive(Copy, Clone, Debug)]
pub struct SawOsc {
    hz: f32,
    sample_rate: SampleRate,
    step: u32,
}

impl SawOsc {
    pub fn new(sample_rate: SampleRate) -> Self {
        SawOsc {
            step: 0,
            sample_rate,
            hz: 0.0,
        }
    }
}

impl Voice for SawOsc {
    fn set_hz(&mut self, hz: f32) {
        self.hz = hz;
    }

    fn get_hz(&self) -> f32 {
        return self.hz;
    }

    fn next(&mut self) -> Option<f32> {
        let period = self.sample_rate.0 as f32 / self.hz;
        self.step += 1;

        if self.step > period as u32 {
            self.step = 0;
        }

        Some(self.step as f32 / period)
    }
}

/* in semitones */
const DETUNED_OFFSETS: [f32; 8] = [
    -0.1, 0.1, -0.2, 0.2, -0.3, 0.3, 0.05, -0.05
];

/* log(2) / 12 */
const LOG_2_OVER_12: f32 = 0.057762265046662105;

#[derive(Copy, Clone, Debug)]
pub struct DetunedSaw {
    hz: f32,
    osc: [SawOsc; 8],
}

impl DetunedSaw {
    pub fn new(sample_rate: SampleRate) -> Self {
        DetunedSaw {
            osc: [SawOsc::new(sample_rate); 8],
            hz: 0.0,
        }
    }
}

impl Voice for DetunedSaw {
    fn set_hz(&mut self, hz: f32) {
        self.hz = hz;
    }

    fn get_hz(&self) -> f32 {
        return self.hz;
    }

    fn next(&mut self) -> Option<f32> {
        let mut sum = 0.0;

        for i in 0..8 {
            let detuned = (DETUNED_OFFSETS[i] * LOG_2_OVER_12).exp() * self.hz;

            self.osc[i].hz = detuned;
            sum += self.osc[i].next().unwrap();
        }

        Some(sum)
    }
}

#[derive(Copy, Clone)]
pub struct Combined<U, V> {
    a: U,
    b: V,
}

impl<U, V> Combined<U, V> where U: Voice, V: Voice {
    pub fn new(a: U, b: V) -> Self {
        return Combined {
            a,
            b,
        };
    }
}

impl<U, V> Voice for Combined<U, V> where U: Voice, V: Voice {
    fn set_hz(&mut self, hz: f32) {
        self.a.set_hz(hz);
        self.b.set_hz(hz);
    }

    fn get_hz(&self) -> f32 {
        return self.a.get_hz();
    }

    fn next(&mut self) -> Option<f32> {
        return Some(self.a.next().unwrap() + self.b.next().unwrap());
    }
}