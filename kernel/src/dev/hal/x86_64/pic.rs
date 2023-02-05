use crate::{*, dev::{Read, Write}};
use super::interrupts;
use dev::hal::{cpu, port};

const PIC_MASTER_PORT: u16 = 0x20;
const PIC_SLAVE_PORT: u16 = 0xA0;

const PIC_MASTER_DISABLE_PORT: u16 = 0xa1;
const PIC_SLAVE_DISABLE_PORT: u16 = 0x21;

const WAIT_PORT: u16 = 0x11;

const ICW1_ICW4: u8 = 0x01;
const ICW1_INIT: u8 = 0x10;
const ICW4_8086: u8 = 0x01;

pub const PIC_MASTER_OFFSET: u8 = 0x20;
pub const PIC_SLAVE_OFFSET: u8 = PIC_MASTER_OFFSET + 0x08;

const END_OF_INTERRUPT: u8 = 0x20;

pub fn init() {
    let mut master_cmd: port::Port = port::Port::new(PIC_MASTER_PORT);
    let mut master_data: port::Port = port::Port::new(PIC_MASTER_PORT + 1);
    let mut slave_cmd: port::Port = port::Port::new(PIC_SLAVE_PORT);
    let mut slave_data: port::Port = port::Port::new(PIC_SLAVE_PORT + 1);
    let mut wait_port: port::Port = port::Port::new(WAIT_PORT);
    let mut wait = || write_one!(wait_port, 0).unwrap();
    let a1 = read_one!(master_data).unwrap();
    let a2 = read_one!(slave_data).unwrap();
    cpu::atomic_no_interrupts(|| {
        write_one!(master_cmd, ICW1_INIT + ICW1_ICW4).unwrap();
        wait();
        write_one!(slave_cmd, ICW1_INIT + ICW1_ICW4).unwrap();
        wait();
        write_one!(master_data, PIC_MASTER_OFFSET).unwrap();
        wait();
        write_one!(slave_data, PIC_SLAVE_OFFSET).unwrap();
        wait();
        write_one!(master_data, 4).unwrap();
        wait();
        write_one!(slave_data, 2).unwrap();
        wait();
        write_one!(master_data, ICW4_8086).unwrap();
        wait();
        write_one!(slave_data, ICW4_8086).unwrap();
        wait();
        write_one!(master_data, a1).unwrap();
        write_one!(slave_data, a2).unwrap();
    });
    // mask everything except timer and keyboard
    for _i in 2..16 {
        //mask(i);
    }
    let mut pit_cmd: port::Port = port::Port::new(0x43);
    let mut pit_data: port::Port = port::Port::new(0x40);
    let divisor: u16 = (1193180 / 100) as u16;
    cpu::atomic_no_interrupts(|| {
        write_one!(pit_cmd, 0x36).unwrap();
        write_one!(pit_data, (divisor & 0xFF) as u8).unwrap();
        write_one!(pit_data, ((divisor >> 8) & 0xFF) as u8).unwrap();
    });
}

pub fn mask(irq: u8) {
    let mut port: port::Port;
    let mut irq = irq;
    if irq < 8 {
        port = port::Port::new(PIC_MASTER_PORT + 1);
    } else {
        port = port::Port::new(PIC_SLAVE_PORT + 1);
        irq -= 8;
    }
    let val = read_one!(port).unwrap() | (1 << irq);
    write_one!(port, val).unwrap();
}

pub fn unmask(irq: u8) {
    let mut port: port::Port;
    let mut irq = irq;
    if irq < 8 {
        port = port::Port::new(PIC_MASTER_PORT + 1);
    } else {
        port = port::Port::new(PIC_SLAVE_PORT + 1);
        irq -= 8;
    }
    let val = read_one!(port).unwrap() & !(1 << irq);
    write_one!(port, val).unwrap();
}

pub fn deinit() {
    let mut master_cmd: port::Port = port::Port::new(PIC_MASTER_DISABLE_PORT);
    let mut slave_cmd: port::Port = port::Port::new(PIC_SLAVE_DISABLE_PORT);
    write_one!(master_cmd, 0xff);
    write_one!(slave_cmd, 0xff);
}

pub fn end_of_interrupt(interrupt_id: interrupts::HardwareInterrupt) {
    if (interrupt_id as u8) >= PIC_SLAVE_OFFSET && (interrupt_id as u8) < PIC_SLAVE_OFFSET + 8 {
        write_one!(port::Port::new(PIC_SLAVE_PORT), END_OF_INTERRUPT).unwrap();
    }
    write_one!(port::Port::new(PIC_MASTER_PORT), END_OF_INTERRUPT).unwrap();
}