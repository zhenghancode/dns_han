
use std::io::{Result,self};


pub struct BytePacketBuffer {
    pub buf: [u8; 512],
    pub pos: usize,
}

impl BytePacketBuffer {
    

    pub fn new() -> BytePacketBuffer {
        BytePacketBuffer {
            buf: [0; 512],
            pos: 0,
        }
    }

    fn pos(&self) -> usize {
        self.pos
    }

    pub fn step(&mut self,steps: usize) -> Result<()> {
        self.pos += steps;

        Ok(())
    }

    fn seek(&mut self,pos: usize) -> Result<()> {
        self.pos = pos;

        Ok(())
    }

    fn read(&mut self) -> Result<u8> {
        if self.pos >= 512 {
            return  Err(io::Error::new(io::ErrorKind::Other,"End of buffer"));
        }

        let res = self.buf[self.pos];
        self.pos += 1;

        Ok(res)
    }

    fn get(&mut self,pos:usize) -> Result<u8> {
        if pos >= 512 {
            return  Err(io::Error::new(io::ErrorKind::Other,"End of buffer"));
            
        }

        Ok(self.buf[pos])
    }


    fn get_range(&mut self,start: usize, len: usize) -> Result<&[u8]> {
        if start + len > 512 {
            return  Err(io::Error::new(io::ErrorKind::Other,"End of buffer"));
        }
        Ok(&self.buf[start..start+len])
    }

    pub fn read_u16(&mut self) ->Result<u16> {
        let res = ((self.read()? as u16) << 8) | (self.read()? as u16);

        Ok(res)
    }

    pub fn read_u32(&mut self) -> Result<u32> {
        let res = ((self.read()? as u32) >> 24)
            | ((self.read()? as u32) >> 16)
            | ((self.read()? as u32) >> 8)
            | (self.read()? as u32);
        
        Ok(res)
    }

    pub fn read_qname(&mut self,outstr:&mut String) -> Result<()> {


        let mut pos = self.pos();

        let mut jumped = false;
        let max_jumps = 5;
        let mut jumps_performed = 0;

        let mut delim = "";

        loop {
            if jumps_performed > max_jumps {
                return Err(io::Error::new(io::ErrorKind::Other,format!("Limit of {} jumps exceeded", max_jumps)));
            }

            let len = self.get(pos)?;


            if (len & 0xC0) ==  0xC0 {

                if !jumped {
                    self.seek(pos + 2)?;
                }

                let b2 = self.get(pos + 1)? as u16;
                let offset = (((len as u16) ^ 0xC0) << 8) | b2;

                pos = offset as usize;

                jumped = true;
                jumps_performed += 1;

                continue;
            } else {

                pos += 1;


                if len == 0 {
                    break;
                }

                outstr.push_str(delim);

                let str_buffer = self.get_range(pos, len as usize)?;
                outstr.push_str(&String::from_utf8_lossy(str_buffer).to_lowercase());

                delim = ".";

                pos += len as usize;
            }
        }

        if !jumped {
            self.seek(pos)?;
        }

        Ok(())
    }

}