#![no_main]
#![no_std]

// set the panic handler
use panic_semihosting as _;
use rtfm::app;
use stm32f4xx_hal::prelude::*;
use stm32f4xx_hal::usb::{Peripheral, UsbBusType};
use stm32f4xx_hal::{stm32, timer};
use stm32f4xx_hal::gpio::{self, Input, Output, PushPull, PullUp};
use keyberon::debounce::Debouncer;
use keyberon::matrix::{Matrix, PressedKeys};
use generic_array::typenum::{U1};
use keyberon::impl_heterogenous_array;
use embedded_hal::digital::v2::{InputPin, OutputPin};
use stm32f4xx_hal::gpio::gpioa::{PA1, PA0};
use usb_device::bus::UsbBusAllocator;
use usb_device::class::UsbClass as _;
use keyberon::layout::Layout;

type UsbClass = keyberon::Class<'static, UsbBusType, Leds>;
type UsbDevice = keyberon::Device<'static, UsbBusType>;

pub struct Cols(
    pub PA0<Input<PullUp>>,
);
impl_heterogenous_array! {
    Cols,
    dyn InputPin<Error = ()>,
    U1,
    [0]
}

pub struct Rows(
    pub PA1<Output<PushPull>>,
);
impl_heterogenous_array! {
    Rows,
    dyn OutputPin<Error = ()>,
    U1,
    [0]
}

pub static LAYERS: keyberon::layout::Layers = &[&[&[keyberon::action::k(keyberon::key_code::KeyCode::Space)]]];

pub struct Leds {
    caps_lock: gpio::gpioc::PC13<gpio::Output<gpio::PushPull>>,
}
impl keyberon::keyboard::Leds for Leds {
    fn caps_lock(&mut self, status: bool) {
        if status {
            self.caps_lock.set_low().unwrap()
        } else {
            self.caps_lock.set_high().unwrap()
        }
    }
}

#[app(device = stm32f4xx_hal::stm32, peripherals = true)]
const APP: () = {
    struct Resources {
        usb_dev: UsbDevice,
        usb_class: UsbClass,
        matrix: Matrix<Cols, Rows>,
        debouncer: Debouncer<PressedKeys<U1, U1>>,
        #[init(Layout::new(LAYERS))]
        layout: Layout,
        timer: timer::Timer<stm32::TIM3>,
    }
    
    #[init]
    fn init(c: init::Context) -> init::LateResources {
        static mut EP_MEMORY: [u32; 1024] = [0; 1024];
        static mut USB_BUS: Option<UsbBusAllocator<UsbBusType>> = None;

        let rcc = c.device.RCC.constrain();
        let clocks = rcc
            .cfgr
            .use_hse(25.mhz())
            .sysclk(48.mhz())
            .pclk1(24.mhz())
            .require_pll48clk()
            .freeze();
        let gpioa = c.device.GPIOA.split();

        let gpioc = c.device.GPIOC.split();
        let mut led = gpioc.pc13.into_push_pull_output();
        led.set_low().unwrap();
        let leds = Leds { caps_lock: led };

        let usb = Peripheral {
            usb_global: c.device.OTG_FS_GLOBAL,
            usb_device: c.device.OTG_FS_DEVICE,
            usb_pwrclk: c.device.OTG_FS_PWRCLK,
            pin_dm: gpioa.pa11.into_alternate_af10(),
            pin_dp: gpioa.pa12.into_alternate_af10(),
        };
        *USB_BUS = Some(UsbBusType::new(usb, EP_MEMORY));
        let usb_bus = USB_BUS.as_ref().unwrap();

        let usb_class = keyberon::new_class(usb_bus, leds);
        let usb_dev = keyberon::new_device(usb_bus);

        let mut timer = timer::Timer::tim3(c.device.TIM3, 1.khz(), clocks);
        timer.listen(timer::Event::TimeOut);

        let matrix = Matrix::new(
            Cols(gpioa.pa0.into_pull_up_input()),
            Rows(gpioa.pa1.into_push_pull_output()),
        );

        init::LateResources {
            usb_dev,
            usb_class,
            timer,
            debouncer: Debouncer::new(PressedKeys::new(), PressedKeys::new(), 5),
            matrix: matrix.unwrap(),
        }
    }

    #[task(binds = OTG_FS, priority = 2, resources = [usb_dev, usb_class])]
    fn usb_tx(mut c: usb_tx::Context) {
        usb_poll(&mut c.resources.usb_dev, &mut c.resources.usb_class);
    }

    #[task(binds = OTG_FS_WKUP, priority = 2, resources = [usb_dev, usb_class])]
    fn usb_rx(mut c: usb_rx::Context) {
        usb_poll(&mut c.resources.usb_dev, &mut c.resources.usb_class);
    }

    #[task(binds = TIM3, priority = 1, resources = [usb_class, matrix, debouncer, layout, timer])]
    fn tick(mut c: tick::Context) {
        //c.resources.timer.clear_interrupt(timer::Event::TimeOut);
        unsafe { &*stm32::TIM3::ptr() }.sr.write(|w| w.uif().clear_bit());

        if c.resources
            .debouncer
            .update(c.resources.matrix.get().unwrap())
        {
            let data = c.resources.debouncer.get();
            let report = c.resources.layout.report_from_pressed(data.iter_pressed());
            c.resources
                .usb_class
                .lock(|k| k.device_mut().set_keyboard_report(report.clone()));
            while let Ok(0) = c.resources.usb_class.lock(|k| k.write(report.as_bytes())) {}
        }
    }
};

fn usb_poll(usb_dev: &mut UsbDevice, keyboard: &mut UsbClass) {
    if usb_dev.poll(&mut [keyboard]) {
        keyboard.poll();
    }
}
