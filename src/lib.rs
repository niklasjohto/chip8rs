use rand::{self, Rng};

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const RAM_SIZE: usize = 4096;
const NUM_VREGS: usize = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;

const START_ADDR: u16 = 0x200;

const FONTSET_SIZE: usize = 80;

const FONTSET: [u8; FONTSET_SIZE] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

pub struct Emulator {
    program_counter: u16,
    ram: [u8; RAM_SIZE],
    display: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    v_register: [u8; NUM_VREGS],
    i_register: u16,
    stack_pointer: u16,
    stack: [u16; STACK_SIZE],
    keys: [bool; NUM_KEYS],
    delay_timer: u8,
    sound_timer: u8,
}

impl Emulator {
    pub fn new() -> Self {
        let mut new_emulator = Self { 
            program_counter: START_ADDR, 
            ram: [0; RAM_SIZE], 
            display: [false; SCREEN_WIDTH * SCREEN_HEIGHT], 
            v_register: [0; NUM_VREGS], 
            i_register: 0, 
            stack_pointer: 0, 
            stack: [0; STACK_SIZE], 
            keys: [false; NUM_KEYS], 
            delay_timer: 0, 
            sound_timer: 0 
        };

        new_emulator.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);

        new_emulator
    }

    fn push(&mut self, value: u16) {
        self.stack[self.stack_pointer as usize] = value;
        self.stack_pointer += 1;
    }

    fn pop(&mut self) -> u16 {
        self.stack_pointer -= 1;
        self.stack[self.stack_pointer as usize]
    }

    pub fn reset(&mut self) {
        self.program_counter = START_ADDR;
        self.ram = [0; RAM_SIZE];
        self.display = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.v_register = [0; NUM_VREGS];
        self.i_register = 0;
        self.stack_pointer = 0;
        self.stack = [0; STACK_SIZE];
        self.keys = [false; NUM_KEYS];
        self.delay_timer = 0;
        self.sound_timer = 0;
        self.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
    }

    pub fn tick(&mut self) {
        let opcode = self.fetch();
        self.execute(opcode);
    }

    pub fn tick_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                // BEEP
            }

            self.sound_timer -= 1;
        }
    }

    fn fetch(&mut self) -> u16 {
        let higher_byte = self.ram[self.program_counter as usize] as u16;
        let lower_byte = self.ram[(self.program_counter + 1) as usize] as u16;
        let opcode = (higher_byte << 8) | lower_byte;
        self.program_counter += 2;
        opcode
    }

    fn execute(&mut self, opcode: u16) {
        let digit1 = (opcode & 0xF000) >> 12;
        let digit2 = (opcode & 0x0F00) >> 8;
        let digit3 = (opcode & 0x00F0) >> 4;
        let digit4 = opcode & 0x000F;

        match (digit1, digit2, digit3, digit4) {
            (0,0,0,0) => return,
            (0,0,0xE,0) => {
                self.display = [false; SCREEN_WIDTH * SCREEN_HEIGHT]
            },
            (0,0,0xE,0xE) => {
                let ret_addr = self.pop();
                self.program_counter = ret_addr;
            }
            (1,_,_,_) => {
                let nnn = opcode & 0xFFF;
                self.program_counter = nnn;
            },
            (2,_,_,_) => {
                let nnn = opcode & 0xFFF;
                self.push(self.program_counter);
                self.program_counter = nnn;
            },
            (3,_,_,_)  => {
                let ptr = digit2 as usize;
                let nn = (opcode & 0xFF) as u8;
                if self.v_register[ptr] == nn {
                    self.program_counter += 2;
                }
            },
            (4,_,_,_) => {
                let ptr = digit2 as usize;
                let nn = (opcode & 0xFF) as u8;
                if self.v_register[ptr] != nn {
                    self.program_counter += 2;
                }
            },
            (5,_,_,_) => {
                let ptr1 = digit2 as usize;
                let ptr2 = digit3 as usize;
                if self.v_register[ptr1] == self.v_register[ptr2] {
                    self.program_counter += 2;
                }
            },
            (6,_,_,_) => {
                let ptr = digit2 as usize;
                let nn = (opcode & 0xFF) as u8;
                self.v_register[ptr] = nn;
            },
            (7,_,_,_) => {
                let ptr = digit2 as usize;
                let nn = (opcode & 0xFF) as u8;
                self.v_register[ptr] = self.v_register[ptr].wrapping_add(nn);
            },
            (8,_,_,0) => {
                let ptr1 = digit2 as usize;
                let ptr2 = digit3 as usize;

                self.v_register[ptr1] = self.v_register[ptr2];
            },
            (8,_,_,1) => {
                let ptr1 = digit2 as usize;
                let ptr2 = digit3 as usize;

                self.v_register[ptr1] |= self.v_register[ptr2];
            },
            (8,_,_,2) => {
                let ptr1 = digit2 as usize;
                let ptr2 = digit3 as usize;

                self.v_register[ptr1] &= self.v_register[ptr2];
            },
            (8,_,_,3) => {
                let ptr1 = digit2 as usize;
                let ptr2 = digit3 as usize;

                self.v_register[ptr1] ^= self.v_register[ptr2];
            },
            (8,_,_,4) => {
                let ptr1 = digit2 as usize;
                let ptr2 = digit3 as usize;

                let (new_vx, carry) = self.v_register[ptr1].overflowing_add(self.v_register[ptr2]);
                let new_vf = if carry {1} else {0};

                self.v_register[ptr1] = new_vx;
                self.v_register[0xF] = new_vf;
            },
            (8,_,_,5) => {
                let ptr1 = digit2 as usize;
                let ptr2 = digit3 as usize;

                let (new_vx, borrow) = self.v_register[ptr1].overflowing_sub(self.v_register[ptr2]);
                let new_vf = if borrow {0} else {1};

                self.v_register[ptr1] = new_vx;
                self.v_register[0xF] = new_vf;
            },
            (8,_,_,6) => {
                let ptr = digit2 as usize;
                let dropped_bit = self.v_register[ptr] & 1;
                self.v_register[ptr] >>= 1;
                self.v_register[0xF] = dropped_bit;
            },
            (8,_,_,7) => {
                let ptr1 = digit2 as usize;
                let ptr2 = digit3 as usize;

                let (new_vx, borrow) = self.v_register[ptr2].overflowing_sub(self.v_register[ptr1]);
                let new_vf = if borrow {0} else {1};

                self.v_register[ptr1] = new_vx;
                self.v_register[0xF] = new_vf;
            },
            (8,_,_,0xE) => {
                let ptr = digit2 as usize;
                let dropped_bit = (self.v_register[ptr] >> 7) & 1;
                self.v_register[ptr] <<= 1;
                self.v_register[0xF] = dropped_bit;
            },
            (9,_,_,0) => {
                let ptr1 = digit2 as usize;
                let ptr2 = digit3 as usize;

                if self.v_register[ptr1] != self.v_register[ptr2] {
                    self.program_counter += 2;
                }
            },
            (0xA,_,_,_) => {
                let nnn = opcode & 0xFFF;
                self.i_register = nnn;
            },
            (0xB,_,_,_) => {
                let nnn = opcode & 0xFFF;
                self.program_counter = (self.v_register[0] as u16) + nnn;
            }
            (0xC,_,_,_) => {
                let ptr = digit2 as usize;
                let nn = (opcode & 0xFF) as u8;
                let rng: u8 = rand::thread_rng().gen();
                self.v_register[ptr] = rng & nn;
            },
            (0xD,_,_,_) => {
                let x_coord = self.v_register[digit2 as usize] as u16;
                let y_coord = self.v_register[digit3 as usize] as u16;

                let num_rows = digit4;

                let mut flipped = false;

                for y_line in 0..num_rows {
                    let addr = self.i_register + y_line as u16;
                    let pixels = self.ram[addr as usize];

                    for x_line in 0..8 {
                         if (pixels & (0b1000_0000 >> x_line)) != 0 {
                            let x = (x_coord + x_line) as usize % SCREEN_WIDTH;
                            let y = (y_coord + y_line) as usize % SCREEN_HEIGHT;

                            let index = x + SCREEN_HEIGHT * y;

                            flipped |= self.display[index];
                            self.display[index] ^= true;
                        } 
                    }
                }

                if flipped {
                   self.v_register[0xF] = 1;
                } else {
                    self.v_register[0xF] = 0;
                }
            },
            (0xE,_,9,0xE) => {
                let ptr = digit2 as usize;
                let vx = self.v_register[ptr];
                let key = self.keys[vx as usize];
                if key {
                    self.program_counter += 2;
                }
            },
            (0xE,_,0xA,1) => {
                let ptr = digit2 as usize;
                let vx = self.v_register[ptr];
                let key = self.keys[vx as usize];
                if !key {
                    self.program_counter += 2;
                }
            },
            (0xF,_,0,7) => {
                let ptr = digit2 as usize;
                self.v_register[ptr] = self.delay_timer;
            },
            (0xF,_,0,0xA) => {
                let ptr = digit2 as usize;
                let mut pressed = false;

                for i in 0..self.keys.len() {
                    if self.keys[i] {
                        self.v_register[ptr] = i as u8;
                        pressed = true;
                        break;
                    }
                }

                if !pressed {
                    self.program_counter -= 2;
                }
            },
            (0xF,_,1,5) => {
                let ptr = digit2 as usize;
                self.delay_timer = self.v_register[ptr];
            },
            (0xF,_,1,8) => {
                let ptr = digit2 as usize;
                self.sound_timer = self.v_register[ptr];
            },
            (0xF,_,1,0xE) => {
                let ptr = digit2 as usize;
                let vx = self.v_register[ptr] as u16;
                self.i_register = self.i_register.wrapping_add(vx);
            },
            (0xF,_,2,9) => {
                let ptr = digit2 as usize;
                let char = self.v_register[ptr] as u16;
                self.i_register = char * 5;
            },
            (0xF,_,3,3) => {
                let ptr = digit2 as usize;
                let vx = self.v_register[ptr] as f32;

                let hundres = (vx / 100.0).floor() as u8;
                let tens = ((vx / 10.0) % 10.0).floor() as u8;
                let ones = (vx % 10.0) as u8;
                
                self.ram[self.i_register as usize] = hundres;
                self.ram[(self.i_register + 1) as usize] = tens;
                self.ram[(self.i_register + 2) as usize] = ones;
            },
            (0xF,_,5,5) => {
                let ptr = digit2 as usize;
                let i = self.i_register as usize;

                for index in 0..=ptr {
                    self.ram[i + ptr] = self.v_register[index];
                }
            },
            (0xF,_,6,5) => {
                let ptr = digit2 as usize;
                let i = self.i_register as usize;

                for index in 0..=ptr {
                     self.v_register[index] = self.ram[i + ptr];
                }
            }
            (_,_,_,_) => unimplemented!("{:#04x}", opcode)
        }
    }

    pub fn get_display(&self) -> &[bool] {
        &self.display
    }

    pub fn keypress(&mut self, index: usize, pressed: bool) {
        self.keys[index] = pressed;
    }

    pub fn load(&mut self, data: &[u8]) {
        let start = START_ADDR as usize;
        let end = start + data.len();
        self.ram[start..end].copy_from_slice(data);
    }
}