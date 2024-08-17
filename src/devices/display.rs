use core::ops::{Add, Div, Mul, Sub};

use esp_hal::{
    clock::Clocks,
    delay::Delay,
    gpio::{Input, InputPin, Level, Output, OutputPin, Pull, WakeEvent},
    peripheral::Peripheral,
    peripherals::{SPI0, SPI1, SPI2, SPI3},
    prelude::*,
    rtc_cntl::{sleep::GpioWakeupSource, Rtc},
    spi::{self, master::Spi, FullDuplexMode},
    time::current_time,
};
use esp_println::{dbg, println};

/*
#define DISPLAY_CS 5
#define DISPLAY_RES 9
#define DISPLAY_DC 10
#define DISPLAY_BUSY 19
*/

const WIDTH: usize = 200;
const HEIGHT: usize = 200;

const SCREEN_RECT: Rect = Rect {
    x: Span {
        lo: 0,
        hi: WIDTH as i16,
    },
    y: Span {
        lo: 0,
        hi: HEIGHT as i16,
    },
};

pub struct Span {
    pub lo: i16,
    pub hi: i16,
}

impl Span {
    /// Returns the size of the span, calculated as `hi - lo`.
    pub fn size(&self) -> i16 {
        self.hi - self.lo
    }

    /// Computes the intersection of two spans.
    /// Returns `None` if there is no intersection, otherwise returns `Some(Span)`.
    pub fn intersection(&self, other: &Span) -> Option<Span> {
        let lo = self.lo.max(other.lo);
        let hi = self.hi.min(other.hi);

        if lo <= hi {
            Some(Span { lo, hi })
        } else {
            None
        }
    }
}

pub struct Rect {
    pub x: Span,
    pub y: Span,
}

impl Rect {
    // Returns the width of the rectangle, which is the size of the x-span.
    pub fn width(&self) -> i16 {
        self.x.size()
    }

    /// Returns the height of the rectangle, which is the size of the y-span.
    pub fn height(&self) -> i16 {
        self.y.size()
    }

    /// Computes the intersection of two rectangles.
    /// Returns `None` if there is no intersection, otherwise returns `Some(Rect)`.
    pub fn intersection(&self, other: &Rect) -> Option<Rect> {
        let x = self.x.intersection(&other.x)?;
        let y = self.y.intersection(&other.y)?;

        Some(Rect { x, y })
    }
}

pub struct Display<'d, Cs, Dc, Busy, Rst> {
    pub power_is_on: bool,
    pub using_partial_mode: bool,
    pub initial_refresh: bool,
    pub initial_write: bool,
    pub pulldown_rst_mode: bool,
    pub delay: Delay,
    pub rtc: Rtc<'d>,
    pub spi: Spi<'d, SPI2, FullDuplexMode>,
    pub cs: Output<'d, Cs>,
    pub dc: Output<'d, Dc>,
    pub busy: Input<'d, Busy>,
    pub rst: Rst,
}

