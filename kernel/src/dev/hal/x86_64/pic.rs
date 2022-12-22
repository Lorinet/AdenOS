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
    let mut wait = || wait_port.write_one(0).unwrap();
    let a1 = master_data.read_one().unwrap();
    let a2 = slave_data.read_one().unwrap();
    cpu::atomic_no_interrupts(|| {
        master_cmd.write_one(ICW1_INIT + ICW1_ICW4).unwrap();
        wait();
        slave_cmd.write_one(ICW1_INIT + ICW1_ICW4).unwrap();
        wait();
        master_data.write_one(PIC_MASTER_OFFSET).unwrap();
        wait();
        slave_data.write_one(PIC_SLAVE_OFFSET).unwrap();
        wait();
        master_data.write_one(4).unwrap();
        wait();
        slave_data.write_one(2).unwrap();
        wait();
        master_data.write_one(ICW4_8086).unwrap();
        wait();
        slave_data.write_one(ICW4_8086).unwrap();
        wait();
        master_data.write_one(a1).unwrap();
        slave_data.write_one(a2).unwrap();
    });
    // mask everything except timer and keyboard
    for _i in 2..16 {
        //mask(i);
    }
    let mut pit_cmd: port::Port = port::Port::new(0x43);
    let mut pit_data: port::Port = port::Port::new(0x40);
    let divisor: u16 = (1193180 / 100) as u16;
    cpu::atomic_no_interrupts(|| {
        pit_cmd.write_one(0x36).unwrap();
        pit_data.write_one((divisor & 0xFF) as u8).unwrap();
        pit_data.write_one(((divisor >> 8) & 0xFF) as u8).unwrap();
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
    let val = port.read_one().unwrap() | (1 << irq);
    port.write_one(val).unwrap();
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
    let val = port.read_one().unwrap() & !(1 << irq);
    port.write_one(val).unwrap();
}

pub fn deinit() {
    let mut master_cmd: port::Port = port::Port::new(PIC_MASTER_DISABLE_PORT);
    let mut slave_cmd: port::Port = port::Port::new(PIC_SLAVE_DISABLE_PORT);
    master_cmd.write_one(0xff);
    slave_cmd.write_one(0xff);
}

pub fn end_of_interrupt(interrupt_id: interrupts::HardwareInterrupt) {
    if (interrupt_id as u8) >= PIC_SLAVE_OFFSET && (interrupt_id as u8) < PIC_SLAVE_OFFSET + 8 {
        port::Port::new(PIC_SLAVE_PORT).write_one(END_OF_INTERRUPT).unwrap();
    }
    port::Port::new(PIC_MASTER_PORT).write_one(END_OF_INTERRUPT).unwrap();
}