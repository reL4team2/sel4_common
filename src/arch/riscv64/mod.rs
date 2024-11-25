//! SBI (RISC-V Supervisor Binary Interface) wrapper
#![allow(unused)]

mod arch_tcb;
pub mod config;
mod message_info;
mod object;
mod registers;
mod timer;
mod vm_rights;
use crate::platform::time_def::ticks_t;
pub use arch_tcb::ArchTCB;
pub use message_info::*;
pub use object::*;
pub use registers::*;
use riscv::register::time;
pub use timer::*;
pub use vm_rights::*;

const SBI_SET_TIMER: usize = 0;
const SBI_CONSOLE_PUTCHAR: usize = 1;
const SBI_CONSOLE_GETCHAR: usize = 2;

const SBI_CLEAR_IPI: usize = 3;
const SBI_REMOTE_SFENCE_VMA: usize = 6;
const SBI_SHUTDOWN: usize = 8;
const SYSCALL_WRITE: usize = 64;

#[no_mangle]
pub fn sbi_call(which: usize, arg0: usize, arg1: usize, arg2: usize) -> usize {
    let mut ret;
    unsafe {
        core::arch::asm!(
        "ecall",
        inlateout("x10") arg0 => ret,
        in("x11") arg1,
        in("x12") arg2,
        in("x17") which,
        );
    }
    ret
}

pub fn set_timer(timer: ticks_t) {
    sbi_call(SBI_SET_TIMER, timer, 0, 0);
}

pub fn clear_ipi() {
    sbi_call(SBI_CLEAR_IPI, 0, 0, 0);
}
pub fn shutdown() -> ! {
    sbi_call(SBI_SHUTDOWN, 0, 0, 0);
    panic!("It should shutdown!");
}

pub fn sys_write(fd: usize, buffer: &[u8]) {
    sbi_call(SYSCALL_WRITE, fd, buffer.as_ptr() as usize, buffer.len());
}

pub fn remote_sfence_vma(hart_mask: usize, start: usize, size: usize) {
    let virt_addr_hart_mask = (&hart_mask) as *const usize as usize;
    sbi_call(SBI_REMOTE_SFENCE_VMA, virt_addr_hart_mask, 0, 0);
}

pub fn get_time() -> usize {
    time::read()
}
