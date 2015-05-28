extern crate byteorder;
use byteorder::{ ReadBytesExt, BigEndian };

use std::error::Error;
use std::fs::File;
use std::io::{Cursor, Read};
use std::mem;

#[derive(Debug)]
struct Header {
    version: u8,
    num_utc: u32,
    num_standard: u32,
    num_leap_seconds: u32,
    num_transition_times: u32,
    num_time_types: u32,
    num_chars: u32,
}

#[derive(Debug)]
struct Transition {
    timestamp: u32,
    index: usize,
}

#[derive(Debug)]
enum TransitionType {
    Standard,
    Wall,
    UTC,
}

#[derive(Debug)]
struct TimeInfo {
    offset: u32,
    is_dst: bool,
    ttype:  u8,
}

#[derive(Debug)]
struct LeapSecondInfo {
    timestamp: u32,
    offset: u32,
}

#[derive(Debug)]
struct TimezoneData {
    pub transitions:   Vec<Transition>,
    pub types:         Vec<TimeInfo>,
    pub leap_seconds:  Vec<LeapSecondInfo>,
}

/// Maximum number of transition times
static MAX_TIMES: usize = 1200;

/// Maximum number of TimeInfo settings
static MAX_TYPES: usize = 256;

/// Maximum number of bytes in timezone abbreviations
static MAX_CHARS: usize = 50;

/// Maximum number of leap second specifications
static MAX_LEAPS: usize = 50;

struct Parser {
    cursor: Cursor<Vec<u8>>,
}

impl Parser {
    fn new(buf: Vec<u8>) -> Parser {
        Parser {
            cursor: Cursor::new(buf),
        }
    }

    fn read_magic_number(&mut self) -> Result<bool, Box<Error>> {
        let mut magic = [0u8; 4];
        try!(self.cursor.read(&mut magic));
        Ok(magic == *b"TZif")
    }

    fn skip_initial_zeroes(&mut self, count: usize) -> Result<[u8; 15], Box<Error>> {
        let mut magic = [0u8; 15];
        try!(self.cursor.read(&mut magic));
        Ok(magic)
    }

    fn read_header(&mut self) -> Result<Header, Box<Error>> {
        Ok(Header {
            version: try!(self.cursor.read_u8()),
            num_utc: try!(self.cursor.read_u32::<BigEndian>()),
            num_standard: try!(self.cursor.read_u32::<BigEndian>()),
            num_leap_seconds: try!(self.cursor.read_u32::<BigEndian>()),
            num_transition_times: try!(self.cursor.read_u32::<BigEndian>()),
            num_time_types: try!(self.cursor.read_u32::<BigEndian>()),
            num_chars: try!(self.cursor.read_u32::<BigEndian>()),
        })
    }

    fn read_transition_times(&mut self, count: usize) -> Result<Vec<u32>, Box<Error>> {
        let mut buf = Vec::with_capacity(count);
        for _ in 0 .. count {
            buf.push(try!(self.cursor.read_u32::<BigEndian>()));
        }
        Ok(buf)
     }

    fn read_transition_indices(&mut self, count: usize) -> Result<Vec<u8>, Box<Error>> {
        let mut buf = Vec::with_capacity(count);
        for _ in 0 .. count {
            buf.push(try!(self.cursor.read_u8()));
        }
        Ok(buf)
    }

    fn read_info_structures(&mut self, count: usize) -> Result<Vec<TimeInfo>, Box<Error>> {
        let mut buf = Vec::with_capacity(count);
        for _ in 0 .. count {
            buf.push(TimeInfo {
                offset:  try!(self.cursor.read_u32::<BigEndian>()),
                is_dst:  try!(self.cursor.read_u8()) != 0,
                ttype:   try!(self.cursor.read_u8()),
            });
        }
        Ok(buf)
    }

    fn read_leap_second_info(&mut self, count: usize) -> Result<Vec<LeapSecondInfo>, Box<Error>> {
        let mut buf = Vec::with_capacity(count);
        for _ in 0 .. count {
            buf.push(LeapSecondInfo {
                timestamp:  try!(self.cursor.read_u32::<BigEndian>()),
                offset:     try!(self.cursor.read_u32::<BigEndian>()),
            });
        }
        Ok(buf)
    }

}

fn main() {
    let mut file = File::open("/etc/localtime").unwrap();
    let mut buf = Vec::new();
    file.read_to_end(&mut buf);

    let mut parser = Parser::new(buf);
    let magic = parser.read_magic_number();
    println!("{:?}", magic);

    let zeroes = parser.skip_initial_zeroes(15);
    println!("{:?}", zeroes);

    let header = parser.read_header().unwrap();
    println!("{:?}", header);

    let ts = parser.read_transition_times(header.num_transition_times as usize);
    println!("{:?}", ts);

    let ti = parser.read_transition_indices(header.num_transition_times as usize);
    println!("{:?}", ti);

    let tt = parser.read_info_structures(header.num_time_types as usize);
    println!("{:?}", tt);

    let tt = parser.read_leap_second_info(header.num_leap_seconds as usize);
    println!("{:?}", tt);


}
