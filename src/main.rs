#![no_std]
#![no_main]

use display::Display;
use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl, delay::Delay, gpio::{Event, Gpio9, GpioPin, Input, Io, Level, Output, Pull}, peripheral::Peripheral, peripherals::Peripherals, prelude::*, rtc_cntl::Rtc, spi::{master::Spi, FullDuplexMode, SpiMode}, system::SystemControl, time::current_time, timer::timg::TimerGroup
};
use fugit::{HertzU32, Rate};

extern crate alloc;
use core::{cell::RefCell, mem::MaybeUninit};

use critical_section::Mutex;
use esp_println::println;

mod display;

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

/*
#define DISPLAY_CS 5
#define DISPLAY_RES 9
#define DISPLAY_DC 10
#define DISPLAY_BUSY 19
*/
#[entry]
fn main() -> ! {
    println!("Welcome!");
    let peripherals = Peripherals::take();
    let system = SystemControl::new(peripherals.SYSTEM);

    let clocks = ClockControl::max(system.clock_control).freeze();
    let delay = Delay::new(&clocks);
    
    let rtc = Rtc::new(peripherals.LPWR, Some(handler));

    let mut io = Io::new(peripherals.GPIO, peripherals.IO_MUX);
    io.set_interrupt_handler(handler);

    let spi = Spi::<'_, _, FullDuplexMode>::with_cs(
        Spi::<'_, _, FullDuplexMode>::with_mosi(
            Spi::<'_, _, FullDuplexMode>::with_sck(
                Spi::new(
                    peripherals.SPI2,
                    HertzU32::Hz(20000000),
                    SpiMode::Mode0,
                    &clocks,
                ),
                io.pins.gpio18,
            ),
            io.pins.gpio23,
        ),
        unsafe { io.pins.gpio5.clone_unchecked() },
    );

    let mut display = Display {
        power_is_on: false,
        using_partial_mode: false,
        initial_refresh: true,
        initial_write: true,
        pulldown_rst_mode: true,
        delay,
        rtc,
        spi,
        cs: Output::new(io.pins.gpio5, Level::High),
        dc: Output::new(io.pins.gpio10, Level::High),
        busy: Input::new(io.pins.gpio19, Pull::None),
        rst: io.pins.gpio9,
        rst_in: None,
    };

    display.init().unwrap();

    display.clear_screen(0xFF).unwrap();

    display.draw_image(include_bytes!("../bg.bin"), 0, 0, 200, 200, false, false, false).unwrap();

    let mut op = Output::new(io.pins.gpio13, Level::Low);

    op.set_high();
    delay.delay(500.millis());
    op.set_low();
    delay.delay(500.millis());

    let mut ip = Input::new(io.pins.gpio26, Pull::Up);

    if ip.is_high() {
        op.set_high();
        delay.delay(500.millis());
        op.set_low();
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
