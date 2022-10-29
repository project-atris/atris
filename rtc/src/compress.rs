use flate2::Compression;
//use flate2::write::ZlibEncoder;
use flate2::write::GzEncoder;
use flate2::read::GzDecoder;
use std::io::prelude::*;
use std::fs;
use std::env;
use crate::signal;
use std::str;

pub fn main() {
    let mut raw: Vec<u8>;
    let mut com: Vec<u8>;
    let mut braw: Vec<u8>;
    let mut bcom: Vec<u8>;
    let mut lens: [usize; 4];
    let mut cur_len: usize;

    let list = ["basic", "short", "long", "random"];

    //for name in std::array::IntoIter::into_iter(list) {
    for name in list.into_iter() {

        raw = read_file(&["./text_", name, ".txt"].join(""));
        com = compress(raw.clone());
        braw = signal::encode(&String::from_utf8(raw.clone()).unwrap()).as_bytes().to_vec();
        bcom = compress(braw.clone());

        lens = [raw.len(), com.len(), braw.len(), bcom.len()];
        cur_len = *lens.iter().min().unwrap();

        println!("-- READING: text_{}.txt --", name);
        println!("RAW:  {:?}", raw.len());
        println!("COM:  {:?}", com.len());
        //println!("DEC: {:?}", decompress(com).len());
        println!("BRAW: {:?}", braw.len());
        println!("BCOM: {:?}", bcom.len());
        println!("-->{}\n",
            if cur_len == lens[0] {
                "RAW"
            } else if cur_len == lens[1] {
                "COM"
            } else if cur_len == lens[2] {
                "BRAW"
            } else {
                "BCOM"
            }
        );
    }
}

fn read_file(target: &str) -> Vec<u8> {
    fs::read(target)
        .expect("Should have been able to read the file")
}

fn compress(inp: Vec<u8>) -> Vec<u8> {
    let mut encryptor = GzEncoder::new(Vec::new(), Compression::default());
    encryptor.write_all(inp.as_slice());
    encryptor.finish().unwrap()
}

fn decompress(inp: Vec<u8>) -> Vec<u8> {
    let mut decryptor = GzDecoder::new(inp.as_slice());
    let mut ret: Vec<u8> = Vec::new();
    decryptor.read_to_end(&mut ret).unwrap();
    return ret;
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