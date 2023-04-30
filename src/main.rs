use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported

use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::{prelude::Peripherals, i2c::{I2cConfig, I2cDriver}};
use esp_idf_hal::gpio::PinDriver;

use ism330dhcx::*;

mod max17048;
use crate::max17048::Max17048;

mod bmp280;
use crate::bmp280::BMP280;

mod imu;
use crate::imu::boot_imu;

mod espnow;

use serde::{Serialize, Deserialize};

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
struct Sample {

    imu_temperature: f32,
    imu_gyroscope: [f64; 3],
    imu_accelerometer: [f64; 3],

    baro_temperature: f64,
    baro_pressure: f64,

    bat_voltage: f32

}

// fn flash_write(buffer: &[u8], address: u32, length: u32) -> Result<(), esp_idf_sys::EspError> {
//     unsafe { esp_idf_sys::esp_flash_write(std::ptr::null::<esp_idf_sys::esp_flash_t>() as *mut _, buffer.as_ptr() as *mut _, address, length); }
//     Ok(())
// }

fn main() {
    esp_idf_sys::link_patches();

    let peripherals = Peripherals::take().expect("Failed to get peripherals.");
    let sda = peripherals.pins.gpio3;
    let scl = peripherals.pins.gpio4;

    let gpio9_driver = PinDriver::input(peripherals.pins.gpio9).expect("Could not create GPIO9 driver.");
    println!("Waiting for GPIO9 signal...");
    FreeRtos::delay_ms(1000u32);
    println!("GPIO 9 state: {:?}", gpio9_driver.is_high());

    let mut i2c_power = PinDriver::output(peripherals.pins.gpio7).expect("Could not create I2C power driver.");
    i2c_power.set_high().expect("Failed to turn on I2C power.");

    println!("Starting I2C");
    let config = I2cConfig::new().baudrate(400_000.into());
    let mut i2c = I2cDriver::new(peripherals.i2c0, sda, scl, &config).expect("Failed to init I2C driver");
    println!("I2C initialised.");

    // Configure IMU
    let mut imu = Ism330Dhcx::new(&mut i2c).expect("Failed to init IMU");
    println!("IMU initialised.");

    boot_imu(&mut imu, &mut i2c);
    println!("IMU booted.");

    // Configure barometer
    let mut ps = BMP280::new_with_address(&mut i2c, 0x77u8).expect("Failed to init barometer");
    ps.reset(&mut i2c);
    ps.set_control(&mut i2c, bmp280::Control { osrs_t: bmp280::Oversampling::x1,
                                     osrs_p: bmp280::Oversampling::x1,
                                     mode: bmp280::PowerMode::Normal });
    println!("Barometer initialised.");
    /*println!("Baro ID: {}\r\n", ps.id(&mut i2c));
    println!("Baro Control: {:?}\r\n", ps.control(&mut i2c));
    println!("Baro Config: {:?}\r\n", ps.config(&mut i2c));*/

    // Configure battery voltage monitor
    let mut max17048 = Max17048::new(&mut i2c);

    // Configure wifi / radio link
    let radio = espnow::init(peripherals.modem);

    // Wait
    println!("Waiting for peripheral boot...");
    FreeRtos::delay_ms(1000u32);

    // Check status
    println!(
        "# BATTERY MONITOR\r\nVersion: {}]\r\n\r\n",
        max17048.version(&mut i2c).expect("Failed to get battery monitor version.")
    );

    println!(
        "# BAROMETER\r\nID: {:?}\r\n{:?}\r\n{:?}\r\n{:?}\r\n\r\n",
        ps.id(&mut i2c),
        ps.status(&mut i2c),
        ps.config(&mut i2c),
        ps.control(&mut i2c)
    );


    let mut sample = Sample::default();

    loop {

        sample.imu_temperature = imu.get_temperature(&mut i2c).expect("Failed to get temperature");
        sample.imu_gyroscope = imu.get_gyroscope(&mut i2c).expect("Failed to get gyroscope");
        sample.imu_accelerometer = imu.get_accelerometer(&mut i2c).expect("Failed to get accelerometer");

        sample.baro_temperature = ps.temp(&mut i2c);
        sample.baro_pressure = ps.pressure(&mut i2c);

        sample.bat_voltage = max17048.vcell(&mut i2c).expect("Failed to get battery voltage");

        //println!("{:?}", sample);
        let buf = &bincode::serialize(&sample).unwrap();
        //println!("{:?}", buf);

        if let Err(e) = radio.espnow.send(
            espnow::BROADCAST_ADDR.clone(),
            buf
        ) {
            println!("Failed to send message: {:?}", e);
        }

        //FreeRtos::delay_ms(1000u32);
        FreeRtos::delay_ms(10u32);
    }
}
