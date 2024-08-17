#![no_std]
#![no_main]

use devices::{display::{Display, Rect, Span}, vibration_motor::VibrationMotor};
use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl,
    delay::Delay,
    gpio::{Event, Gpio9, GpioPin, Input, Io, Level, Output, Pull},
    peripheral::Peripheral,
    peripherals::Peripherals,
    prelude::*,
    rtc_cntl::Rtc,
    spi::{master::Spi, FullDuplexMode, SpiMode},
    system::SystemControl,
    time::current_time,
    timer::timg::TimerGroup,
};
use fugit::{HertzU32, Rate};

extern crate alloc;
use core::{cell::RefCell, mem::MaybeUninit};

use critical_section::Mutex;
use esp_println::println;

mod devices {
    pub mod display;
    pub mod vibration_motor;
}

#[global_allocator]
static ALLOCATOR: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();

fn init_heap() {
    const HEAP_SIZE: usize = 32 * 1024;
    static mut HEAP: MaybeUninit<[u8; HEAP_SIZE]> = MaybeUninit::uninit();

    unsafe {
        ALLOCATOR.init(HEAP.as_mut_ptr() as *mut u8, HEAP_SIZE);
    }
}

static BUTTON: Mutex<RefCell<Option<Input<GpioPin<26>>>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let system = SystemControl::new(peripherals.SYSTEM);

    let clocks = ClockControl::max(system.clock_control).freeze();
    let delay = Delay::new(&clocks);

    let rtc = Rtc::new(peripherals.LPWR, Some(handler));

    let mut io = Io::new(peripherals.GPIO, peripherals.IO_MUX);
    io.set_interrupt_handler(handler);

    let spi = Spi::new(
        peripherals.SPI2,
        HertzU32::Hz(20000000),
        SpiMode::Mode0,
        &clocks,
    )
    .with_mosi(io.pins.gpio23)
    .with_sck(io.pins.gpio18);

    let mut display = Display::new(
        rtc,
        spi,
        io.pins.gpio5,
        io.pins.gpio10,
        io.pins.gpio19,
        io.pins.gpio9,
        &clocks,
    );

    display.init().unwrap();

    display.clear_screen(0xFF).unwrap();

    display
        .draw_image(
            include_bytes!("../bg.bin"),
            Rect {
                x: Span { lo: 0, hi: 200 },
                y: Span { lo: 0, hi: 200 },
            },
            false,
            false,
        )
        .unwrap();

    let mut vibration_motor = VibrationMotor::new(io.pins.gpio13);
    vibration_motor.set_vibrating(true);
    delay.delay(500.millis());
    vibration_motor.set_vibrating(false);
    delay.delay(500.millis());

    let mut ip = Input::new(io.pins.gpio26, Pull::Up);

    if ip.is_high() {
        vibration_motor.set_vibrating(true);
        delay.delay(500.millis());
        vibration_motor.set_vibrating(false);
    }

    critical_section::with(|cs| {
        ip.listen(Event::RisingEdge);
        BUTTON.borrow_ref_mut(cs).replace(ip)
    });
    esp_println::logger::init_logger_from_env();
    /*init_heap();

    let timer = esp_hal::timer::PeriodicTimer::new(
        esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG1, &clocks, None)
            .timer0
            .into(),
    );
    let init = esp_wifi::initialize(
        esp_wifi::EspWifiInitFor::Ble,
        timer,
        esp_hal::rng::Rng::new(peripherals.RNG),
        peripherals.RADIO_CLK,
        &clocks,
    )
    .unwrap();

    let ble_conn = esp_wifi::ble::controller::BleConnector::new(&init, peripherals.BT);*/

    delay.delay(500.millis());

    loop {
        println!("FUCK");
        delay.delay(500.millis());
    }
}

#[handler]
fn handler() {
    critical_section::with(|cs| {
        println!("GPIO interrupt");
        BUTTON
            .borrow_ref_mut(cs)
            .as_mut()
            .unwrap()
            .clear_interrupt();
    });
}
