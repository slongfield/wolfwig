use std::cmp::min;
///! Model of the Audio Processing Unit
use std::collections::VecDeque;
use std::time;

pub struct Sweep {
    time: u8,
    direction: bool,
    // This is described in all the documentation I read as "number of sweep shift", whatever the
    // heck that means.
    // TODO(slongfield): Figure out whatever the heck that means.
    shift: u8,
    modified: bool,
}

impl Sweep {
    fn new() -> Self {
        Self {
            time: 0,
            direction: false,
            shift: 0,
            modified: false,
        }
    }
    pub fn time(&self) -> u8 {
        self.time
    }
    pub fn direction(&self) -> u8 {
        self.direction as u8
    }
    pub fn shift(&self) -> u8 {
        self.shift as u8
    }
    pub fn set_time(&mut self, val: u8) {
        self.time = val;
        self.modified = true
    }
    pub fn set_direction(&mut self, val: u8) {
        self.direction = val != 0;
        self.modified = true
    }
    pub fn set_shift(&mut self, val: u8) {
        self.shift = val;
        self.modified = true
    }
}

pub struct LengthPattern {
    // Duty cycle, ranges from 0-4 (12.5%, 25%, 50%, 75%)
    duty: u8,
    // Lengths, in units of 1/64ths of a second
    length: u8,
    // How much of the length has been played out.
    length_sec: f32,
    played_length: f32,
    modified: bool,
}

impl LengthPattern {
    fn new() -> Self {
        Self {
            duty: 0,
            length: 0,
            length_sec: 0.0,
            played_length: 1000.0,
            modified: false,
        }
    }
    pub fn duty(&self) -> u8 {
        self.duty
    }
    pub fn length(&self) -> u8 {
        self.length
    }
    pub fn set_duty(&mut self, val: u8) {
        self.duty = val;
        self.modified = true
    }
    pub fn set_length(&mut self, val: u8) {
        self.length = val;
        self.length_sec = (64.0 - val as f32) / 256.0;
        self.played_length = 0.0;
        self.modified = true
    }
    fn duty_cycle(&self) -> f32 {
        match self.duty {
            0 => 0.125,
            1 => 0.25,
            2 => 0.5,
            _ => 0.75,
        }
    }
}

pub struct Envelope {
    initial_volume: u8,
    direction: bool,
    sweep: u8,
    modified: bool,
}

impl Envelope {
    fn new() -> Self {
        Self {
            initial_volume: 0,
            direction: false,
            sweep: 0,
            modified: false,
        }
    }

    pub fn initial_volume(&self) -> u8 {
        self.initial_volume
    }
    pub fn direction(&self) -> u8 {
        self.direction as u8
    }
    pub fn sweep(&self) -> u8 {
        self.sweep
    }
    pub fn set_initial_volume(&mut self, val: u8) {
        self.initial_volume = val;
        self.modified = true
    }
    pub fn set_direction(&mut self, val: u8) {
        self.direction = val != 0;
        self.modified = true
    }
    pub fn set_sweep(&mut self, val: u8) {
        self.sweep = val;
        self.modified = true
    }
}

pub struct Frequency {
    frequency: u16,
    start: bool,
    use_counter: bool,
    modified: bool,
}

impl Frequency {
    fn new() -> Self {
        Self {
            frequency: 0,
            start: false,
            use_counter: false,
            modified: false,
        }
    }

    pub fn frequency_low(&self) -> u8 {
        self.frequency.to_be_bytes()[1]
    }
    pub fn frequency_high(&self) -> u8 {
        self.frequency.to_be_bytes()[0]
    }
    pub fn start(&self) -> u8 {
        self.start as u8
    }
    pub fn use_counter(&self) -> u8 {
        self.use_counter as u8
    }
    pub fn hz(&self) -> f32 {
        131072.0 / (2048.0 - self.frequency as f32)
    }
    pub fn set_frequency_low(&mut self, val: u8) {
        self.frequency &= !0xff;
        self.frequency |= u16::from(val);
        self.modified = true;
    }
    pub fn set_frequency_high(&mut self, val: u8) {
        self.frequency &= 0xff;
        self.frequency |= u16::from(val) << 8;
        self.modified = true;
    }
    pub fn set_start(&mut self, val: u8) {
        self.start = val != 0;
        self.modified = true;
    }
    pub fn set_use_counter(&mut self, val: u8) {
        self.use_counter = val != 0;
        self.modified = true;
    }
}

