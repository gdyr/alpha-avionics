use ism330dhcx::*;

pub fn boot_imu<I2C, E>(imu: &mut Ism330Dhcx, i2c: &mut I2C)
where
    I2C: embedded_hal::blocking::i2c::WriteRead<Error = E>
        + embedded_hal::blocking::i2c::Write<Error = E>,
    E: std::fmt::Debug,
{
    // =======================================
    // CTRL3_C

    imu.ctrl3c.set_boot(i2c, true).unwrap();
    imu.ctrl3c.set_bdu(i2c, true).unwrap();
    imu.ctrl3c.set_if_inc(i2c, true).unwrap();

    // =======================================
    // CTRL9_XL

    imu.ctrl9xl.set_den_x(i2c, true).unwrap();
    imu.ctrl9xl.set_den_y(i2c, true).unwrap();
    imu.ctrl9xl.set_den_z(i2c, true).unwrap();
    imu.ctrl9xl.set_device_conf(i2c, true).unwrap();

    // =======================================
    // CTRL1_XL

    imu
        .ctrl1xl
        .set_accelerometer_data_rate(i2c, ctrl1xl::Odr_Xl::Hz52)
        .unwrap();

    imu
        .ctrl1xl
        .set_chain_full_scale(i2c, ctrl1xl::Fs_Xl::G4)
        .unwrap();
    imu.ctrl1xl.set_lpf2_xl_en(i2c, true).unwrap();

    // =======================================
    // CTRL2_G

    imu
        .ctrl2g
        .set_gyroscope_data_rate(i2c, ctrl2g::Odr::Hz52)
        .unwrap();

    imu
        .ctrl2g
        .set_chain_full_scale(i2c, ctrl2g::Fs::Dps500)
        .unwrap();

    // =======================================
    // CTRL7_G

    imu.ctrl7g.set_g_hm_mode(i2c, true).unwrap();
}