//! # Pico USB PicoTool Example
//!
//! Creates a USB device on a Pico board, with the USB driver running in the main thread.
//!
//! This will create a USB Device reseting when triggered by the `picotool` cli util.
//!
//! See the `Cargo.toml` file for Copyright and license details.

#![no_std]
#![no_main]

use panic_halt as _;

use rp_pico::hal::pac;
use rp_pico::hal;
use rp_pico::entry;

// USB Device support
use usb_device::{class_prelude::*, prelude::*};

// USB PicoTool Class Device support
use usbd_picotool_reset::PicoToolReset;


#[entry]
fn main() -> ! {
    // Grab our singleton objects
    let mut pac = pac::Peripherals::take().unwrap();

    // Set up the watchdog driver - needed by the clock setup code
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    // Configure the clocks
    //
    // The default is to generate a 125 MHz system clock
    let clocks = hal::clocks::init_clocks_and_plls(
        rp_pico::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    #[cfg(feature = "rp2040-hal/rp2040-e5")]
    {
        let sio = hal::Sio::new(pac.SIO);
        let _pins = rp_pico::Pins::new(
            pac.IO_BANK0,
            pac.PADS_BANK0,
            sio.gpio_bank0,
            &mut pac.RESETS,
        );
    }

    // Set up the USB driver
    let usb_bus = UsbBusAllocator::new(hal::usb::UsbBus::new(
        pac.USBCTRL_REGS,
        pac.USBCTRL_DPRAM,
        clocks.usb_clock,
        true,
        &mut pac.RESETS,
    ));

    // Set up the USB PicoTool Class Device driver
    let mut picotool: PicoToolReset<_> = PicoToolReset::new(&usb_bus);

    // Create a USB device with a fake VID and PID
    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x16c0, 0x27dd))
        .manufacturer("Fake company")
        .product("Picotool port")
        .serial_number("TEST")
        .device_class(0xFF) // from: https://www.usb.org/defined-class-codes
        .build();

    loop {
        usb_dev.poll(&mut [&mut picotool]) ;
    }
}
