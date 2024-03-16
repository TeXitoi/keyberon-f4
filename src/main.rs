#![no_main]
#![no_std]

// set the panic handler
use panic_halt as _;

use keyberon::debounce::Debouncer;
use keyberon::key_code::KbHidReport;
use keyberon::layout::Layout;
use keyberon::matrix::Matrix;
use rtic::app;
use stm32f4xx_hal::gpio::{self, EPin, Input, Output, PushPull};
use stm32f4xx_hal::otg_fs::{UsbBusType, USB};
use stm32f4xx_hal::prelude::*;
use stm32f4xx_hal::{pac, timer};
use usb_device::bus::UsbBusAllocator;
use usb_device::class::UsbClass as _;

mod layout;

type UsbClass = keyberon::Class<'static, UsbBusType, Leds>;
type UsbDevice = usb_device::device::UsbDevice<'static, UsbBusType>;

pub struct Leds {
    caps_lock: gpio::gpioc::PC13<gpio::Output<gpio::PushPull>>,
}
impl keyberon::keyboard::Leds for Leds {
    fn caps_lock(&mut self, status: bool) {
        if status {
            self.caps_lock.set_low()
        } else {
            self.caps_lock.set_high()
        }
    }
}

#[app(device = stm32f4xx_hal::pac, peripherals = true)]
mod app {
    use super::*;

    #[shared]
    struct Shared {
        usb_dev: UsbDevice,
        usb_class: UsbClass,
    }

    #[local]
    struct Local {
        matrix: Matrix<EPin<Input>, EPin<Output<PushPull>>, 13, 4>,
        debouncer: Debouncer<[[bool; 13]; 4]>,
        layout: Layout<12, 4, 4, ()>,
        timer: timer::CounterHz<pac::TIM3>,
    }

    #[init(local = [bus: Option<UsbBusAllocator<UsbBusType>> = None, ep_mem: [u32; 1024] = [0; 1024]])]
    fn init(c: init::Context) -> (Shared, Local, init::Monotonics) {
        let rcc = c.device.RCC.constrain();
        let clocks = rcc
            .cfgr
            .use_hse(25.MHz())
            .sysclk(84.MHz())
            .require_pll48clk()
            .freeze();
        let gpioa = c.device.GPIOA.split();
        let gpiob = c.device.GPIOB.split();
        let gpioc = c.device.GPIOC.split();

        let mut led = gpioc.pc13.into_push_pull_output();
        led.set_low();
        let leds = Leds { caps_lock: led };

        let usb = USB::new(
            (
                c.device.OTG_FS_GLOBAL,
                c.device.OTG_FS_DEVICE,
                c.device.OTG_FS_PWRCLK,
            ),
            (gpioa.pa11, gpioa.pa12),
            &clocks,
        );
        *c.local.bus = Some(UsbBusType::new(usb, c.local.ep_mem));
        let usb_bus = c.local.bus.as_ref().unwrap();

        let usb_class = keyberon::new_class(usb_bus, leds);
        let usb_dev = keyberon::new_device(usb_bus);

        let mut timer = timer::Timer::new(c.device.TIM3, &clocks).counter_hz();
        timer.start(1.kHz()).unwrap();
        timer.listen(timer::Event::Update);

        let matrix = Matrix::new(
            [
                gpiob.pb14.into_pull_up_input().erase(),
                gpiob.pb15.into_pull_up_input().erase(),
                gpiob.pb5.into_pull_up_input().erase(),
                gpiob.pb6.into_pull_up_input().erase(),
                gpiob.pb7.into_pull_up_input().erase(),
                gpiob.pb8.into_pull_up_input().erase(),
                gpioa.pa5.into_pull_up_input().erase(),
                gpioa.pa6.into_pull_up_input().erase(),
                gpioa.pa7.into_pull_up_input().erase(),
                gpiob.pb0.into_pull_up_input().erase(),
                gpiob.pb1.into_pull_up_input().erase(),
                gpiob.pb10.into_pull_up_input().erase(),
                gpioa.pa0.into_pull_up_input().erase(),
            ],
            [
                gpioa.pa3.into_push_pull_output().erase(),
                gpioa.pa2.into_push_pull_output().erase(),
                gpioa.pa1.into_push_pull_output().erase(),
                gpiob.pb9.into_push_pull_output().erase(),
            ],
        );

        (
            Shared { usb_dev, usb_class },
            Local {
                timer,
                debouncer: Debouncer::new([[false; 13]; 4], [[false; 13]; 4], 5),
                matrix: matrix.unwrap(),
                layout: Layout::new(&crate::layout::LAYERS),
            },
            init::Monotonics(),
        )
    }

    #[task(binds = OTG_FS, priority = 2, shared = [usb_dev, usb_class])]
    fn usb_tx(c: usb_tx::Context) {
        (c.shared.usb_dev, c.shared.usb_class).lock(|usb_dev, usb_class| {
            if usb_dev.poll(&mut [usb_class]) {
                usb_class.poll();
            }
        })
    }

    #[task(binds = TIM3, priority = 1, shared = [usb_class], local = [matrix, debouncer, layout, timer])]
    fn tick(mut c: tick::Context) {
        c.local.timer.clear_flags(timer::Flag::Update);

        for event in c.local.debouncer.events(c.local.matrix.get().unwrap()) {
            c.local.layout.event(event);
        }
        match c.local.layout.tick() {
            keyberon::layout::CustomEvent::Release(()) => unsafe {
                cortex_m::asm::bootload(0x1FFF0000 as _)
            },
            _ => (),
        }

        let report: KbHidReport = c.local.layout.keycodes().collect();
        if c.shared
            .usb_class
            .lock(|k| k.device_mut().set_keyboard_report(report.clone()))
        {
            while let Ok(0) = c.shared.usb_class.lock(|k| k.write(report.as_bytes())) {}
        }
    }
}
