use crate::*;
use dev;
use dev::Read;
use dev::hal::cpu;
use dev::hal::port;
use dev::input::keyboard;
use x86_64::structures::idt;

mod scancodes;

static mut KEYBOARD_HANDLER: Option<fn(keyboard::Key)> = None;
static mut KEYBOARD_PORT: port::Port<u8> = port::Port::new(0x60);

pub struct PS2KeyboardPIC8259 {}

impl PS2KeyboardPIC8259 {
    pub fn set_input_handler(input_handler: fn(keyboard::Key)) {
        unsafe {
            KEYBOARD_HANDLER = Some(input_handler);
        }
    }

    pub extern "x86-interrupt" fn _input_handler(_stack_frame: idt::InterruptStackFrame) {
        unsafe {
            if let Some(handler) = KEYBOARD_HANDLER {
                if let Ok(Some(key_event)) = scancodes::add_byte(KEYBOARD_PORT.read_one().unwrap()) {
                    if let Some(key) = scancodes::process_keyevent(key_event) {
                        handler(key);
                    }
                }
            }
        }
        cpu::pic_end_of_interrupt(cpu::HardwareInterrupt::Keyboard);
    }
}

impl dev::StaticDevice for PS2KeyboardPIC8259 {
    fn device_name() -> &'static str {
        "Input/PS2KeyboardPIC8259"
    }

    fn init_device() -> Result<(), dev::Error> {
        cpu::register_interrupt_handler(cpu::HardwareInterrupt::Keyboard, PS2KeyboardPIC8259::_input_handler);
        Ok(())
    }
}
