use esp_hal::{
    gpio::{AnyPin, Event, Input, InputConfig, Pull},
    i2c::master::{Config, I2c},
    peripherals::I2C0,
};
use ft6336u_driver::{FT6336U, GestureMode, TouchStatus};

pub struct Peripherals {
    pub i2c: I2C0<'static>,
    pub sda: AnyPin<'static>,
    pub scl: AnyPin<'static>,
    pub int: AnyPin<'static>,
    pub rst: AnyPin<'static>,
}

#[embassy_executor::task]
pub async fn touch_task(peripherals: Peripherals) {
    let i2c = I2c::new(peripherals.i2c, Config::default())
        .unwrap()
        .with_sda(peripherals.sda)
        .with_scl(peripherals.scl);
    let mut touch = FT6336U::new(i2c);
    touch.write_g_mode(GestureMode::Trigger).unwrap();
    let mut touch_int = Input::new(peripherals.int, InputConfig::default().with_pull(Pull::Up));
    touch_int.listen(Event::FallingEdge);
    loop {
        touch_int.wait_for_falling_edge().await;

        match touch.scan() {
            Ok(data) => {
                if data.touch_count > 0 {
                    for i in 0..data.touch_count as usize {
                        let point = &data.points[i];
                        match point.status {
                            TouchStatus::Touch => {
                                log::info!("Touch #{} started at ({}, {})", i, point.x, point.y);
                            }
                            TouchStatus::Stream => {
                                log::info!("Touch #{} at ({}, {})", i, point.x, point.y);
                            }
                            TouchStatus::Release => {
                                log::info!("Touch #{} ended", i);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                log::error!("Touch read error: {:?}", e);
            }
        }
    }
}
