#![no_std]
#![no_main]

//use cortex_m_rt::entry;

use stm32l4xx_hal::{pac, prelude::*};
use stm32l4xx_hal::serial::{Config, Event, Rx, Serial};

use defmt_rtt as _;
use panic_probe as _; // panic handler // global logger

//import image
use tp_led_matrix::image::Image;

mod matrix;
use matrix::Matrix;

// import DwtSystick
use dwt_systick_monotonic::DwtSystick;
use dwt_systick_monotonic::ExtU32;

use heapless::pool::{Box, Node, Pool};

use core::mem::{swap};
use core::borrow::BorrowMut;

use ibm437::IBM437_8X8_REGULAR;

use embedded_graphics::{
    //primitives::{Circle, Line, Primitive, PrimitiveStyleBuilder, Rectangle, Triangle},
    prelude::*,
    //mono_font::MonoTextStyleBuilder,
    mono_font::MonoTextStyle,
    text::Text,
    //draw_target::DrawTarget,
    pixelcolor::Rgb888,
};

#[rtic::app(device = pac, dispatchers = [USART2, USART3])]
mod app {
    use core::mem::MaybeUninit;
    use stm32l4xx_hal::device::{USART1};

    use super::*;

    #[monotonic(binds = SysTick, default = true)]
    type MyMonotonic = DwtSystick<80_000_000>; // 80MHz
    type Instant = <MyMonotonic as rtic::Monotonic>::Instant; // alias for the monotonic timer

    #[shared]
    struct Shared {
        //image: Image,
        next_image: Option<Box<Image>>, // the next image to display if one is ready
        pool: Pool<Image>, // the pool from which to draw or return Box<Image> buffers
        changes: u32
    }

    #[local]
    struct Local {
        matrix: Matrix,
        usart1_rx: Rx<USART1>,
        current_image: Box<Image>,
        rx_image: Box<Image>
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        defmt::info!("Starting dfmt initialization");

        let mut cp = cx.core; // Cortex-M peripherals
        let dp = cx.device; // Device specific peripherals
        
        let mut mono = DwtSystick::new(&mut cp.DCB, cp.DWT, cp.SYST, 80_000_000);

        // Initialize the clocks, hardware and matrix using your existing code

        // Get high-level representations of hardware modules
        let mut rcc = dp.RCC.constrain(); // Reset and Clock Control
        let mut flash = dp.FLASH.constrain(); // Flash memory
        let mut pwr = dp.PWR.constrain(&mut rcc.apb1r1); // Power control

        // Setup the clocks at 80MHz using HSI (by default since HSE/MSI are not configured).
        // The flash wait states will be configured accordingly.
        let clocks = rcc.cfgr.sysclk(80.MHz()).freeze(&mut flash.acr, &mut pwr); // Clocks

        // Define pins
        let mut gpioa = dp.GPIOA.split(&mut rcc.ahb2);
        let mut gpiob = dp.GPIOB.split(&mut rcc.ahb2);
        let mut gpioc = dp.GPIOC.split(&mut rcc.ahb2);

        // Configure PB6 and PB7 in alternate mode for USART1
        let tx_pin = gpiob.pb6
        .into_alternate::<7>(&mut gpiob.moder, &mut gpiob.otyper, &mut gpiob.afrl);

        let rx_pin = gpiob.pb7
        .into_alternate::<7>(&mut gpiob.moder, &mut gpiob.otyper, &mut gpiob.afrl);

        // Instantiate a Config structure using the default value
        let config = Config::default();

        // Set the baudrate to 38400 bits per second
        let config = config.baudrate(38400.bps());

        // Initializate the serial port
        let mut serial = Serial::usart1(
            dp.USART1,
            (tx_pin, rx_pin),
            config,
            clocks,
            &mut rcc.apb2,
        );

        // Enable the interrupt for the RXNE event
        serial.listen(Event::Rxne);

        // Get the serial port's RX and TX pins
        let (_, usart1_rx) = serial.split();


        // Create matrix: I didn't understand why I don't had to modify gpioa.moder, gpioa.otyper, gpiob.moder, gpiob.otyper, gpioc.moder, gpioc.otyper
        let matrix = Matrix::new(
            gpioa.pa2,
            gpioa.pa3,
            gpioa.pa4,
            gpioa.pa5,
            gpioa.pa6,
            gpioa.pa7,
            gpioa.pa15,
            gpiob.pb0,
            gpiob.pb1,
            gpiob.pb2,
            gpioc.pc3,
            gpioc.pc4,
            gpioc.pc5,
            &mut gpioa.moder,
            &mut gpioa.otyper,
            &mut gpiob.moder,
            &mut gpiob.otyper,
            &mut gpioc.moder,
            &mut gpioc.otyper,
            clocks,
        );

        //let image = Image::new_solid(image::BLACK); // es necesario inicializarla?

        defmt::info!("Finishing dfmt initialization");
        display::spawn(mono.now()).unwrap(); // Spawn the display task: it will run immediately
        defmt::info!("Display function spawned");

