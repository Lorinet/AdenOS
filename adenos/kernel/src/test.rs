use crate::*;
use dev::power;
use dev::PowerControl;

pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        (**test).run();
    }
    let mut exit = power::QemuExit::new();
    exit.shutdown();
}

pub trait Testable {
    fn run(&self);
}

impl<T> Testable for T
where T: Fn() + ?Sized {
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}
