pub mod keyboard;

mod ps2_keyboard_pic8259;

#[cfg(target_arch = "x86_64")]
pub use ps2_keyboard_pic8259::PS2KeyboardPIC8259;