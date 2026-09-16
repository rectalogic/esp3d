#![no_std]
#![no_main]

use embassy_executor::Spawner;

#[esp_rtos::main]
async fn main(_spawner: Spawner) -> ! {
    esp3d::app::app(triangle::render).await
}
