#![allow(dead_code)]
#![allow(unused_variables)]

use embedded_hal::blocking::i2c::{WriteRead, Write};
extern crate embedded_hal as ehal;

const MAX17048_DEFAULT_ADDR: u8 = 0x36;
const DEFAULT_RCOMP: u8 = 0x97;


pub struct Max17048 {
    address: u8,
    recv_buffer: [u8; 2]
}


impl Max17048
{

    pub fn new<E, I2C>(i2c: &mut I2C) -> Self
    where
    I2C: WriteRead<Error = E> + Write<Error = E>, E: std::fmt::Debug
    {
      Self::new_with_address(i2c, MAX17048_DEFAULT_ADDR)
    }

    pub fn new_with_address<E, I2C>(i2c: &mut I2C, address: u8) -> Self
    where
    I2C: WriteRead<Error = E> + Write<Error = E>, E: std::fmt::Debug
    {
      let mut max = Max17048 {
        recv_buffer: [0u8; 2],
        address
      };
      max.compensation(i2c, DEFAULT_RCOMP).unwrap();
      max
    }


    pub fn version<E, I2C>(&mut self, i2c: &mut I2C) -> Result<u16, E>
    where
    I2C: WriteRead<Error = E>
    {
        self.read(i2c, 0x08)
    }

    pub fn soc<I2C, E>(&mut self, i2c: &mut I2C) -> Result<u16, E>
    where
        I2C: WriteRead<Error = E>
    {
        match self.read(i2c, 0x04) {
            Ok(val) => Ok(val / 256),
            Err(e) => Err(e)
        }
    }

    /// Return C/Rate in %/hr
    pub fn charge_rate<I2C, E>(&mut self, i2c: &mut I2C) -> Result<f32, E>
    where
        I2C: WriteRead<Error = E>
    {
        match self.read(i2c, 0x16) {
            Ok(val) => { 
                Ok(val as f32 * 0.208)
            },
            Err(e) => Err(e)
        }
    }

    pub fn vcell<I2C, E>(&mut self, i2c: &mut I2C) -> Result<f32, E>
    where
        I2C: WriteRead<Error = E>
    {
        match self.read(i2c, 0x02) {
            Ok(val) => Ok(val as f32 * 0.000078125),
            Err(e) => Err(e)
        }
    }

    pub fn temp_compensation<I2C, E>(&mut self, i2c: &mut I2C, temp: f32) -> Result<(), E>
    where
        I2C: WriteRead<Error = E> + Write<Error = E>
    {
        let rcomp = if temp > 20.0 {
            DEFAULT_RCOMP as f32 + (temp - 20.0) * -0.5
        } else {
            DEFAULT_RCOMP as f32 + (temp - 20.0) * -5.0
        };
        self.compensation(i2c, rcomp as u8)
    }

    fn compensation<I2C, E>(&mut self, i2c: &mut I2C, rcomp: u8) -> Result<(), E>
    where
        I2C: WriteRead<Error = E> + Write<Error = E>
    {
        // read the current reg vals
        match self.read(i2c, 0x0C) {
            Ok(mut value) => {
                value &= 0x00FF;
                value |= (rcomp as u16) << 8;
                // write to the rcomp bits only
                self.write(i2c, 0x0C, value)?;
                Ok(())
            },
            Err(e) => Err(e)
        }
    }

    fn read<I2C, E>(&mut self, i2c: &mut I2C, reg: u8) -> Result<u16, E>
    where
        I2C: WriteRead<Error = E>
    {
        match i2c.write_read(self.address, &[reg], &mut self.recv_buffer) {
            Ok(_) => Ok((self.recv_buffer[0] as u16) << 8 | self.recv_buffer[1] as u16),
            Err(e) => Err(e)
        }
    }

    fn write<I2C, E>(&mut self, i2c: &mut I2C, reg: u8, value: u16) -> Result<(), E>
    where
        I2C: WriteRead<Error = E> + Write<Error = E>
    {
        i2c.write(self.address, &[reg])?;
        let msb = ((value & 0xFF00) >> 8) as u8;
        let lsb = ((value & 0x00FF) >> 0) as u8;
        i2c.write(self.address, &[msb, lsb])?;
        Ok(())
    }
}
