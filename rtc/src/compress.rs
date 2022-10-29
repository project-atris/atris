use flate2::Compression;
//use flate2::write::ZlibEncoder;
use flate2::write::{GzEncoder, ZlibEncoder, DeflateEncoder};
use flate2::read::{GzDecoder, ZlibDecoder, DeflateDecoder};
use std::io::prelude::*;
use std::fs;
use crate::signal;
use std::str;

pub fn main() {
    let list = [
        "basic",
        "short_1",
        "short_19",
        "short_20",
        "short_25",
        "short_30",
        "medium",
        "medium1",
        "long",
        "random",
    ];

    //for name in std::array::IntoIter::into_iter(list) {
    for name in list.into_iter() {

        // raw
        let raw = read_file(&["./text_", name, ".txt"].join(""));
        let braw = signal::encode(&String::from_utf8(raw.clone()).unwrap()).as_bytes().to_vec();

        // gzip
        let com_g = compress_g(raw.clone());
        let bcom_g = compress_g(braw.clone());

        // zlib
        let com_z = compress_z(raw.clone());
        let bcom_z = compress_z(braw.clone());

        // deflate
        let com_d = compress_d(raw.clone());
        let bcom_d = compress_d(braw.clone());

        let lens = [raw.len(), braw.len(), com_g.len(), bcom_g.len(), com_z.len(), bcom_z.len(), com_d.len(), bcom_d.len()];
        let cur_len = *lens.iter().min().unwrap();

        println!("-- READING: text_{}.txt --", name);
        println!("RAW:      {:?}", lens[0]);
        println!("B64:      {:?}", lens[1]);
        println!("Gzip_RAW: {:?}", lens[2]);
        println!("Gzip_B64: {:?}", lens[3]);
        println!("Zlib_RAW: {:?}", lens[4]);
        println!("Zlib_B64: {:?}", lens[5]);
        println!("DEFL_RAW: {:?}", lens[6]);
        println!("DEFL_B64: {:?}", lens[7]);
        println!("-->{}\n",
            if cur_len == lens[0] {
                "RAW"
            } else if cur_len == lens[1] {
                "B64"
            } else if cur_len == lens[2] {
                "Gzip_RAW"
            } else if cur_len == lens[3] {
                "Gzip_B64"
            } else if cur_len == lens[4] {
                "Zlib_RAW"
            } else if cur_len == lens[5] {
                "Zlib_B64"
            } else if cur_len == lens[6] {
                "DEFL_RAW"
            } else {
                "DEFL_B64"
            }
            
        );
    }
}

fn read_file(target: &str) -> Vec<u8> {
    fs::read(target)
        .expect("Should have been able to read the file")
}

fn compress_g(inp: Vec<u8>) -> Vec<u8> {
    let mut encryptor = GzEncoder::new(Vec::new(), Compression::default());
    encryptor.write_all(inp.as_slice());
    encryptor.finish().unwrap()
}

fn compress_z(inp: Vec<u8>) -> Vec<u8> {
    let mut encryptor = ZlibEncoder::new(Vec::new(), Compression::default());
    encryptor.write_all(inp.as_slice());
    encryptor.finish().unwrap()
}

fn compress_d(inp: Vec<u8>) -> Vec<u8> {
    let mut encryptor = DeflateEncoder::new(Vec::new(), Compression::default());
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