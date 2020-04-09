//! simple time counter, controlled by an IRQ

#![no_std]
#![no_main]

extern crate cortex_m;
extern crate cortex_m_rt as rt;
extern crate panic_halt;
extern crate stm32f0xx_hal as hal;

use cortex_m_rt::entry;
use ssd1306::{prelude::*, Builder as SSD1306Builder};

use core::fmt;
use core::fmt::Write;
use arrayvec::ArrayString;

use core::cell::{Cell, RefCell};
use cortex_m::{peripheral::Peripherals as c_m_Peripherals};

use cortex_m::interrupt::{free, Mutex};

use crate::hal::{
    prelude::*,
    i2c::I2c,
    delay::Delay,
    stm32::{self, interrupt, Interrupt, Peripherals, TIM3},
    time::Hertz,
    timers::*,
};

//globally accessible counter value

static COUNTER: Mutex<Cell<u16>> = Mutex::new(Cell::new(0u16));

//globally accessible timer and display peripherals

static GTIMER: Mutex<RefCell<Option<Timer<TIM3>>>> = Mutex::new(RefCell::new(None));

static GDISPLAY: Mutex<RefCell<Option<ssd1306::mode::terminal::TerminalMode
    <ssd1306::interface::i2c::I2cInterface<hal::i2c::I2c<hal::stm32::I2C1, 
    hal::gpio::gpiob::PB8<hal::gpio::Alternate<hal::gpio::AF1>>, 
    hal::gpio::gpiob::PB7<hal::gpio::Alternate<hal::gpio::AF1>>>>>>>> = Mutex::new(RefCell::new(None));


//delay necessary for the I2C to initiate correctly and start on boot without having to reset the board

const BOOT_DELAY_MS: u16 = 200;



#[interrupt]

fn TIM3() {
        
    static mut DISPLAY: Option<ssd1306::mode::terminal::TerminalMode
    <ssd1306::interface::i2c::I2cInterface<hal::i2c::I2c<hal::stm32::I2C1, 
    hal::gpio::gpiob::PB8<hal::gpio::Alternate<hal::gpio::AF1>>, 
    hal::gpio::gpiob::PB7<hal::gpio::Alternate<hal::gpio::AF1>>>>>> = None;
    
    static mut TIMER: Option<Timer<TIM3>> = None;

    let int = TIMER.get_or_insert_with(|| {
        cortex_m::interrupt::free(|cs| {
            // Move TIMER here, leaving a None in its place
            GTIMER.borrow(cs).replace(None).unwrap()
        })
    });
    
    let disp = DISPLAY.get_or_insert_with(|| {
        cortex_m::interrupt::free(|cs| {
            // Move DISPLAY here, leaving a None in its place
            GDISPLAY.borrow(cs).replace(None).unwrap()
        })
    });
        
    let counter = free(|cs| COUNTER.borrow(cs).get());

    let mut output = ArrayString::<[u8; 64]>::new();
    
    format_time(&mut output, counter);

    disp.write_str(output.as_str());

    free(|cs| COUNTER.borrow(cs).replace(COUNTER.borrow(cs).get() + 1));

    int.wait().ok();

}


#[entry]
fn main() -> ! {

    let mut p = Peripherals::take().unwrap();
    let mut cp = c_m_Peripherals::take().unwrap();
        
    cortex_m::interrupt::free(move |cs| {
    
        let mut rcc = p.RCC.configure().sysclk(8.mhz()).freeze(&mut p.FLASH);
            
        let mut delay = Delay::new(cp.SYST, &rcc);
    
    
        delay.delay_ms(BOOT_DELAY_MS);
        
        let gpiob = p.GPIOB.split(&mut rcc);
        let scl = gpiob.pb8.into_alternate_af1(cs);
        let sda = gpiob.pb7.into_alternate_af1(cs);
        let i2c = I2c::i2c1(p.I2C1, (scl, sda), 400.khz(), &mut rcc);
            
        // Set up a timer expiring after 1s
        let mut timer = Timer::tim3(p.TIM3, Hertz(1), &mut rcc);
            
        timer.listen(Event::TimeOut);

        // Move the timer into the global storage
        *GTIMER.borrow(cs).borrow_mut() = Some(timer);
            

        // Set up the display
            
        let mut disp: TerminalMode<_> = SSD1306Builder::new().size(DisplaySize::Display128x32).connect_i2c(i2c).into();
            
        disp.init().unwrap();

        disp.clear().unwrap();

        // move the display into the global storage

        *GDISPLAY.borrow(cs).borrow_mut() = Some(disp);

        let mut nvic = cp.NVIC;
        unsafe {
            nvic.set_priority(Interrupt::TIM3, 1);
            cortex_m::peripheral::NVIC::unmask(Interrupt::TIM3);
              
            }
            cortex_m::peripheral::NVIC::unpend(Interrupt::TIM3);

        });
    
    loop {continue;}
    
}

   
// helper function to convert seconds to hours, minutes and seconds    

fn format_time(buf: &mut ArrayString<[u8; 64]>, elapsed: u16) {
    
    let (e_hrs, e_mins, e_secs) = time_digits(elapsed);
    
    fmt::write(buf, format_args!("    {:02}:{:02}:{:02}                                                    ",
    e_hrs, e_mins, e_secs)).unwrap();
}

// helper function to convert seconds to hours, minutes and seconds    

fn time_digits(time: u16) -> (u8, u8, u8) {
    
    let mut hours = time / 3600;
    
    let mut minutes = time / 60;
    minutes = minutes % 60;
    let seconds = time % 60;

    (hours as u8, minutes as u8, seconds as u8)
}

