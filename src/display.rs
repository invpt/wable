use esp_hal::{
    delay::Delay, gpio::{GpioPin, Input, Level, Output, Pull, WakeEvent}, peripheral::Peripheral, peripherals::{SPI0, SPI1, SPI2, SPI3}, prelude::*, rtc_cntl::{sleep::GpioWakeupSource, Rtc}, spi::{self, master::Spi, FullDuplexMode}, time::current_time
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

pub struct Display<'d> {
    pub power_is_on: bool,
    pub using_partial_mode: bool,
    pub initial_refresh: bool,
    pub initial_write: bool,
    pub pulldown_rst_mode: bool,
    pub delay: Delay,
    pub rtc: Rtc<'d>,
    pub spi: Spi<'d, SPI2, FullDuplexMode>,
    pub cs: Output<'d, GpioPin<5>>,
    pub dc: Output<'d, GpioPin<10>>,
    pub busy: Input<'d, GpioPin<19>>,
    pub rst: GpioPin<9>,
    pub rst_in: Option<Input<'d, GpioPin<9>>>,
}

impl<'d> Display<'d> {
    pub fn init(&mut self) -> Result<(), spi::Error> {
        self.cs.set_high();
        self.reset()?;
        Ok(())
    }

    pub fn reset(&mut self) -> Result<(), spi::Error> {
        if self.pulldown_rst_mode {
            let mut rst_out = Output::new(unsafe { self.rst.clone_unchecked() }, Level::Low);
            rst_out.set_low();
            self.delay.delay(10.millis());
            drop(rst_out);
            let mut rst_in = Input::new(unsafe { self.rst.clone_unchecked() }, Pull::Up);
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

    pub fn draw_image(&mut self, bitmap: &[u8], x: i16, y: i16, w: i16, h: i16, invert: bool, mirror_y: bool, pgm: bool) -> Result<(), spi::Error> {
        self.write_image(bitmap, x, y, w, h, invert, mirror_y, pgm)?;
        self.refresh(x, y, w, h)?;
        self.write_image_again(bitmap, x, y, w, h, invert, mirror_y, pgm)?;

        Ok(())
    }

    pub fn write_image(&mut self, bitmap: &[u8], x: i16, y: i16, w: i16, h: i16, invert: bool, mirror_y: bool, pgm: bool) -> Result<(), spi::Error> {
        self.write_image_inner(0x24, bitmap, x, y, w, h, invert, mirror_y, pgm)?;
        Ok(())
    }

    pub fn write_image_again(&mut self, bitmap: &[u8], x: i16, y: i16, w: i16, h: i16, invert: bool, mirror_y: bool, pgm: bool) -> Result<(), spi::Error> {
        self.write_image_inner(0x24, bitmap, x, y, w, h, invert, mirror_y, pgm)?;
        Ok(())
    }

    fn write_image_inner(&mut self, command: u8, bitmap: &[u8], mut x: i16, y: i16, mut w: i16, h: i16, invert: bool, mirror_y: bool, pgm: bool) -> Result<(), spi::Error> {
        if self.initial_write {
            self.write_screen_buffer(0xFF)?;
        }

        let wb = (w + 7) / 8;
        x -= x % 8;
        w = wb * 8;
        let x1 = x.max(0);
        let y1 = y.max(0);
        let mut w1 = if x + w < WIDTH as i16 { w } else { WIDTH as i16 - x };
        let mut h1 = if y + h < HEIGHT as i16 { h } else { HEIGHT as i16 - y };
        let dx = x1 - x;
        let dy = y1 - y;
        w1 -= dx;
        h1 -= dy;
        if w1 <= 0 || h1 <= 0 {
            return Ok(());
        }
        if (!self.using_partial_mode) {
            self.init_part()?;
        }
        self.set_partial_ram_area(x1 as u16, y1 as u16, w1 as u16, h1 as u16)?;
        self.start_transfer();
        self.transfer_command(command)?;
        for i in 0..h1 {
            for j in 0..w1 / 8 {
                let mut data = 0u8;
                let idx = if mirror_y { j + dx / 8 + ((h - 1 - (i + dy))) * wb } else { j + dx / 8 + (i + dy) * wb };
                data = bitmap[idx as usize];
                if invert {
                    data = !data;
                }
                self.spi.transfer(&mut[data])?;
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

        self.set_partial_ram_area(0, 0, WIDTH as u16, HEIGHT as u16)?;

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
            self.refresh(0, 0, WIDTH as i16, HEIGHT as i16)?;
        } else {
            if self.using_partial_mode {
                self.init_full()?;
            }
            self.update_full()?;
            self.initial_refresh = false;
        }

        Ok(())
    }

    fn refresh(&mut self, x: i16, y: i16, w: i16, h: i16) -> Result<(), spi::Error> {
        if self.initial_refresh {
            return self.refresh_all(false);
        }
        let mut w1 = if x < 0 { w + x } else { w };
        let mut h1 = if y < 0 { h + y } else { h };
        let mut x1 = if x < 0 { 0 } else { x };
        let y1 = if y < 0 { 0 } else { y };
        w1 = if x1 + w1 < WIDTH as i16 {
            w1
        } else {
            WIDTH as i16 - x1
        };
        h1 = if y1 + h1 < HEIGHT as i16 {
            h1
        } else {
            HEIGHT as i16 - y1
        };
        if w1 <= 0 || h1 <= 0 {
            return Ok(());
        }
        w1 += x1 % 8;
        if w1 % 8 > 0 {
            w1 += 8 - w1 % 8;
        }
        x1 -= x1 % 8;
        if !self.using_partial_mode {
            self.init_part()?;
        }
        self.set_partial_ram_area(x1 as u16, y1 as u16, w1 as u16, h1 as u16)?;
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

    fn set_partial_ram_area(&mut self, x: u16, y: u16, w: u16, h: u16) -> Result<(), spi::Error> {
        self.start_transfer();
        self.transfer_command(0x11)?;
        self.spi.transfer(&mut [0x03])?;
        self.transfer_command(0x44)?;
        self.spi
            .transfer(&mut [(x / 8) as u8, ((x + w - 1) / 8) as u8])?;
        self.transfer_command(0x45)?;
        self.spi.transfer(&mut [
            (y % 256) as u8,
            (y / 256) as u8,
            ((y + h - 1) % 256) as u8,
            ((y + h - 1) % 256) as u8,
        ])?;
        self.transfer_command(0x4e)?;
        self.spi.transfer(&mut [(x / 8) as u8])?;
        self.transfer_command(0x4f)?;
        self.spi.transfer(&mut [(y % 256) as u8, (y / 256) as u8])?;
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
            println!("{debug} Back from sleep! {:?}ms", current_time().checked_duration_since(start).map(|v| v.to_millis()));
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
