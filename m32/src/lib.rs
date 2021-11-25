#![feature(iter_order_by)]
#![feature(is_sorted)]
#![feature(iter_zip)]
#![feature(trace_macros)]

#[macro_use]
extern crate lazy_static;

mod attribs;
mod interval;
mod simple_note;
mod pitch;
mod duration;
mod color;
mod lyric;
mod tuplet;
mod config;
mod gnote;
mod measure;
mod part;
mod score;
mod xml_import;
mod xml_export;
