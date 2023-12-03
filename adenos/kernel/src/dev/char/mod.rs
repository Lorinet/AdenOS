mod vga_textmode;
mod framebuffer_console;
mod uart_16550;
mod kernel_logger;
pub use vga_textmode::VgaTextMode;
pub use framebuffer_console::FramebufferConsole;
pub use self::uart_16550::Uart16550;
pub use kernel_logger::KernelLogger;