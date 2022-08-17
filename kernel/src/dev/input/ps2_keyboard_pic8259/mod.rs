use crate::*;
use dev;
use dev::Read;
use dev::hal::{cpu, pic, port, interrupts};
use dev::input::keyboard;
use async_task::*;
use x86_64::structures::idt;
use conquer_once::spin::OnceCell;
use crossbeam_queue::ArrayQueue;
use core::{pin::Pin, task::{Poll, Context}, arch::asm};
use futures_util::stream::Stream;
use futures_util::stream::StreamExt;
use futures_util::task::AtomicWaker;

mod scancodes;

static mut KEYBOARD_HANDLER: Option<fn(keyboard::Key)> = None;
static mut KEYBOARD_PORT: port::Port<u8> = port::Port::new(0x60);

static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();
static WAKER: AtomicWaker = AtomicWaker::new();

pub struct PS2KeyboardPIC8259 {}

impl PS2KeyboardPIC8259 {
    pub fn set_input_handler(input_handler: fn(keyboard::Key)) {
        unsafe {
            KEYBOARD_HANDLER = Some(input_handler);
        }
    }

    pub extern "x86-interrupt" fn _input_handler(_stack_frame: idt::InterruptStackFrame) {
        unsafe {
            asm!("cli");
            Self::add_scancode(KEYBOARD_PORT.read_one().unwrap());
        }
        pic::end_of_interrupt(interrupts::HardwareInterrupt::Keyboard);
    }

    fn add_scancode(scancode: u8) {
        if let Ok(queue) = SCANCODE_QUEUE.try_get() {
            if let Err(_) = queue.push(scancode) {
                println!("WARNING: scancode queue full; dropping keyboard input");
            } else {
                WAKER.wake();
            }
        } else {
            println!("WARNING: scancode queue uninitialized");
        }
    }

    async fn _input_handler_task() {
        let mut scancodes = ScancodeStream::new();
        if let Some(handler) = unsafe { KEYBOARD_HANDLER } {
            while let Some(scancode) = scancodes.next().await {
                if let Ok(Some(key_event)) = scancodes::add_byte(scancode) {
                    if let Some(key) = scancodes::process_keyevent(key_event) {
                        handler(key);
                    }
                }
            }
            
        }
    }
}

impl dev::StaticDevice for PS2KeyboardPIC8259 {
    fn device_name() -> &'static str {
        "Input/PS2KeyboardPIC8259"
    }

    fn init_device() -> Result<(), dev::Error> {
        println!("Initializing {}...", PS2KeyboardPIC8259::device_name());
        kernel_executor::spawn(Task::new(Self::_input_handler_task()));
        cpu::register_interrupt_handler(interrupts::HardwareInterrupt::Keyboard, PS2KeyboardPIC8259::_input_handler);
        Ok(())
    }
}

pub struct ScancodeStream {
    _private: (),
}

impl ScancodeStream {
    pub fn new() -> Self {
        SCANCODE_QUEUE.try_init_once(|| ArrayQueue::new(100))
            .expect("ScancodeStream::new should only be called once");
        ScancodeStream { _private: () }
    }
}

impl Stream for ScancodeStream {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<u8>> {
        let queue = SCANCODE_QUEUE
            .try_get()
            .expect("scancode queue not initialized");

        // fast path
        if let Ok(scancode) = queue.pop() {
            return Poll::Ready(Some(scancode));
        }

        WAKER.register(&cx.waker());
        match queue.pop() {
            Ok(scancode) => {
                WAKER.take();
                Poll::Ready(Some(scancode))
            }
            Err(crossbeam_queue::PopError) => Poll::Pending,
        }
    }
}