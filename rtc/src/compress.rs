use flate2::Compression;
//use flate2::write::ZlibEncoder;
use flate2::write::GzEncoder;
use flate2::read::GzDecoder;
use std::io::prelude::*;

pub fn main() {
    
}

fn read_file(target: &str) {
    
}

fn first() {
    let test_str = "we testing this thing";

    // compression
    let mut e = GzEncoder::new(Vec::new(), Compression::default());
    e.write_all(b"foo");
    e.write_all(b"bar");
    e.write_all(test_str.as_bytes());
    let compressed_bytes = e.finish().unwrap();

    // decompression
    let mut d = GzDecoder::new(compressed_bytes.as_slice());
    let mut s = String::new();
    d.read_to_string(&mut s).unwrap();
    println!("{}", s);
}