// Polynomial counter, used to produce noise
pub struct PolyCounter {
    pub frequency: u8,
    // false = 15 bits, true = 7 bits
    pub width: bool,
    pub ratio: u8,
}

impl PolyCounter {
    fn new() -> Self {
        Self {
            frequency: 0,
            width: false,
            ratio: 0,
        }
    }

    pub fn set_frequency(&mut self, val: u8) {
        self.frequency = val
    }

    pub fn set_width(&mut self, val: u8) {
        self.width = val != 0
    }

    pub fn set_ratio(&mut self, val: u8) {
        self.ratio = val
    }

    pub fn frequency(&self) -> u8 {
        self.frequency
    }

    pub fn width(&self) -> u8 {
        self.width as u8
    }

    pub fn ratio(&self) -> u8 {
        self.ratio
    }
}

pub struct ChannelOne {
    pub sweep: Sweep,
    pub length_pattern: LengthPattern,
    pub envelope: Envelope,
    pub frequency: Frequency,
    phase: f32,
    active: bool,
}

impl ChannelOne {
    fn new() -> Self {
        Self {
            sweep: Sweep::new(),
            length_pattern: LengthPattern::new(),
            envelope: Envelope::new(),
            frequency: Frequency::new(),
            phase: 0.0,
            active: false,
        }
    }

    pub fn active(&self) -> u8 {
        self.active as u8
    }

    fn get_samples(&mut self, nsamples: usize, device_freq: f32) -> Vec<f32> {
        let mut samples = vec![];
        if self.frequency.start {
            self.length_pattern.played_length = 0.0;
            self.frequency.start = false;
            if !self.frequency.use_counter {
                self.length_pattern.length_sec = 1000.0
            }
        }
        if self.length_pattern.played_length >= self.length_pattern.length_sec {
            for _ in 0..nsamples {
                samples.push(0.0)
            }
            return samples;
        }
        let phase_inc = self.frequency.hz() / device_freq;
        if self.frequency.modified || self.length_pattern.modified {
            debug!(
                "CH1: Playing {} hz tone for {} seconds? {}",
                self.frequency.hz(),
                self.length_pattern.length_sec,
                self.frequency.use_counter
            );
            self.frequency.modified = false;
            self.length_pattern.modified = false;
        }
        for _ in 0..nsamples {
            if self.phase <= self.length_pattern.duty_cycle() {
                samples.push(1.0);
            } else {
                samples.push(0.0);
            }
            self.phase = (self.phase + phase_inc) % 1.0;
        }
        self.length_pattern.played_length += (nsamples as f32) / device_freq;
        samples
    }
}

pub struct ChannelTwo {
    pub length_pattern: LengthPattern,
    pub envelope: Envelope,
    pub frequency: Frequency,
    phase: f32,
    active: bool,
}

impl ChannelTwo {
    fn new() -> Self {
        Self {
            length_pattern: LengthPattern::new(),
            envelope: Envelope::new(),
            frequency: Frequency::new(),
            phase: 0.0,
            active: false,
        }
    }

    pub fn active(&self) -> u8 {
        self.active as u8
    }

    fn get_samples(&mut self, nsamples: usize, device_freq: f32) -> Vec<f32> {
        let mut samples = vec![];
        if self.frequency.start {
            self.length_pattern.played_length = 0.0;
            self.frequency.start = false;
            if !self.frequency.use_counter {
                self.length_pattern.length_sec = 1000.0
            }
        }
        if self.length_pattern.played_length >= self.length_pattern.length_sec {
            for _ in 0..nsamples {
                samples.push(0.0)
            }
            return samples;
        }
        let phase_inc = self.frequency.hz() / device_freq;
        if self.frequency.modified || self.length_pattern.modified {
            debug!(
                "CH2: Playing {} hz tone for {} seconds? {}",
                self.frequency.hz(),
                self.length_pattern.length_sec,
                self.frequency.use_counter
            );
            self.frequency.modified = false;
            self.length_pattern.modified = false;
        }
        for _ in 0..nsamples {
            if self.phase <= self.length_pattern.duty_cycle() {
                samples.push(1.0);
            } else {
                samples.push(0.0);
            }
            self.phase = (self.phase + phase_inc) % 1.0;
        }
        self.length_pattern.played_length += (nsamples as f32) / device_freq;
        samples
    }
}

