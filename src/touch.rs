use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    channel::{Channel, Receiver, Sender},
};
use embassy_time::Timer;
use esp_hal::{
    gpio::{AnyPin, Event, Input, InputConfig, Level, Output, OutputConfig, Pull},
    i2c::master::{Config, I2c},
    peripherals::I2C0,
};
use ft6x36::{Dimension, Ft6x36, Orientation, TouchPoint};

pub type TouchEvent = (Option<TouchPoint>, Option<TouchPoint>);
const TOUCH_CAPACITY: usize = 25;
pub type TouchChannel = Channel<CriticalSectionRawMutex, TouchEvent, TOUCH_CAPACITY>;
pub type TouchSender = Sender<'static, CriticalSectionRawMutex, TouchEvent, TOUCH_CAPACITY>;
pub type TouchReceiver = Receiver<'static, CriticalSectionRawMutex, TouchEvent, TOUCH_CAPACITY>;

pub struct Peripherals {
    pub i2c: I2C0<'static>,
    pub sda: AnyPin<'static>,
    pub scl: AnyPin<'static>,
    pub int: AnyPin<'static>,
    pub rst: AnyPin<'static>,
}

#[embassy_executor::task]
pub async fn touch_task(peripherals: Peripherals, touch_sender: TouchSender) {
    let mut touch_rst = Output::new(peripherals.rst, Level::Low, OutputConfig::default());
    Timer::after_millis(10).await;
    touch_rst.set_high();
    Timer::after_millis(300).await;

    let i2c = I2c::new(peripherals.i2c, Config::default())
        .unwrap()
        .with_sda(peripherals.sda)
        .with_scl(peripherals.scl);
    let mut touch = Ft6x36::new(i2c, Dimension(240, 320));
    touch.set_orientation(Orientation::Landscape);
    let mut touch_int = Input::new(peripherals.int, InputConfig::default().with_pull(Pull::Up));
    touch_int.listen(Event::FallingEdge);

    loop {
        touch_int.wait_for_falling_edge().await;

        loop {
            match touch.get_touch_event() {
                Ok(event) => {
                    touch_sender.send((event.p1, event.p2)).await;
                    if event.p1.is_none() && event.p2.is_none() {
                        break;
                    }
                }
                Err(e) => {
                    log::error!("Touch read error: {:?}", e);
                }
            }
            Timer::after_millis(10).await;
        }
        Timer::after_millis(20).await;
    }
}
