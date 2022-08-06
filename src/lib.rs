pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const RAM_SIZE: usize = 4096;
const NUM_VREGS: usize = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;

const START_ADDR: u16 = 0x200;

pub struct Emulator {
    pc: u16,
    ram: [u8; RAM_SIZE],
    display: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    v_register: [u8; NUM_VREGS],
    i_register: u16,
    sp: u16,
    stack: [u16; STACK_SIZE],
    keys: [bool; NUM_KEYS],
    delay_timer: u8,
    sound_timer: u8,
}

impl Emulator {
    pub fn new() -> Self {
        Self { 
            pc: START_ADDR, 
            ram: [0; RAM_SIZE], 
            display: [false; SCREEN_WIDTH * SCREEN_HEIGHT], 
            v_register: [0; NUM_VREGS], 
            i_register: 0, 
            sp: 0, 
            stack: [0; STACK_SIZE], 
            keys: [false; NUM_KEYS], 
            delay_timer: 0, 
            sound_timer: 0 
        }
    }
}