pub struct ChannelThree {
    pub enable: bool,
    pub length: u8,
    pub level: u8,
    pub frequency: Frequency,
    pub table: Vec<u8>,
    active: bool,
}

impl ChannelThree {
    const TABLE_SIZE: usize = 16;

    fn new() -> Self {
        Self {
            enable: false,
            length: 0,
            level: 0,
            frequency: Frequency::new(),
            table: vec![0; Self::TABLE_SIZE],
            active: false,
        }
    }

    pub fn set_enable(&mut self, val: u8) {
        self.enable = val != 0;
    }

    pub fn set_length(&mut self, val: u8) {
        self.length = val;
    }

    pub fn set_level(&mut self, val: u8) {
        self.level = val;
    }

    pub fn set_table(&mut self, offset: usize, val: u8) {
        if let Some(old) = self.table.get_mut(offset) {
            *old = val;
        }
    }

    pub fn enable(&self) -> u8 {
        self.enable as u8
    }

    pub fn length(&self) -> u8 {
        self.length
    }

    pub fn level(&self) -> u8 {
        self.level
    }

    pub fn table(&self, offset: usize) -> u8 {
        if let Some(&val) = self.table.get(offset) {
            val
        } else {
            0xFF
        }
    }

    pub fn active(&self) -> u8 {
        self.active as u8
    }
}

/// Channel Four is the noise channel, usually used for snares or other percussion.
pub struct ChannelFour {
    pub length: u8,
    pub envelope: Envelope,
    pub counter: PolyCounter,
    pub start: bool,
    pub stop_on_length: bool,
    active: bool,
}

impl ChannelFour {
    fn new() -> Self {
        Self {
            length: 0,
            envelope: Envelope::new(),
            counter: PolyCounter::new(),
            start: false,
            stop_on_length: false,
            active: false,
        }
    }

    pub fn set_length(&mut self, val: u8) {
        self.length = val
    }

    pub fn set_start(&mut self, val: u8) {
        self.start = val != 0
    }

    pub fn set_stop_on_length(&mut self, val: u8) {
        self.stop_on_length = val != 0
    }

    pub fn length(&self) -> u8 {
        self.length
    }

    pub fn stop_on_length(&self) -> u8 {
        self.stop_on_length as u8
    }

    pub fn active(&self) -> u8 {
        self.active as u8
    }
}

pub struct Volume {
    pub left: u8,
    pub right: u8,
}

impl Volume {
    fn new() -> Self {
        Self { left: 0, right: 0 }
    }

    pub fn set_left(&mut self, val: u8) {
        self.left = val
    }

    pub fn set_right(&mut self, val: u8) {
        self.right = val
    }

    pub fn left(&self) -> u8 {
        self.left
    }

    pub fn right(&self) -> u8 {
        self.right
    }
}

bitflags! {
    pub struct ChannelEnable: u8 {
        const CH4_LEFT  = 0b1000_0000;
        const CH3_LEFT  = 0b0100_0000;
        const CH2_LEFT  = 0b0010_0000;
        const CH1_LEFT  = 0b0001_0000;
        const CH4_RIGHT = 0b0000_1000;
        const CH3_RIGHT = 0b0000_0100;
        const CH2_RIGHT = 0b0000_0010;
        const CH1_RIGHT = 0b0000_0001;
    }
}

impl ChannelEnable {
    fn new() -> Self {
        Self::empty()
    }

    pub fn set_enable(&mut self, val: u8) {
        self.remove(Self::all());
        self.insert(Self::from_bits_truncate(val));
    }

    pub fn enable(&self) -> u8 {
        self.bits()
    }
}

pub struct Control {
    pub volume: Volume,
    pub channel_enable: ChannelEnable,
    pub enable: bool,
}

