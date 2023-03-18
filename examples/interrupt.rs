#![no_std]
#![no_main]

extern crate panic_halt;

use hifive1::{
    hal::{
        core::{
            plic::{Interrupt, Plic, Priority},
            CorePeripherals,
        },
        prelude::*,
        DeviceResources,
    },
    pin, sprintln,
};
use riscv::register::{mie, mstatus};
use riscv_rt::entry;

#[no_mangle]
pub unsafe extern "C" fn MachineTimer() {
    sprintln!("timer");
    let mut clint = CorePeripherals::steal().clint;
    sprintln!("  (mtime = {})", clint.mtime.mtime());
    clint.mtimecmp.set_mtimecmp(clint.mtime.mtime() + 65536 / 2);
}

#[no_mangle]
pub unsafe extern "C" fn MachineExternal() {
    sprintln!("external");
    if let Some(intr) = Plic::claim() {
        match intr {
            Interrupt::RTC => {
                let rtc = &*hifive1::hal::e310x::RTC::ptr();
                let rtccmp = rtc.rtccmp.read().bits();
                sprintln!("  (rtccmp = {})", rtccmp);
                rtc.rtccmp.write(|w| w.bits(rtccmp + 65536));
            }
            _ => panic!("unknown interrupt"),
        }
        Plic::complete(intr);
    } else {
        panic!("machine external triggered erroneously");
    }
}

#[entry]
fn main() -> ! {
    let dr = DeviceResources::take().unwrap();
    let p = dr.peripherals;
    let pins = dr.pins;

    // Configure clocks
    let clocks = hifive1::clock::configure(p.PRCI, p.AONCLK, 64.mhz().into());

    // make sure that interrupts are off
    unsafe {
        mstatus::clear_mie();
        mie::clear_mtimer();
        mie::clear_mext();
    };

    // Configure UART for stdout
    hifive1::stdout::configure(
        p.UART0,
        pin!(pins, uart0_tx),
        pin!(pins, uart0_rx),
        115_200.bps(),
        clocks,
    );

    sprintln!("\nhello world!");

    // Disable watchdog
    let wdg = p.WDOG;
    wdg.wdogcfg.modify(|_, w| w.enalways().clear_bit());

    // Configure CLINT for MTIMER interrupts
    let mut clint = dr.core_peripherals.clint;
    clint.mtimecmp.set_mtimecmp(clint.mtime.mtime() + 10000);

    // configure PLIC for MEXT interrupts
    unsafe {
        let mut plic = CorePeripherals::steal().plic;

        // Reset PLIC
        plic.reset();
        // Activate RTC interrupts
        plic.interrupt_enable(Interrupt::RTC);
        Plic::set_priority(Interrupt::RTC, Priority::P7);
        // Set PLIC threshold
        Plic::set_threshold(Priority::P1);
    }

    // Configure RTC
    let mut rtc = p.RTC.constrain();
    rtc.disable();
    rtc.set_scale(0);
    rtc.set_rtc(0);
    rtc.set_rtccmp(10000);

    // activate interrupts
    unsafe {
        mstatus::set_mie();
        mie::set_mtimer();
        mie::set_mext();
        rtc.enable();
    };

    loop {}
}
