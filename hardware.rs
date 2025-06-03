use stm32f4xx_hal as hal; // Import the stm32f4xx_hal crate as hal
use hal::{rcc::Clocks, serial::{config, Serial}, stm32}; // Import the Clocks, Config, Serial, and stm32 structs from the stm32f4xx_hal crate

let p = stm32::Peripherals::take().unwrap(); // Take ownership of the peripheral singleton: only one instance of this struct can exist at a time
let rcc = p.RCC.constrain(); // Constrain the RCC register: only one instance of this struct can exist at a time
let clocks: Clocks = rcc // Get the RCC struct
    .cfgr // Get the clock configuration register
    .sysclk(64.mhz()) // Set the system clock to 64 MHz
    .pclk1(32.mhz()) // Set the peripheral clock 1 to 32 MHz
    .pclk2(64.mhz()) // Set the peripheral clock 2 to 64 MHz
    .freeze(); // Freeze the clock configuration and return the Clocks struct
let gpioa = dp.GPIOA.split(); // Split the GPIOA register into independent pins. dp: Device Peripherals
let tx_pin = gpioa.pa9.into_alternate_af7(); // Set the TX pin to alternate function 7
let rx_pin = gpioa.pa10.into_alternate_af7(); // Set the RX pin to alternate function 7
let usart1_config = config::Config { // Create a new configuration struct
    baudrate: 9_600.bps(), // Set the baud rate to 9,600 bps
    wordlength: config::WordLength::DataBits8, // Set the word length to 8 bits
    parity: config::Parity::ParityNone, // Set the parity to none
    stopbits: config::StopBits::STOP1, // Set the stop bits to 1
};

let usart1 = Serial::usart1( // Create a new serial interface
    p.USART1, // The USART1 peripheral. p: Peripherals
    (rx_pin, tx_pin), // The TX and RX pins
    usart1_config, // The configuration struct
    clocks, // The Clocks struct
).unwrap(); // Unwrap the Result struct

usart1.write(b'A').unwrap(); // Write the byte 'A' to the serial interface