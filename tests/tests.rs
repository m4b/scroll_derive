extern crate scroll;
#[macro_use]
extern crate scroll_derive;

#[derive(Debug, PartialEq, Pread, Pwrite)]
struct Data {
  id: u32,
  timestamp: f64,
}

use scroll::{Pread, Pwrite, Gread, Buffer, LE};

#[test]
fn test_data (){
    let bytes = Buffer::new([0xefu8, 0xbe, 0xad, 0xde, 0, 0, 0, 0, 0, 0, 224, 63]);
    let data: Data = bytes.pread_with(0, LE).unwrap();
    println!("data: {:?}", &data);
    assert_eq!(data.id, 0xdeadbeefu32);
    assert_eq!(data.timestamp, 0.5f64);
    let mut bytes2 = Buffer::with(0, ::std::mem::size_of::<Data>());
    bytes2.pwrite_with(data, 0, LE).unwrap();
    let data: Data = bytes.pread_with(0, LE).unwrap();
    let data2: Data = bytes2.pread_with(0, LE).unwrap();
    assert_eq!(data, data2);
}

#[derive(Debug, PartialEq, Pread, Pwrite)]
struct Data2 {
  name: [u8; 32],
}

#[test]
fn test_array (){
    let bytes = Buffer::with(0, 64);
    let data: Data2 = bytes.pread_with(0, LE).unwrap();
    println!("data: {:?}", &data);
}

#[derive(Debug, PartialEq, Pread, Pwrite, SizeWith)]
struct Data3 {
  name: u32,
}

#[test]
fn test_sizewith (){
    let bytes = Buffer::with(0, 64);
    let data: Data3 = bytes.gread_with(&mut 0, LE).unwrap();
    println!("data: {:?}", &data);
}