        // Create the pool
        let pool: Pool<Image> = Pool::new();
        unsafe {
            static mut MEMORY: MaybeUninit<[Node<Image>; 3]> = MaybeUninit::uninit();
            pool.grow_exact(&mut MEMORY); // unsafe to access a static mut

        }
        let current_image = pool.alloc().unwrap().init(Image::default());
        let rx_image = pool.alloc().unwrap().init(Image::default());

        let next_image = None;

        let changes = 0;

        // Spawn the screensaver task
        screensaver::spawn(mono.now()).unwrap();
        
        // Return the resources and the monotonic timer
        (Shared { 
            next_image,
            pool,
            changes
        },
        Local { 
            matrix, 
            usart1_rx, 
            current_image, 
            rx_image 
        },
        init::Monotonics(mono))
    }

    #[task(local = [matrix, current_image, next_line: usize = 0], shared = [next_image, &pool], priority = 2)]
    fn display(mut cx: display::Context, at: Instant) { 
        // Display line next_line (cx.local.next_line) of
        // the image (cx.local.image) on the matrix (cx.local.matrix).
        // All those are mutable references.

        /* cx.shared.image.lock(|image| {
            cx.local.matrix.send_row(*cx.local.next_line, image.row(*cx.local.next_line));
        }); */
        let matrix = cx.local.matrix;
        let current_image = cx.local.current_image;
        let next_row = cx.local.next_line;
        let pool = cx.shared.pool;

        matrix.send_row(*next_row, current_image.row(*next_row));

        if *next_row == 7 {
            cx.shared.next_image.lock(|next_image| {
                if next_image.is_some() {
                    // take next image in a image
                    let mut image = next_image.take().unwrap(); // next_image will be None
                    swap(current_image, &mut image); // swap current_image and image. Now current_image contains the image that was in next_image
                    pool.free(image); // free the image
                }
            });
        }

        // Increment next_line up to 7 and wraparound to 0
        *next_row = (*next_row + 1) % 8; 

        // Schedule the next display task
        let next = at + 1.secs()/(8*60); // 8 lines, 60Hz
        display::spawn_at(next, next).unwrap(); // Spawn the display task: it will run immediately
    }

    #[task(binds = USART1,
            local = [usart1_rx, rx_image, next_pos: usize = 0],
            shared = [next_image, &pool])]
    fn receive_byte(mut cx: receive_byte::Context)
    {
        let next_pos: &mut usize = cx.local.next_pos;
        let rx_image: &mut Image = cx.local.rx_image;
        let pool = cx.shared.pool;

        if let Ok(b) = cx.local.usart1_rx.read() {
            // Handle the incoming byte according to the SE203 protocol
            // and update next_image
            // Do not forget that next_image.as_mut() might be handy here!
            let mut start = false;
            if b == 0xff {
                start = true;
            }
            if *next_pos > 0 {
                rx_image.as_mut()[*next_pos-1] = b;
                *next_pos += 1;
            } else if start {
                *next_pos += 1;
            }
            
            // If the received image is complete, make it available to
            // the display task.
            if *next_pos == 8 * 8 * 3 + 1 {
                // check if there is a next image
                cx.shared.next_image.lock(|next_image| {
                    if next_image.is_some(){
                        let image = next_image.take().unwrap(); // next_image will be None
                        pool.free(image); // free the image
                    }
                    // Obtain a future_image from the pool
                    let mut future_image = pool.alloc().unwrap().init(Image::default());
                    swap(rx_image, &mut future_image); // swap rx_image and future_image. Now rx_image contains the image that was in future_image
                    // Store the completed image in next_image
                    swap(next_image, Some(future_image).borrow_mut()); // swap next_image and future_image. Now next_image contains the image that was in future_image
                    *next_pos = 0;
                    
                    // spawn notice_change
                    notice_change::spawn().unwrap(); // spawn means that the task will be executed as soon as possible
                });
            }
        }
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
        }
    }

#[task(local = [last_changes: u32 = 0, offset: i32 = 20], shared = [changes, &pool, next_image])]
fn screensaver(cx: screensaver::Context, at: Instant) {
    let last_changes = cx.local.last_changes;
        let offset = cx.local.offset;
        let pool = cx.shared.pool;

        (cx.shared.changes, cx.shared.next_image).lock(|changes, next_image| {
            // If there are no changes, we can display the screensaver
            if *last_changes == *changes {
                if next_image.is_some() {
                    pool.free(next_image.take().unwrap());
                };
                let text = Text::new(
                    "Hola Mundo!",
                    Point::new(*offset, 7),
                    MonoTextStyle::new(&IBM437_8X8_REGULAR, Rgb888::RED),
                );
                let mut image = Image::default();
                text.draw(&mut image).unwrap();
                let screensaver_image = pool.alloc().unwrap().init(image);
                swap(next_image, Some(screensaver_image).borrow_mut());

                if *offset > -120 {
                    *offset -= 1;
                } else {
                    *offset = 30;
                }
            } else {
                *last_changes = *changes;
                *offset = 30;
            }
        });

        // Calculate new time of spawning
        let next: Instant = at + 60.millis();

        screensaver::spawn_at(next, next).unwrap();
}

    #[task(shared = [changes])]
    fn notice_change(mut cx: notice_change::Context) {
        cx.shared.changes.lock(|changes| {
            *changes = changes.wrapping_add(1); // wrapping add is used to avoid overflow
        });
    }
}
