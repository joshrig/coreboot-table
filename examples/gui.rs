extern crate coreboot_table;
extern crate orbtk;

use std::{env, str};
use std::collections::BTreeMap;

use coreboot_table::{CmosRecord, Table};
use orbtk::{ComboBox, Label, Place, Rect, Text, Window};

use mapper::FileMapper;

mod mapper;

fn main() {
    let mut window = Window::new(
        Rect::new(
            -1, -1,
            1024, 768
        ),
        "Coreboot Table"
    );

    let path = env::args()
        .skip(1)
        .next()
        .unwrap_or("res/example.bin".to_string());

    let mut mapper = FileMapper::new(&path).unwrap();

    let mut entries = Vec::new();
    let mut enums_map = BTreeMap::new();

    coreboot_table::tables(|table| {
        match table {
            Table::Cmos(cmos) => {
                println!("{:?}", cmos);
                for record in cmos.records() {
                    match record {
                        CmosRecord::Entry(entry) => {
                            let name = str::from_utf8(entry.name()).unwrap();
                            println!("    {}: {:?}", name, entry);
                            entries.push(
                                (name.to_string(), entry.config_id)
                            );
                        },
                        CmosRecord::Enum(enum_) => {
                            let text = str::from_utf8(enum_.text()).unwrap();
                            println!("    {}: {:?}", text, enum_);
                            (*enums_map.entry(enum_.config_id).or_insert(Vec::new())).push(
                                (text.to_string(), enum_.value)
                            );
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

    let mut widgets = Vec::new();

    let x = 10;
    let mut y = 16;
    for (name, config_id) in entries.iter() {
        let label = Label::new();
        label.position(x, y)
            .text(name.as_str());
        window.add(&label);

        if let Some(enums) = enums_map.get(config_id) {
            let combo_box = ComboBox::new();
            combo_box.position(x + 200, y);
            for (name, value) in enums {
                combo_box.push(name);
            }
            widgets.push(combo_box);
        }

        y += 32;
    }

    for widget in widgets.iter().rev() {
        window.add(&widget);
    }

    window.exec();
}
