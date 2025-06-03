use crate::Color;
use crate::Image;
use core::convert::Infallible;


use embedded_graphics::{
    draw_target::DrawTarget,
    pixelcolor::Rgb888,
    prelude::*,
};

impl From<Rgb888> for Color {
    fn from(color: Rgb888) -> Self {
        Self {
            r: color.r(),
            g: color.g(),
            b: color.b(),
        }
    }
}

impl DrawTarget for Image {
    type Color = Rgb888;
    type Error = Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>, 
    {
        for Pixel(coord, color) in pixels.into_iter() {
            if let Ok((x @ 0..=7, y @ 0..=7)) = coord.try_into() {
                self[(x as usize, y as usize)] = color.into();
            }
        }
        Ok(())
    }
}

impl OriginDimensions for Image {
    fn size(&self) -> Size {
        Size::new(8, 8)
    }
}