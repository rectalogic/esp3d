#![no_std]

use embedded_graphics::{
    Drawable,
    geometry::Point,
    pixelcolor::{Rgb565, RgbColor},
    prelude::{Primitive, Transform},
    primitives::{PrimitiveStyleBuilder, Triangle},
};

pub async fn render(mut swapchain: esp3d::SwapChain, touch_receiver: esp3d::TouchReceiver) -> ! {
    let triangle = Triangle::new(Point::new(-3, 3), Point::new(3, 3), Point::new(0, -3))
        .into_styled(
            PrimitiveStyleBuilder::new()
                .stroke_color(Rgb565::GREEN)
                .stroke_width(1)
                .fill_color(Rgb565::RED)
                .build(),
        );
    let mut last_touch = (None, None);
    loop {
        let touch = touch_receiver.receive().await;
        if touch == last_touch {
            continue;
        }
        let present = draw2(triangle, touch, swapchain.back_buffer());
        //XXX swapchain no good for persistent graphics, have to draw twice
        if present {
            swapchain.present().await;
            draw2(triangle, touch, swapchain.back_buffer());
        }
        last_touch = touch;
    }
}

fn draw2<T>(triangle: T, touch: esp3d::TouchEvent, framebuffer: &mut esp3d::FrameBuffer) -> bool
where
    T: Transform + Drawable<Color = Rgb565, Output = ()> + Copy,
{
    let d0 = draw(triangle, touch.0, framebuffer);
    let d1 = draw(triangle, touch.1, framebuffer);
    d0 || d1
}

fn draw<T>(triangle: T, p: Option<esp3d::TouchPoint>, framebuffer: &mut esp3d::FrameBuffer) -> bool
where
    T: Transform + Drawable<Color = Rgb565, Output = ()> + Copy,
{
    if let Some(p) = p {
        triangle
            .translate(Point::new(p.x as i32, p.y as i32))
            .draw(framebuffer);
        true
    } else {
        false
    }
}
