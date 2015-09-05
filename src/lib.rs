#![feature(io,core)]

pub struct SerializerState {
    blobs: Vec<Vec<u8>>,
    blobs_total_len: u64
}

impl SerializerState {
    pub fn write_blobs<T: std::io::Write>(&self, writer: &mut T) -> std::io::Result<()> {
        for blob in self.blobs.iter() {
            try!(writer.write_all(blob));
        }
        Ok(())
    }

    pub fn add_blob<T: std::io::Write>(&mut self, blob: &[u8], writer: &mut T) -> std::io::Result<()> {
        let current_len: u64 = blob.len() as u64;
        if current_len < 16 {
            let mut buf = [0u8; 16];
            buf[0] = (current_len as u8) | 0b1000_0000u8;
            let mut i = 1;
            for &b in blob {
                buf[i] = b;
                i += 1;
            }
            try!(writer.write_all(&buf));
        } else {
            let current_pos: u64 = self.blobs_total_len;
            self.blobs_total_len += current_len;
            self.blobs.push(blob.to_vec());
                try!(current_pos.write_serialized(writer, self));
                try!(current_len.write_serialized(writer, self));
        }
        Ok(())
    }
}

trait Serializable {
    fn write_serialized<T: std::io::Write>(&self, writer: &mut T, state: &mut SerializerState) -> std::io::Result<()>;
}

impl<U: std::num::Int> Serializable for U {
    fn write_serialized<T: std::io::Write>(&self, writer: &mut T, _: &mut SerializerState) -> std::io::Result<()> {
        unsafe {
            try!(writer.write_all(std::slice::from_raw_parts(std::mem::transmute(self), std::mem::size_of_val(self))));
        }
        Ok(())
    }
}

#[test]
fn it_works() {
}
