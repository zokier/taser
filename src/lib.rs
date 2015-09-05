pub struct SerializerState<'a> {
    writer: &'a mut std::io::Write,
    blobs: Vec<Vec<u8>>,
    blobs_total_len: u64
}

impl<'a> SerializerState<'a> {
    pub fn flush_blobs(&mut self) -> std::io::Result<()> {
        try!(self.blobs_total_len.clone().write_serialized(self));
        for blob in self.blobs.iter() {
            try!(self.writer.write_all(blob));
        }
        Ok(())
    }

    pub fn add_var_blob(&mut self, blob: &[u8]) -> std::io::Result<()> {
        let current_len: u64 = blob.len() as u64;
        if current_len < 16 {
            let mut buf = [0u8; 16];
            buf[0] = (current_len as u8) | 0b1000_0000u8;
            let mut i = 1;
            for &b in blob {
                buf[i] = b;
                i += 1;
            }
            try!(self.add_fixed_blob(&buf));
        } else {
            let current_pos: u64 = self.blobs_total_len;
            self.blobs_total_len += current_len;
            self.blobs.push(blob.to_vec());
            try!(current_pos.write_serialized(self));
            try!(current_len.write_serialized(self));
        }
        Ok(())
    }
    
    pub fn add_fixed_blob(&mut self, blob: &[u8]) -> std::io::Result<()> {
        self.writer.write_all(blob)
    }
}

trait Serializable {
    fn write_serialized(&self, state: &mut SerializerState) -> std::io::Result<()>;
}

macro_rules! serializable_impl {
    ($($t:ty)*) => ($(
impl Serializable for $t {
    fn write_serialized(&self, state: &mut SerializerState) -> std::io::Result<()> {
        unsafe {
            state.add_fixed_blob(std::slice::from_raw_parts(std::mem::transmute(self), std::mem::size_of_val(self)))
        }
    }
}
)*)
}

serializable_impl! { usize u8 u16 u32 u64 isize i8 i16 i32 i64 f32 f64 }

impl Serializable for bool {
    fn write_serialized(&self, state: &mut SerializerState) -> std::io::Result<()> {
        match *self {
            true => state.add_fixed_blob(&[1u8]),
            false => state.add_fixed_blob(&[0u8])
        }
    }
}

pub struct VarStr<'a>(&'a str);
pub struct FixedStr<'a>(&'a str);

impl<'a> Serializable for VarStr<'a> {
    fn write_serialized(&self, state: &mut SerializerState) -> std::io::Result<()> {
        let VarStr(s) = *self;
        state.add_var_blob(s.as_bytes())
    }
}

impl<'a> Serializable for FixedStr<'a> {
    fn write_serialized(&self, state: &mut SerializerState) -> std::io::Result<()> {
        let FixedStr(s) = *self;
        state.add_fixed_blob(s.as_bytes())
    }
}

