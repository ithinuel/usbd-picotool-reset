#![no_std]

use core::marker::PhantomData;

use usb_device::class_prelude::{InterfaceNumber, UsbBus, UsbBusAllocator};

// Vendor specific class
const CLASS_VENDOR_SPECIFIC: u8 = 0xFF;
// cf: https://github.com/raspberrypi/pico-sdk/blob/f396d05f8252d4670d4ea05c8b7ac938ef0cd381/src/common/pico_usb_reset_interface/include/pico/usb_reset_interface.h#L17
const RESET_INTERFACE_SUBCLASS: u8 = 0x00;
const RESET_INTERFACE_PROTOCOL: u8 = 0x01;
const RESET_REQUEST_BOOTSEL: u8 = 0x01;
const RESET_REQUEST_FLASH: u8 = 0x02;

pub enum DisableInterface {
    BothEnabled = 0,
    DisableMassStorage = 1,
    DisablePicoBoot = 2,
}
impl DisableInterface {
    const fn into(self) -> u32 {
        match self {
            DisableInterface::BothEnabled => 0,
            DisableInterface::DisableMassStorage => 1,
            DisableInterface::DisablePicoBoot => 2,
        }
    }
}
pub trait Config {
    const INTERFACE_DISABLE_MASK: DisableInterface;
    const BOOTSEL_ACTIVITY_LED: Option<usize>;
}

pub enum DefaultConfig {}
impl Config for DefaultConfig {
    const INTERFACE_DISABLE_MASK: DisableInterface = DisableInterface::BothEnabled;

    const BOOTSEL_ACTIVITY_LED: Option<usize> = None;
}

pub struct PicoToolsReset<'a, B: UsbBus, C: Config = DefaultConfig> {
    intf: InterfaceNumber,
    _bus: PhantomData<&'a B>,
    _cnf: PhantomData<C>,
}
impl<'a, B: UsbBus, C: Config> PicoToolsReset<'a, B, C> {
    pub fn new(alloc: &'a UsbBusAllocator<B>) -> PicoToolsReset<'a, B, C> {
        Self {
            intf: alloc.interface(),
            _bus: PhantomData,
            _cnf: PhantomData,
        }
    }
}

impl<B: UsbBus, C: Config> usb_device::class::UsbClass<B> for PicoToolsReset<'_, B, C> {
    fn get_configuration_descriptors(
        &self,
        writer: &mut usb_device::descriptor::DescriptorWriter,
    ) -> usb_device::Result<()> {
        writer.interface(
            self.intf,
            CLASS_VENDOR_SPECIFIC,
            RESET_INTERFACE_SUBCLASS,
            RESET_INTERFACE_PROTOCOL,
        )
    }

    fn reset(&mut self) {}

    fn control_out(&mut self, xfer: usb_device::class_prelude::ControlOut<B>) {
        let req = xfer.request();
        if !(req.request_type == usb_device::control::RequestType::Class
            && req.recipient == usb_device::control::Recipient::Interface
            && req.index == u8::from(self.intf) as u16)
        {
            return;
        }

        match req.request {
            RESET_REQUEST_BOOTSEL => {
                let mut gpio_mask = C::BOOTSEL_ACTIVITY_LED.map(|led| 1 << led).unwrap_or(0);
                if req.value & 0x100 != 0 {
                    gpio_mask = 1 << (req.value >> 9);
                }
                rp2040_hal::rom_data::reset_to_usb_boot(
                    gpio_mask,
                    u32::from(req.value & 0x7F) | C::INTERFACE_DISABLE_MASK.into(),
                )
            }
            RESET_REQUEST_FLASH => todo!(),
            _ => {
                let _ = xfer.accept();
            }
        }
    }

    fn control_in(&mut self, xfer: usb_device::class_prelude::ControlIn<B>) {
        let _ = xfer;
    }
}
