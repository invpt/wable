#![no_std]
#![no_main]

use devices::{
    ble::{
        command::{
            le_set_scan_enable::LeSetScanEnable, le_set_scan_parameters::LeSetScanParameters,
            reset::Reset, set_event_mask::SetEventMask,
        },
        event::le_advertising_report::LeAdvertisingReport,
        Ble, PollBehavior,
    },
    vibration_motor::VibrationMotor,
};
use embedded_graphics::{
    mono_font::{ascii::FONT_9X18_BOLD, MonoTextStyle},
    prelude::*,
    text::Text,
};
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl, delay::Delay, gpio::{GpioPin, Input, Io, Level, Output, Pull}, i2c::I2C, peripherals::Peripherals, prelude::*, rng::Rng, spi::{master::Spi, SpiMode}, system::SystemControl, timer::{timg::TimerGroup, PeriodicTimer}
};
use esp_wifi::{ble::controller::BleConnector, current_millis, EspWifiInitFor};
use fugit::HertzU32;
use pcf8563::DateTime;
use wepd::{Display, DisplayConfiguration, Framebuffer};

use core::cell::RefCell;

use critical_section::Mutex;
use esp_println::println;

mod devices {
    pub mod ble;
    pub mod vibration_motor;
}

static BUTTON: Mutex<RefCell<Option<Input<GpioPin<26>>>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let system = SystemControl::new(peripherals.SYSTEM);

    let clocks = ClockControl::max(system.clock_control).freeze();
    let delay = Delay::new(&clocks);

    //let rtc = Rtc::new(peripherals.LPWR, Some(handler));

    let mut io = Io::new(peripherals.GPIO, peripherals.IO_MUX);
    io.set_interrupt_handler(handler);

    let bus = Spi::new(
        peripherals.SPI2,
        HertzU32::Hz(20000000),
        SpiMode::Mode0,
        &clocks,
    )
    .with_mosi(io.pins.gpio23)
    .with_sck(io.pins.gpio18);

    let mut display = Display::new(DisplayConfiguration {
        spi: ExclusiveDevice::new(bus, Output::new(io.pins.gpio5, Level::High), delay).unwrap(),
        dc: Output::new(io.pins.gpio10, Level::High),
        rst: Output::new(io.pins.gpio9, Level::High),
        busy: Input::new(io.pins.gpio19, Pull::None),
        delay,
        current_millis,
        wait: || delay.delay(1.millis()),
    })
    .unwrap();

    display.reset().unwrap();

    display.clear_screen(0xFF).unwrap();

    let mut fb = Framebuffer::new();

    let style = MonoTextStyle::new(&FONT_9X18_BOLD, wepd::Color::Black);
    Text::new("Hello world", Point { x: 5, y: 15 }, style)
        .draw(&mut fb)
        .unwrap();
    fb.flush(&mut display).unwrap();

    display.power_off().unwrap();

    let i2c = I2C::new(
        peripherals.I2C0,
        io.pins.gpio21,
        io.pins.gpio22,
        HertzU32::Hz(32768),
        &clocks,
        None,
    );

    let mut rtc = pcf8563::PCF8563::new(i2c);

    rtc.set_datetime(&DateTime {
        year: 24,
        month: 8,
        weekday: 6,
        day: 17,
        hours: 8,
        minutes: 10,
        seconds: 0,
    })
    .unwrap();

    let mut vibration_motor = VibrationMotor::new(io.pins.gpio13);
    vibration_motor.set_vibrating(true);
    delay.delay(500.millis());
    vibration_motor.set_vibrating(false);
    delay.delay(500.millis());

    let timer = PeriodicTimer::new(
        TimerGroup::new(peripherals.TIMG1, &clocks, None)
            .timer0
            .into(),
    );
    let init = esp_wifi::initialize(
        EspWifiInitFor::Ble,
        timer,
        Rng::new(peripherals.RNG),
        peripherals.RADIO_CLK,
        &clocks,
    )
    .unwrap();

    let ble_conn = BleConnector::new(&init, peripherals.BT);

    let (mut ble, qslot) = Ble::new(ble_conn, delay);

    let (status, qslot) = ble
        .run_until_complete(qslot, PollBehavior::Strict, Reset {})
        .unwrap();
    status.assert().unwrap();

    let (status, qslot) = ble
        .run_until_complete(qslot, PollBehavior::Strict, SetEventMask { mask: !0 })
        .unwrap();
    status.assert().unwrap();

    let (status, qslot) = ble
        .run_until_complete(
            qslot,
            PollBehavior::Strict,
            LeSetScanParameters {
                le_scan_type: 0x01,
                le_scan_interval: 0x0100,
                le_scan_window: 0x0010,
                own_address_type: 0x00,
                scanning_filter_policy: 0x00,
            },
        )
        .unwrap();
    status.assert().unwrap();

    let (status, _qslot) = ble
        .run_until_complete(
            qslot,
            PollBehavior::Strict,
            LeSetScanEnable {
                le_scan_enable: 0x01,
                filter_duplicates: 0x00,
            },
        )
        .unwrap();
    status.assert().unwrap();

    loop {
        if let Some(event) = ble.filter_poll::<LeAdvertisingReport>().unwrap() {
            let time = rtc.get_datetime().unwrap();

            println!("received report at {time:?}");
            for item in event.items() {
                let item = item.unwrap();

                println!("{:?}", item);
            }
        }
    }
    /*

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
    init_heap();

    delay.delay(500.millis());

    loop {
        dbg!(rtc.get_datetime().unwrap());
        delay.delay(500.millis());
    }*/
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
