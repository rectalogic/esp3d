use embassy_time::{Duration, Timer};
use embedded_graphics::{
    Drawable,
    geometry::Point,
    pixelcolor::{Rgb565, RgbColor},
    prelude::{Primitive, Transform},
    primitives::{PrimitiveStyleBuilder, Triangle},
};

use crate::swapchain::{FrameBuffer, present_buffer};

pub async fn _render(mut framebuffer: FrameBuffer) -> ! {
    let mut triangle = Triangle::new(Point::new(0, 0), Point::new(50, 0), Point::new(25, 50))
        .into_styled(
            PrimitiveStyleBuilder::new()
                .stroke_color(Rgb565::GREEN)
                .stroke_width(3)
                .fill_color(Rgb565::RED)
                .build(),
        );
    let mut amount = 0;
    let mut step = 5;
    loop {
        triangle.translate_mut(Point::new(0, step));
        amount += step;
        if !(0..240).contains(&amount) {
            step = -step;
        };
        triangle.draw(&mut framebuffer);
        framebuffer = present_buffer(framebuffer).await;
        framebuffer.reset();
        Timer::after(Duration::from_millis(30)).await;
    }
}