impl Control {
    fn new() -> Self {
        Self {
            volume: Volume::new(),
            channel_enable: ChannelEnable::new(),
            enable: false,
        }
    }

    pub fn set_enable(&mut self, val: u8) {
        self.enable = val != 0;
    }

    pub fn enable(&self) -> u8 {
        self.enable as u8
    }
}

struct APUSamples {
    pub left: VecDeque<f32>,
    pub right: VecDeque<f32>,
    pub device_freq: f32,
    update_interval: time::Duration,
    update_samples: usize,
}

impl sdl2::audio::AudioCallback for APUSamples {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        for sample in out.iter_mut().step_by(2) {
            if let Some(val) = self.left.pop_front() {
                *sample = val;
            } else {
                *sample = 0.0;
            }
        }
        for sample in out.iter_mut().skip(1).step_by(2) {
            if let Some(val) = self.right.pop_front() {
                *sample = val;
            } else {
                *sample = 0.0;
            }
        }
    }
}

pub struct Apu {
    pub channel_one: ChannelOne,
    pub channel_two: ChannelTwo,
    pub channel_three: ChannelThree,
    pub channel_four: ChannelFour,
    pub control: Control,
    device: Option<sdl2::audio::AudioDevice<APUSamples>>,
    last_update: time::Instant,
}

impl Apu {
    pub fn new(audio: sdl2::AudioSubsystem) -> Self {
        let desired_spec = sdl2::audio::AudioSpecDesired {
            freq: Some(44100),
            channels: Some(2),
            samples: None,
        };

        let device = audio
            .open_playback(None, &desired_spec, |spec| APUSamples {
                left: VecDeque::new(),
                right: VecDeque::new(),
                device_freq: spec.freq as f32,
                update_interval: time::Duration::from_micros(
                    u64::from(spec.samples) * 1_000_000 / (spec.freq as u64),
                ),
                update_samples: usize::from(spec.samples),
            })
            .unwrap();
        device.resume();

        Self {
            channel_one: ChannelOne::new(),
            channel_two: ChannelTwo::new(),
            channel_three: ChannelThree::new(),
            channel_four: ChannelFour::new(),
            control: Control::new(),
            device: Some(device),
            last_update: time::Instant::now(),
        }
    }

    pub fn new_fake() -> Self {
        Self {
            channel_one: ChannelOne::new(),
            channel_two: ChannelTwo::new(),
            channel_three: ChannelThree::new(),
            channel_four: ChannelFour::new(),
            control: Control::new(),
            device: None,
            last_update: time::Instant::now(),
        }
    }

    pub fn step(&mut self) {
        if let Some(ref mut device) = self.device {
            let mut samples = device.lock();
            if time::Instant::now().duration_since(self.last_update) > samples.update_interval {
                self.last_update = time::Instant::now();
                while samples.right.len() < 2 * samples.update_samples {
                    let mut channel_one_samples = self
                        .channel_one
                        .get_samples(samples.update_samples, samples.device_freq);
                    let mut channel_two_samples = self
                        .channel_two
                        .get_samples(samples.update_samples, samples.device_freq);
                    for i in 0..samples.update_samples {
                        let mut left_sample = 0.0;
                        let mut right_sample = 0.0;
                        if self
                            .control
                            .channel_enable
                            .contains(ChannelEnable::CH1_LEFT)
                        {
                            left_sample += 0.25 * channel_one_samples[i];
                        }
                        if self
                            .control
                            .channel_enable
                            .contains(ChannelEnable::CH2_LEFT)
                        {
                            left_sample += 0.25 * channel_two_samples[i];
                        }
                        if self
                            .control
                            .channel_enable
                            .contains(ChannelEnable::CH1_RIGHT)
                        {
                            right_sample += 0.25 * channel_one_samples[i];
                        }
                        if self
                            .control
                            .channel_enable
                            .contains(ChannelEnable::CH2_RIGHT)
                        {
                            right_sample += 0.25 * channel_two_samples[i];
                        }
                        samples.left.push_back(left_sample);
                        samples.right.push_back(right_sample);
                    }
                }
            }
        }
    }
}
