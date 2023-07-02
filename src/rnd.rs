use js_sys::Math::random;
use rand::{Error, RngCore};

pub struct Rng {}

impl Rng {
    fn next_u8(&mut self) -> u8 {
        f64::floor(random() * (u8::MAX as f64 + 1.)) as u8
    }
}

impl RngCore for Rng {
    fn next_u32(&mut self) -> u32 {
        f64::floor(random() * (u32::MAX as f64 + 1.)) as u32
    }

    fn next_u64(&mut self) -> u64 {
        f64::floor(random() * (u64::MAX as f64 + 1.)) as u64
    }
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        for i in dest.iter_mut() {
            *i = self.next_u8();
        }
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Error> {
        for i in dest.iter_mut() {
            *i = self.next_u8();
        }
        Ok(())
    }
}