impl<'d, Cs, Dc, Busy, Rst> Display<'d, Cs, Dc, Busy, Rst>
where
    Cs: OutputPin + Peripheral<P = Cs>,
    Dc: OutputPin + Peripheral<P = Dc>,
    Busy: InputPin + Peripheral<P = Busy>,
    Rst: OutputPin + InputPin + Peripheral<P = Rst>,
{
    pub fn new(
        rtc: Rtc<'d>,
        spi: Spi<'d, SPI2, FullDuplexMode>,
        cs: Cs,
        dc: Dc,
        busy: Busy,
        rst: Rst,
        clocks: &Clocks,
    ) -> Self {
        Self {
            power_is_on: false,
            using_partial_mode: false,
            initial_refresh: true,
            initial_write: true,
            pulldown_rst_mode: true,
            delay: Delay::new(clocks),
            rtc,
            spi,
            cs: Output::new(cs, Level::High),
            dc: Output::new(dc, Level::High),
            busy: Input::new(busy, Pull::None),
            rst,
        }
    }

    pub fn init(&mut self) -> Result<(), spi::Error> {
        self.cs.set_high();
        self.reset()?;
        Ok(())
    }

    pub fn reset(&mut self) -> Result<(), spi::Error> {
        if self.pulldown_rst_mode {
            drop(Output::new(&mut self.rst, Level::Low));
            self.delay.delay(10.millis());

            drop(Input::new(&mut self.rst, Pull::Up));
            self.delay.delay(10.millis());
        } else {
            todo!()
        }

        Ok(())
    }

    pub fn clear_screen(&mut self, value: u8) -> Result<(), spi::Error> {
        self.write_screen_buffer(value)?;
        self.refresh_all(true)?;
        self.write_screen_buffer_again(value)?;

        Ok(())
    }

    pub fn draw_image(
        &mut self,
        bitmap: &[u8],
        rect: Rect,
        invert: bool,
        mirror_y: bool,
    ) -> Result<(), spi::Error> {
        self.write_image(bitmap, rect, invert, mirror_y)?;
        self.refresh(rect)?;
        self.write_image_again(bitmap, rect, invert, mirror_y)?;

        Ok(())
    }

    pub fn write_image(
        &mut self,
        bitmap: &[u8],
        rect: Rect,
        invert: bool,
        mirror_y: bool,
    ) -> Result<(), spi::Error> {
        self.write_image_inner(0x24, bitmap, rect, invert, mirror_y)?;
        Ok(())
    }

    pub fn write_image_again(
        &mut self,
        bitmap: &[u8],
        rect: Rect,
        invert: bool,
        mirror_y: bool,
    ) -> Result<(), spi::Error> {
        self.write_image_inner(0x24, bitmap, rect, invert, mirror_y)?;
        Ok(())
    }

    fn write_image_inner(
        &mut self,
        command: u8,
        bitmap: &[u8],
        rect: Rect,
        invert: bool,
        mirror_y: bool,
    ) -> Result<(), spi::Error> {
        if self.initial_write {
            self.write_screen_buffer(0xFF)?;
        }

        let wb = (w + 7) / 8;
        x -= x % 8;
        w = wb * 8;
        let x1 = x.max(0);
        let y1 = y.max(0);
        let mut w1 = if x + w < WIDTH as i16 {
            w
        } else {
            WIDTH as i16 - x
        };
        let mut h1 = if y + h < HEIGHT as i16 {
            h
        } else {
            HEIGHT as i16 - y
        };
        let dx = x1 - x;
        let dy = y1 - y;
        w1 -= dx;
        h1 -= dy;
        if w1 <= 0 || h1 <= 0 {
            return Ok(());
        }
        if !self.using_partial_mode {
            self.init_part()?;
        }
        self.set_partial_ram_area(x1 as u16, y1 as u16, w1 as u16, h1 as u16)?;
        self.start_transfer();
        self.transfer_command(command)?;
        for i in 0..h1 {
            for j in 0..w1 / 8 {
                let mut data = 0u8;
                let idx = if mirror_y {
                    j + dx / 8 + (h - 1 - (i + dy)) * wb
                } else {
                    j + dx / 8 + (i + dy) * wb
                };
                data = bitmap[idx as usize];
                if invert {
                    data = !data;
                }
                self.spi.transfer(&mut [data])?;
            }
        }
        self.end_transfer();

        Ok(())
    }

    fn init_full(&mut self) -> Result<(), spi::Error> {
        self.init_display()?;
        self.power_on()?;
        self.using_partial_mode = false;
        Ok(())
    }

    fn init_part(&mut self) -> Result<(), spi::Error> {
        self.init_display()?;
        self.power_on()?;
        self.using_partial_mode = true;
        Ok(())
    }

    fn init_display(&mut self) -> Result<(), spi::Error> {
        // TODO:   if (_hibernating) _reset();

        self.start_transfer();
        self.transfer_command(0x01)?;
        self.spi.transfer(&mut [0xC7, 0x00, 0x00])?;

        // TODO: if(reduceBoosterTime) {...}

        self.transfer_command(0x18)?;
        self.spi.transfer(&mut [0x80])?;
        self.end_transfer();

        self.set_dark_border(false)?;

        self.set_partial_ram_area(SCREEN_RECT)?;

        Ok(())
    }

    fn power_on(&mut self) -> Result<(), spi::Error> {
        //TODO: if(waitingPowerOn)
        if self.power_is_on {
            return Ok(());
        }

        self.start_transfer();
        self.transfer_command(0x22)?;
        self.spi.transfer(&mut [0xf8])?;
        self.transfer_command(0x20)?;
        self.end_transfer();
        self.wait_while_busy("power_on")?;
        self.power_is_on = true;

        Ok(())
    }

    fn set_dark_border(&mut self, dark_border: bool) -> Result<(), spi::Error> {
        //TODO: if(_hibernating)return;
        self.start_transfer();
        self.transfer_command(0x3C)?;
        self.spi
            .transfer(&mut [if dark_border { 0x02 } else { 0x05 }])?;
        self.end_transfer();

        Ok(())
    }

    fn refresh_all(&mut self, partial_update_mode: bool) -> Result<(), spi::Error> {
        if partial_update_mode {
            self.refresh(SCREEN_RECT)?;
        } else {
            if self.using_partial_mode {
                self.init_full()?;
            }
            self.update_full()?;
            self.initial_refresh = false;
        }

        Ok(())
    }

    fn refresh(&mut self, rect: Rect) -> Result<(), spi::Error> {
        if self.initial_refresh {
            return self.refresh_all(false);
        }
        let rect = rect.intersection(&SCREEN_RECT);
        let Some(rect) = rect else {
            return Ok(());
        };
        let rect = Rect {
            x: Span {
                lo: floor_multiple(rect.x.lo, 8),
                hi: ceil_multiple(rect.x.hi, 8),
            },
            y: rect.y,
        };
        if !self.using_partial_mode {
            self.init_part()?;
        }
        self.set_partial_ram_area(rect)?;
        self.update_part()?;

        Ok(())
    }

    fn update_full(&mut self) -> Result<(), spi::Error> {
        self.start_transfer();
        self.transfer_command(0x22)?;
        self.spi.transfer(&mut [0xf4])?;
        self.transfer_command(0x20)?;
        self.end_transfer();
        self.wait_while_busy("update_full")?;

        Ok(())
    }

    fn update_part(&mut self) -> Result<(), spi::Error> {
        self.start_transfer();
        self.transfer_command(0x22)?;
        self.spi.transfer(&mut [0xfc])?;
        self.transfer_command(0x20)?;
        self.end_transfer();
        self.wait_while_busy("update_part")?;

        Ok(())
    }

    fn set_partial_ram_area(&mut self, rect: Rect) -> Result<(), spi::Error> {
        self.start_transfer();
        self.transfer_command(0x11)?;
        self.spi.transfer(&mut [0x03])?;
        self.transfer_command(0x44)?;
        self.spi
            .transfer(&mut [(rect.x.lo / 8) as u8, (rect.x.size() / 8) as u8])?;
        self.transfer_command(0x45)?;
        self.spi.transfer(&mut [
            (rect.y.lo % 256) as u8,
            (rect.y.lo / 256) as u8,
            (rect.y.size() % 256) as u8,
            (rect.y.size() % 256) as u8,
        ])?;
        self.transfer_command(0x4e)?;
        self.spi.transfer(&mut [(rect.x.lo / 8) as u8])?;
        self.transfer_command(0x4f)?;
        self.spi
            .write_bytes(&mut [(rect.y.lo % 256) as u8, (rect.y.lo / 256) as u8])?;
        self.end_transfer();

        Ok(())
    }

    fn write_screen_buffer(&mut self, value: u8) -> Result<(), spi::Error> {
        if !self.using_partial_mode {
            self.init_part()?;
        }
        if self.initial_write {
            self.write_screen_buffer_inner(0x26, value)?;
        }
        self.write_screen_buffer_inner(0x24, value)?;
        self.initial_write = false;

        Ok(())
    }

    fn write_screen_buffer_again(&mut self, value: u8) -> Result<(), spi::Error> {
        if !self.using_partial_mode {
            self.init_part()?;
        }
        self.write_screen_buffer_inner(0x24, value)?;

        Ok(())
    }

    fn write_screen_buffer_inner(&mut self, command: u8, value: u8) -> Result<(), spi::Error> {
        self.start_transfer();
        self.transfer_command(command)?;
        for _ in 0..WIDTH * HEIGHT / 8 {
            self.spi.transfer(&mut [value])?;
        }
        self.end_transfer();

        Ok(())
    }

    fn wait_while_busy(&mut self, debug: &'static str) -> Result<(), spi::Error> {
        self.delay.delay(1.millis());
        let start = current_time();
        loop {
            if self.busy.is_low() {
                break;
            }
            self.busy_callback();
            println!(
                "{debug} Back from sleep! {:?}ms",
                current_time()
                    .checked_duration_since(start)
                    .map(|v| v.to_millis())
            );
            if self.busy.is_low() {
                break;
            }
            let busy_timeout = 10000000.micros();
            if current_time().checked_duration_since(start) > Some(busy_timeout) {
                println!("Busy timeout!");
                break;
            }
            // wdt.feed() if wdt is enabled
        }

        Ok(())
    }

    fn busy_callback(&mut self) {
        if false {
            self.busy.wakeup_enable(true, WakeEvent::LowLevel);
            self.rtc.sleep_light(&[&GpioWakeupSource::new()]);
        } else {
            self.delay.delay(1.millis());
        }
    }

    fn transfer_command(&mut self, value: u8) -> Result<(), spi::Error> {
        self.dc.set_low();
        self.spi.transfer(&mut [value])?;
        self.dc.set_high();
        Ok(())
    }

    fn start_transfer(&mut self) {
        self.cs.set_low();
    }

    fn end_transfer(&mut self) {
        self.cs.set_high();
    }
}

fn floor_multiple(n: i16, m: i16) -> i16 {
    n - n % m
}

fn ceil_multiple(n: i16, m: i16) -> i16 {
    n + if n % m > 0 { m - n % m } else { 0 }
}
