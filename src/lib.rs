#![no_std]
#![feature(alloc)]

#[macro_use]
extern crate alloc;
extern crate plain;

use alloc::{String, Vec};
use plain::Plain;

#[repr(packed)]
#[derive(Clone, Default, Debug)]
pub struct Smbios {
    pub anchor: [u8; 4],
    pub checksum: u8,
    pub length: u8,
    pub major_version: u8,
    pub minor_version: u8,
    pub max_structure_size: u16,
    pub revision: u8,
    pub formatted: [u8; 5],
    pub inter_anchor: [u8; 5],
    pub inter_checksum: u8,
    pub table_length: u16,
    pub table_address: u32,
    pub structure_count: u16,
    pub bcd_revision: u8
}

unsafe impl Plain for Smbios {}

impl Smbios {
    pub fn is_valid(&self) -> bool {
        let mut sum: u8 = self.anchor.iter().fold(0,|a, &b| a + b);
        sum += self.checksum;
        sum += self.length;
        sum += self.major_version;
        sum += self.minor_version;
        sum = sum + (self.max_structure_size as u8);
        sum += self.revision;
        sum += self.formatted.iter().fold(0,|a, &b| a + b);
        sum == 0
    }
}

#[repr(packed)]
#[derive(Clone, Default, Debug)]
pub struct Smbios3 {
    pub anchor: [u8; 5],
    pub checksum: u8,
    pub length: u8,
    pub major_version: u8,
    pub minor_version: u8,
    pub docrev: u8,
    pub revision: u8,
    _reserved: u8,
    pub table_length: u32,
    pub table_address: u64,
}

unsafe impl Plain for Smbios3 {}

impl Smbios3 {
    pub fn is_valid(&self) -> bool {
        //TODO: Checksum
        self.anchor == *b"_SM3_"
    }
}

#[repr(packed)]
#[derive(Clone, Default, Debug)]
pub struct Header {
    pub kind: u8,
    pub len: u8,
    pub handle: u16
}

unsafe impl Plain for Header {}

#[derive(Clone)]
pub struct Table {
    pub header: Header,
    pub data: Vec<u8>,
    pub strings: Vec<String>
}

impl Table {
    pub fn get_str(&self, index: u8) -> Option<&String> {
        if index > 0 {
            self.strings.get((index - 1) as usize)
        } else {
            None
        }
    }
}

#[repr(packed)]
#[derive(Default, Debug)]
pub struct BiosInfo {
    pub vendor: u8,
    pub version: u8,
    pub address: u16,
    pub date: u8,
    pub size: u8,
    pub characteristics: u64,
}

unsafe impl Plain for BiosInfo {}

#[repr(packed)]
#[derive(Default, Debug)]
pub struct SystemInfo {
    pub manufacturer: u8,
    pub name: u8,
    pub version: u8,
    pub serial: u8,
}

unsafe impl Plain for SystemInfo {}

pub fn tables(data: &[u8]) -> Vec<Table> {
    let mut tables = Vec::new();

    let mut i = 0;
    while i < data.len() {
        // Read header
        let mut header = Header::default();
        {
            let bytes = header.as_mut_bytes();

            let mut j = 0;
            while i < data.len() && j < bytes.len() {
                bytes[j] = data[i];
                i += 1;
                j += 1;
            }
        }

        if header.kind == 127 {
            //println!("End header");
            break;
        }

        //println!("{:?}", header);

        // Read data
        let mut table = vec![0; header.len as usize - header.as_bytes().len()];

        {
            let mut j = 0;
            while i < data.len() && j < table.len() {
                table[j] = data[i];
                i += 1;
                j += 1;
            }
        }

        // Read strings
        let mut strings = Vec::new();
        while i < data.len() {
            let mut string = String::new();
            while i < data.len() {
                let b = data[i];
                i += 1;

                if b == 0 {
                    break;
                } else {
                    string.push(b as char);
                }
            }

            if string.is_empty() && ! strings.is_empty() {
                break;
            } else {
                //println!("{}: {}", strings.len(), string);
                strings.push(string);
            }
        }

        tables.push(Table {
            header: header,
            data: table,
            strings: strings
        });
    }

    tables
}
