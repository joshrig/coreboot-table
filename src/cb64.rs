#[derive(Clone, Copy, Debug)]
#[repr(packed)]
pub struct Cb64 {
    lo: u32,
    hi: u32,
}

impl Cb64 {
    fn pack(value: u64) -> Cb64 {
        Cb64 {
            lo: value as u32,
            hi: (value >> 32) as u32,
        }
    }

    fn unpack(&self) -> u64 {
        (self.lo as u64) | ((self.hi as u64) << 32)
    }
}
