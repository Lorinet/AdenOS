use crate::*;
use dev::ConsoleDevice;

#[test_case]
fn test_println_simple() {
    println!("test_println_simple output");
}

#[test_case]
fn test_println_many() {
    for _ in 0..200 {
        println!("test_println_many output");
    }
}

#[test_case]
fn test_println_output() {
    kernel_console::KERNEL_CONSOLE.lock().clear_screen();
    let s = "Some test string that fits on a single line";
    print!("{}", s);
    for (i, c) in s.chars().enumerate() {
        let pos = (i * 2) as isize;
        let screen_char = unsafe { *kernel_console::KERNEL_CONSOLE.lock().buffer.offset(pos) };
        let screen_char = char::from(screen_char);
        assert_eq!(screen_char, c);
    }
}