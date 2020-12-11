#![no_main]
#![no_std]

// set the panic handler
use panic_halt as _;

use core::convert::Infallible;
use embedded_hal::digital::v2::{InputPin, OutputPin};
use generic_array::typenum::{U13, U5};
use keyberon::action::{k, l, m, Action, Action::*, HoldTapConfig};
use keyberon::debounce::Debouncer;
use keyberon::impl_heterogenous_array;
use keyberon::key_code::KbHidReport;
use keyberon::key_code::KeyCode::{self, *};
use keyberon::layout::Layout;
use keyberon::matrix::{Matrix, PressedKeys};
use rtic::app;
use stm32f4xx_hal::gpio::{self, gpioa, gpiob, Input, Output, PullUp, PushPull};
use stm32f4xx_hal::otg_fs::{UsbBusType, USB};
use stm32f4xx_hal::prelude::*;
use stm32f4xx_hal::{stm32, timer};
use usb_device::bus::UsbBusAllocator;
use usb_device::class::UsbClass as _;

type UsbClass = keyberon::Class<'static, UsbBusType, Leds>;
type UsbDevice = usb_device::device::UsbDevice<'static, UsbBusType>;

pub struct Cols(
    gpiob::PB14<Input<PullUp>>,
    gpiob::PB15<Input<PullUp>>,
    gpiob::PB5<Input<PullUp>>,
    gpiob::PB6<Input<PullUp>>,
    gpiob::PB7<Input<PullUp>>,
    gpiob::PB8<Input<PullUp>>,
    gpioa::PA5<Input<PullUp>>,
    gpioa::PA6<Input<PullUp>>,
    gpioa::PA7<Input<PullUp>>,
    gpiob::PB0<Input<PullUp>>,
    gpiob::PB1<Input<PullUp>>,
    gpiob::PB10<Input<PullUp>>,
    gpioa::PA0<Input<PullUp>>,
);
impl_heterogenous_array! {
    Cols,
    dyn InputPin<Error = Infallible>,
    U13,
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]
}

pub struct Rows(
    gpioa::PA4<Output<PushPull>>,
    gpioa::PA3<Output<PushPull>>,
    gpioa::PA2<Output<PushPull>>,
    gpioa::PA1<Output<PushPull>>,
    gpiob::PB9<Output<PushPull>>,
);
impl_heterogenous_array! {
    Rows,
    dyn OutputPin<Error = Infallible>,
    U5,
    [0, 1, 2, 3, 4]
}

const CUT: Action<()> = m(&[LShift, Delete]);
const COPY: Action<()> = m(&[LCtrl, Insert]);
const PASTE: Action<()> = m(&[LShift, Insert]);
const L2_ENTER: Action<()> = HoldTap {
    timeout: 200,
    config: HoldTapConfig::HoldOnOtherKeyPress,
    tap_hold_interval: 0,
    hold: &l(2),
    tap: &k(Enter),
};
const L1_SP: Action<()> = HoldTap {
    timeout: 200,
    config: HoldTapConfig::Default,
    tap_hold_interval: 0,
    hold: &l(1),
    tap: &k(Space),
};
const CSPACE: Action<()> = m(&[LCtrl, Space]);

const SHIFT_ESC: Action<()> = HoldTap {
    timeout: 200,
    tap_hold_interval: 0,
    config: HoldTapConfig::Default,
    hold: &k(LShift),
    tap: &k(Escape),
};
const CTRL_INS: Action<()> = HoldTap {
    timeout: 200,
    tap_hold_interval: 0,
    config: HoldTapConfig::Default,
    hold: &k(LCtrl),
    tap: &k(Insert),
};
const ALT_NL: Action<()> = HoldTap {
    timeout: 200,
    tap_hold_interval: 0,
    config: HoldTapConfig::Default,
    hold: &k(LAlt),
    tap: &k(NumLock),
};

macro_rules! s {
    ($k:ident) => {
        m(&[LShift, $k])
    };
}
macro_rules! a {
    ($k:ident) => {
        m(&[RAlt, $k])
    };
}

