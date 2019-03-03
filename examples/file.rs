extern crate coreboot_table;

use std::{env, str};

use coreboot_table::{CmosRecord, Table};

use mapper::FileMapper;

mod mapper;

fn main() {
    let path = env::args()
        .skip(1)
        .next()
        .unwrap_or("res/example.bin".to_string());

    let mut mapper = FileMapper::new(&path).unwrap();

    coreboot_table::tables(|table| {
        match table {
            Table::Cmos(cmos) => {
                println!("{:?}", cmos);
                for record in cmos.records() {
                    match record {
                        CmosRecord::Entry(entry) => {
                            println!(
                                "    {:?}: {:?}",
                                str::from_utf8(entry.name()),
                                entry
                            )
                        },
                        CmosRecord::Enum(enum_) => {
                            println!(
                                "    {:?}: {:?}",
                                str::from_utf8(enum_.text()),
                                enum_
                            )
                        },
                        CmosRecord::Other(other) => {
                            println!("    {:?}", other);
                        },
                    }
                }
            },
            Table::Framebuffer(framebuffer) => {
                println!("{:?}", framebuffer);
            },
            Table::Memory(memory) => {
                println!("{:?}", memory);
                for range in memory.ranges() {
                    println!("    {:?}", range);
                }
            },
            Table::Other(other) => {
                println!("{:?}", other);
            },
        }
        Ok(())
    }, &mut mapper).unwrap();
}
