pub(crate) const CONFIGURE_TIMER_FREQUENCY: usize = 62500000;
#[cfg(feature = "kernel_mcs")]
#[allow(dead_code)]
pub(crate) const CONFIGURE_CLK_MAGIC: usize = 4611686019;
#[cfg(feature = "kernel_mcs")]
#[allow(dead_code)]
pub(crate) const CONFIGURE_CLK_SHIFT: usize = 58;
#[cfg(feature = "kernel_mcs")]
#[allow(dead_code)]
pub(crate) const CONFIGURE_KERNEL_WCET: usize = 10;
#[cfg(feature = "kernel_mcs")]
#[allow(dead_code)]
pub(crate) const TIMER_PRECISION: usize = 0;
#[cfg(feature = "kernel_mcs")]
#[allow(dead_code)]
pub(crate) const TIMER_OVERHEAD_TICKS: usize = 0;
use core::arch::asm;

use aarch64_cpu::registers::{Writeable, CNTV_CTL_EL0, CNTV_CVAL_EL0, CNTV_TVAL_EL0};

use crate::sel4_config::UINT64_MAX;
#[cfg(feature = "kernel_mcs")]
use crate::BIT;

use super::{
    time_def::{ticks_t, TIMER_CLOCK_HZ},
    Timer_func,
};

pub struct timer;

impl Timer_func for timer {
    fn init_timer(self) {
        // Here use the generic timer init
        // check frequency is correct
        unsafe {
            let mut gpt_cntfrq: usize;
            asm!(
                "mrs {},cntfrq_el0",
                out(reg) gpt_cntfrq,
            );
            if gpt_cntfrq != 0 && gpt_cntfrq != TIMER_CLOCK_HZ {
                panic!("The gpt_cntfrq is unequal to the system configure");
            }
        }
        #[cfg(feature = "kernel_mcs")]
        {
            self.ack_deadline_irq();
            CNTV_CTL_EL0.set(BIT!(0) as u64);
        }
        #[cfg(not(feature = "kernel_mcs"))]
        {
            self.reset_timer();
        }
    }
    fn get_current_time(self) -> ticks_t {
        let time: ticks_t;
        unsafe {
            asm!(
                "mrs {}, cntvct_el0",
                out(reg) time,
            );
        }
        time
    }
    fn set_deadline(self, deadline: ticks_t) {
        CNTV_CVAL_EL0.set(deadline as u64);
    }
    /// Reset the current Timer
    #[no_mangle]
    fn reset_timer(self) {
        /*
            SYSTEM_WRITE_WORD(CNT_TVAL, TIMER_RELOAD);
            SYSTEM_WRITE_WORD(CNT_CTL, BIT(0));
        */
        // TODO: Set a proper timer clock
        CNTV_TVAL_EL0.set(TIMER_CLOCK_HZ as u64 / 1000 * 2);
        CNTV_CTL_EL0.set(1);
    }
    fn ack_deadline_irq(self) {
        let deadline: ticks_t = UINT64_MAX;
        self.set_deadline(deadline);
    }
}
