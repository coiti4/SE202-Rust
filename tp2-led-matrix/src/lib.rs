#![no_std]

use defmt_rtt as _; // global logger

// Reexport the Color and Image types at the top level of the library
pub use image::Color;
pub use image::Image;

pub mod embedded;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}

pub mod image {
    use core::ops::{Div, Mul};

    
    
    enum Result<T, E> {
        Ok(T),
        Err(E),
    }

    

    impl <T,E: core::fmt::Debug> Result<T,E> {
        fn unwrap(self) -> T {
            match self {
                Result::Ok(o) => o,
                Result::Err(e) => panic!("unwrap called on Err value: {:?}", e),
            }
        }
    }

    // 2.4.2.1 COLOR
    use crate::gamma; // for gamma_correct
    use micromath::F32Ext; // for sin/cos

    #[repr(C)] // to make sure the struct is laid out in memory in the same way as C
    #[derive(Clone, Copy, Default)]
    pub struct Color {
        pub r: u8,
        pub g: u8,
        pub b: u8,
    }

    pub const RED: Color = Color { r: 255, g: 0, b: 0 };
    pub const GREEN: Color = Color { r: 0, g: 255, b: 0 };
    pub const BLUE: Color = Color { r: 0, g: 0, b: 255 };
    pub const BLACK: Color = Color { r: 0, g: 0, b: 0 };

    impl Color {
        pub fn gamma_correct(&self) -> Self {
            //
            let r = gamma::gamma_correct(self.r);
            let g = gamma::gamma_correct(self.g);
            let b = gamma::gamma_correct(self.b);
            Self { r, g, b }
        }
    }

    impl core::ops::Mul<f32> for Color {
        type Output = Color;
        fn mul(self, rhs: f32) -> Color {
            let r = mul_u8(self.r, rhs);
            let g = mul_u8(self.g, rhs);
            let b = mul_u8(self.b, rhs);
            Color { r, g, b }
        }
    }
    fn mul_u8(x: u8, rhs: f32) -> u8 {
        let scaled = (x as f32 * rhs).round() as i32;
        if scaled < 0 {
            0
        } else if scaled > 255 {
            255
        } else {
            scaled as u8
        }
    }

    impl core::ops::Div<f32> for Color {
        type Output = Color;
        fn div(self, rhs: f32) -> Color {
            /* use function mul defined above */
            let inv = 1.0 / rhs;
            self.mul(inv)
        }
    }

    // 2.4.2.2 IMAGE
    #[repr(transparent)]

    #[derive(Clone, Copy)]
    pub struct Image([Color; 64]);

    impl Image {
        pub fn new_solid(color: Color) -> Self {
            Self([color; 64]) // 64 is the number of pixels and color is the color of the pixels
        }
        pub fn row(&self, row: usize) -> &[Color] {
            let start = row * 8;
            &self.0[start..start + 8]
        }
        pub fn gradient(color: Color) -> Self {
            let mut img = Self::new_solid(color);
            for row in 0..8 {
                for col in 0..8 {
                    let d = 1 + row * row + col;
                    img[(row, col)] = color.div(d as f32);
                }
            }
            img
        }
    }

    impl Default for Image {
        fn default() -> Self {
            Self::new_solid(Color::default()) // default color is black
        }
    }

    impl core::ops::Index<(usize, usize)> for Image {
        type Output = Color;
        fn index(&self, (x, y): (usize, usize)) -> &Color {
            let ind = y * 8 + x;
            &self.0[ind]
        }
    }

    impl core::ops::IndexMut<(usize, usize)> for Image {
        fn index_mut(&mut self, (x, y): (usize, usize)) -> &mut Color {
            let ind = y * 8 + x;
            &mut self.0[ind]
        }
    }

    impl AsRef<[u8; 192]> for Image {
        fn as_ref(&self) -> &[u8; 192] {
            unsafe { core::mem::transmute(self) }
        }
    }

    impl AsMut<[u8; 192]> for Image {
        fn as_mut(&mut self) -> &mut [u8; 192] {
            unsafe { core::mem::transmute(self) }
        }
    }
}

pub mod gamma {
    const GAMMA_TAB: [u8; 256] = [
        0x00, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x02, 0x02, 0x02, 0x02, 0x02,
        0x03, 0x03, 0x03, 0x03, 0x04, 0x04, 0x04, 0x04, 0x05, 0x05, 0x05, 0x06, 0x06, 0x06, 0x07,
        0x07, 0x08, 0x08, 0x08, 0x09, 0x09, 0x0a, 0x0a, 0x0b, 0x0b, 0x0b, 0x0c, 0x0c, 0x0d, 0x0d,
        0x0e, 0x0e, 0x0f, 0x0f, 0x10, 0x11, 0x11, 0x12, 0x12, 0x13, 0x13, 0x14, 0x15, 0x15, 0x16,
        0x16, 0x17, 0x18, 0x18, 0x19, 0x1a, 0x1a, 0x1b, 0x1c, 0x1c, 0x1d, 0x1e, 0x1e, 0x1f, 0x20,
        0x20, 0x21, 0x22, 0x23, 0x23, 0x24, 0x25, 0x26, 0x26, 0x27, 0x28, 0x29, 0x2a, 0x2a, 0x2b,
        0x2c, 0x2d, 0x2e, 0x2e, 0x2f, 0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x35, 0x36, 0x37, 0x38,
        0x39, 0x3a, 0x3b, 0x3c, 0x3d, 0x3e, 0x3f, 0x40, 0x41, 0x42, 0x42, 0x43, 0x44, 0x45, 0x46,
        0x47, 0x48, 0x49, 0x4a, 0x4c, 0x4d, 0x4e, 0x4f, 0x50, 0x51, 0x52, 0x53, 0x54, 0x55, 0x56,
        0x57, 0x58, 0x59, 0x5a, 0x5c, 0x5d, 0x5e, 0x5f, 0x60, 0x61, 0x62, 0x64, 0x65, 0x66, 0x67,
        0x68, 0x69, 0x6b, 0x6c, 0x6d, 0x6e, 0x6f, 0x71, 0x72, 0x73, 0x74, 0x75, 0x77, 0x78, 0x79,
        0x7a, 0x7c, 0x7d, 0x7e, 0x7f, 0x81, 0x82, 0x83, 0x85, 0x86, 0x87, 0x89, 0x8a, 0x8b, 0x8c,
        0x8e, 0x8f, 0x91, 0x92, 0x93, 0x95, 0x96, 0x97, 0x99, 0x9a, 0x9b, 0x9d, 0x9e, 0xa0, 0xa1,
        0xa2, 0xa4, 0xa5, 0xa7, 0xa8, 0xaa, 0xab, 0xac, 0xae, 0xaf, 0xb1, 0xb2, 0xb4, 0xb5, 0xb7,
        0xb8, 0xba, 0xbb, 0xbd, 0xbe, 0xc0, 0xc1, 0xc3, 0xc4, 0xc6, 0xc7, 0xc9, 0xca, 0xcc, 0xcd,
        0xcf, 0xd1, 0xd2, 0xd4, 0xd5, 0xd7, 0xd8, 0xda, 0xdc, 0xdd, 0xdf, 0xe0, 0xe2, 0xe4, 0xe5,
        0xe7, 0xe9, 0xea, 0xec, 0xee, 0xef, 0xf1, 0xf3, 0xf4, 0xf6, 0xf8, 0xf9, 0xfb, 0xfd, 0xfe,
        0xff,
    ];
    pub fn gamma_correct(x: u8) -> u8 {
        GAMMA_TAB[x as usize]
    }
}
