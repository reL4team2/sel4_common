use super::{CONTEXT_REG_NUM, ELR_EL1, SPSR_EL1, TLS_BASE, TPIDRRO_EL0, TPIDR_EL0};

/// Get value from the system register
/// TODO: Move this macro into a proper place
macro_rules! mrs {
    ($reg: literal) => {
        {
            let value: usize;
            unsafe {
                core::arch::asm!(concat!("mrs {0}, ", $reg), out(reg) value);
            }
            value
        }
    };
}

/// This is `arch_tcb_t` in the sel4_c_impl.
#[repr(C)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ArchTCB {
    pub(in crate::arch) registers: [usize; CONTEXT_REG_NUM],
}

/// Implements the Default for the `ArchTCB`
impl Default for ArchTCB {
    fn default() -> Self {
        let mut registers = [0; CONTEXT_REG_NUM];
        registers[SPSR_EL1] = (1 << 6) | 0 | (1 << 8);
        Self { registers }
    }
}
impl ArchTCB {
    /// Config the registers fot the idle thread.
    pub fn config_idle_thread(&mut self, idle_thread: usize) {
        self.registers[ELR_EL1] = idle_thread;
        self.registers[SPSR_EL1] = (1 << 6) | 5 | (1 << 8);
    }

    /// Save TLS(Thread local Storage) registers
    #[inline]
    pub fn save_thread_local(&mut self) {
        self.registers[TPIDR_EL0] = mrs!("tpidr_el0");
        self.registers[TPIDRRO_EL0] = mrs!("tpidrro_el0");
    }
}
