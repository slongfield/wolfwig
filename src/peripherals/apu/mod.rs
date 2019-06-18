///! Model of the Audio Processing Unit

pub struct Sweep {
    time: u8,
    direction: bool,
    // This is described in all the documentation I read as "number of sweep shift", whatever the
    // heck that means.
    // TODO(slongfield): Figure out whatever the heck that means.
    shift: u8,
}

impl Sweep {
    pub fn new() -> Self {
        Self {
            time: 0,
            direction: false,
            shift: 0,
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
        self.time = val
    }
    pub fn set_direction(&mut self, val: u8) {
        self.direction = val != 0
    }
    pub fn set_shift(&mut self, val: u8) {
        self.shift = val
    }
}

pub struct LengthPattern {
    // Duty cycle, ranges from 0-4 (12.5%, 25%, 50%, 75%)
    duty: u8,
    // Lengths, in units of 1/256ths of a second
    length: u8,
}

impl LengthPattern {
    pub fn new() -> Self {
        Self { duty: 0, length: 0 }
    }
    pub fn duty(&self) -> u8 {
        self.duty
    }
    pub fn length(&self) -> u8 {
        self.length
    }
    pub fn set_duty(&mut self, val: u8) {
        self.duty = val
    }
    pub fn set_length(&mut self, val: u8) {
        self.length = val
    }
}

pub struct Envelope {
    initial_volume: u8,
    direction: bool,
    sweep: u8,
}

impl Envelope {
    pub fn new() -> Self {
        Self {
            initial_volume: 0,
            direction: false,
            sweep: 0,
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
        self.initial_volume = val
    }
    pub fn set_direction(&mut self, val: u8) {
        self.direction = val != 0
    }
    pub fn set_sweep(&mut self, val: u8) {
        self.sweep = val
    }
}

pub struct Frequency {
    frequency: u16,
    start: bool,
    use_counter: bool,
}

impl Frequency {
    pub fn new() -> Self {
        Self {
            frequency: 0,
            start: false,
            use_counter: false,
        }
    }

    pub fn frequency_low(&self) -> u8 {
        self.frequency.to_be_bytes()[0]
    }
    pub fn frequency_high(&self) -> u8 {
        self.frequency.to_be_bytes()[1]
    }
    pub fn start(&self) -> u8 {
        self.start as u8
    }
    pub fn use_counter(&self) -> u8 {
        self.use_counter as u8
    }
    pub fn set_frequency_low(&mut self, val: u8) {
        let mut bytes = self.frequency.to_be_bytes();
        bytes[0] = val;
        self.frequency = u16::from_be_bytes(bytes);
    }
    pub fn set_frequency_high(&mut self, val: u8) {
        let mut bytes = self.frequency.to_be_bytes();
        bytes[1] = val;
        self.frequency = u16::from_be_bytes(bytes);
    }
    pub fn set_start(&mut self, val: u8) {
        self.start = val != 0
    }
    pub fn set_use_counter(&mut self, val: u8) {
        self.use_counter = val != 0
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
    pub fn new() -> Self {
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
}

impl ChannelOne {
    pub fn new() -> Self {
        Self {
            sweep: Sweep::new(),
            length_pattern: LengthPattern::new(),
            envelope: Envelope::new(),
            frequency: Frequency::new(),
        }
    }
}

pub struct ChannelTwo {
    pub length_pattern: LengthPattern,
    pub envelope: Envelope,
    pub frequency: Frequency,
}

impl ChannelTwo {
    pub fn new() -> Self {
        Self {
            length_pattern: LengthPattern::new(),
            envelope: Envelope::new(),
            frequency: Frequency::new(),
        }
    }
}

pub struct ChannelThree {
    pub enable: bool,
    pub length: u8,
    pub level: u8,
    pub frequency: Frequency,
    pub table: Vec<u8>,
}

impl ChannelThree {
    const TABLE_SIZE: usize = 16;

    pub fn new() -> Self {
        Self {
            enable: false,
            length: 0,
            level: 0,
            frequency: Frequency::new(),
            table: vec![0; Self::TABLE_SIZE],
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
}

/// Channel Four is the noise channel, usually used for snares or other percussion.
pub struct ChannelFour {
    pub length: u8,
    pub envelope: Envelope,
    pub counter: PolyCounter,
    pub start: bool,
    pub stop_on_length: bool,
}

impl ChannelFour {
    pub fn new() -> Self {
        Self {
            length: 0,
            envelope: Envelope::new(),
            counter: PolyCounter::new(),
            start: false,
            stop_on_length: false,
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
}

pub struct Apu {
    pub channel_one: ChannelOne,
    pub channel_two: ChannelTwo,
    pub channel_three: ChannelThree,
    pub channel_four: ChannelFour,
}

impl Apu {
    pub fn new() -> Self {
        Self {
            channel_one: ChannelOne::new(),
            channel_two: ChannelTwo::new(),
            channel_three: ChannelThree::new(),
            channel_four: ChannelFour::new(),
        }
    }

    pub fn step(&self) {}
}