// The 13th column is the hardware button of the development board,
// thus all the column is activated when the button is pushed. Because
// of that, only one action is defined in the 13th column.
#[rustfmt::skip]
pub static LAYERS: keyberon::layout::Layers<()> = &[
    &[
        &[k(Grave),  k(Kb1),k(Kb2),k(Kb3),  k(Kb4),k(Kb5), k(Kb6),   k(Kb7),  k(Kb8), k(Kb9),  k(Kb0),   k(Minus),  k(Space)],
        &[k(Tab),     k(Q), k(W),  k(E),    k(R), k(T),    k(Y),     k(U),    k(I),   k(O),    k(P),     k(LBracket)],
        &[k(RBracket),k(A), k(S),  k(D),    k(F), k(G),    k(H),     k(J),    k(K),   k(L),    k(SColon),k(Quote)   ],
        &[k(Equal),   k(Z), k(X),  k(C),    k(V), k(B),    k(N),     k(M),    k(Comma),k(Dot), k(Slash), k(Bslash)  ],
        &[Trans,      Trans,k(LGui),k(LAlt),L1_SP,k(LCtrl),k(RShift),L2_ENTER,k(RAlt),k(BSpace),Trans,   Trans      ],
    ], &[
        &[k(F1),         k(F2),   k(F3), k(F4),     k(F5),    k(F6),k(F7),      k(F8),    k(F9),    k(F10), k(F11),  k(F12)],
        &[Trans,         k(Pause),Trans, k(PScreen),Trans,    Trans,Trans,      k(BSpace),k(Delete),Trans,  Trans,   Trans ],
        &[Trans,         Trans,   ALT_NL,CTRL_INS,  SHIFT_ESC,Trans,k(CapsLock),k(Left),  k(Down),  k(Up),  k(Right),Trans ],
        &[k(NonUsBslash),k(Undo), CUT,   COPY,      PASTE,    Trans,Trans,      k(Home),  k(PgDown),k(PgUp),k(End),  Trans ],
        &[Trans,         Trans,   Trans, Trans,     Trans,    Trans,Trans,      Trans,    Trans,    Trans,  Trans,   Trans ],
    ], &[
        &[Trans,    Trans,  Trans,  Trans,  Trans,  Trans,  Trans,  Trans,  Trans,  Trans,  Trans,  Trans    ],
        &[s!(Grave),s!(Kb1),s!(Kb2),s!(Kb3),s!(Kb4),s!(Kb5),s!(Kb6),s!(Kb7),s!(Kb8),s!(Kb9),s!(Kb0),s!(Minus)],
        &[ k(Grave), k(Kb1), k(Kb2), k(Kb3), k(Kb4), k(Kb5), k(Kb6), k(Kb7), k(Kb8), k(Kb9), k(Kb0), k(Minus)],
        &[a!(Grave),a!(Kb1),a!(Kb2),a!(Kb3),a!(Kb4),a!(Kb5),a!(Kb6),a!(Kb7),a!(Kb8),a!(Kb9),a!(Kb0),a!(Minus)],
        &[Trans,    Trans,  Trans,  Trans,  CSPACE, Trans,  Trans,  Trans,  Trans,  Trans,  Trans,  Trans    ],
    ], &[
        &[Trans,Trans,Trans,Trans,Trans,Trans,Trans,Trans,Trans,Trans, Trans, Trans ],
        &[k(F1),k(F2),k(F3),k(F4),k(F5),k(F6),k(F7),k(F8),k(F9),k(F10),k(F11),k(F12)],
        &[Trans,Trans,Trans,Trans,Trans,Trans,Trans,Trans,Trans,Trans, Trans, Trans ],
        &[Action::Custom(()),Trans,Trans,Trans,Trans,Trans,Trans,Trans,Trans,Trans, Trans, Trans ],
        &[Trans,Trans,Trans,Trans,Trans,Trans,Trans,Trans,Trans,Trans, Trans, Trans ],
    ],
];

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
        debouncer: Debouncer<PressedKeys<U5, U13>>,
        layout: Layout<()>,
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
            .sysclk(84.mhz())
            .require_pll48clk()
            .freeze();
        let gpioa = c.device.GPIOA.split();
        let gpiob = c.device.GPIOB.split();
        let gpioc = c.device.GPIOC.split();

        let mut led = gpioc.pc13.into_push_pull_output();
        led.set_low().unwrap();
        let leds = Leds { caps_lock: led };

        let usb = USB {
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
            Cols(
                gpiob.pb14.into_pull_up_input(),
                gpiob.pb15.into_pull_up_input(),
                gpiob.pb5.into_pull_up_input(),
                gpiob.pb6.into_pull_up_input(),
                gpiob.pb7.into_pull_up_input(),
                gpiob.pb8.into_pull_up_input(),
                gpioa.pa5.into_pull_up_input(),
                gpioa.pa6.into_pull_up_input(),
                gpioa.pa7.into_pull_up_input(),
                gpiob.pb0.into_pull_up_input(),
                gpiob.pb1.into_pull_up_input(),
                gpiob.pb10.into_pull_up_input(),
                gpioa.pa0.into_pull_up_input(),
            ),
            Rows(
                gpioa.pa4.into_push_pull_output(),
                gpioa.pa3.into_push_pull_output(),
                gpioa.pa2.into_push_pull_output(),
                gpioa.pa1.into_push_pull_output(),
                gpiob.pb9.into_push_pull_output(),
            ),
        );

        init::LateResources {
            usb_dev,
            usb_class,
            timer,
            debouncer: Debouncer::new(PressedKeys::default(), PressedKeys::default(), 5),
            matrix: matrix.unwrap(),
            layout: Layout::new(LAYERS),
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
        c.resources.timer.clear_interrupt(timer::Event::TimeOut);

        for event in c
            .resources
            .debouncer
            .events(c.resources.matrix.get().unwrap())
        {
            c.resources.layout.event(event);
        }
        match c.resources.layout.tick() {
            keyberon::layout::CustomEvent::Release(()) => cortex_m::peripheral::SCB::sys_reset(),
            _ => (),
        }
        send_report(c.resources.layout.keycodes(), &mut c.resources.usb_class);
    }
};

fn send_report(iter: impl Iterator<Item = KeyCode>, usb_class: &mut resources::usb_class<'_>) {
    use rtic::Mutex;
    let report: KbHidReport = iter.collect();
    if usb_class.lock(|k| k.device_mut().set_keyboard_report(report.clone())) {
        while let Ok(0) = usb_class.lock(|k| k.write(report.as_bytes())) {}
    }
}

fn usb_poll(usb_dev: &mut UsbDevice, keyboard: &mut UsbClass) {
    if usb_dev.poll(&mut [keyboard]) {
        keyboard.poll();
    }
}
