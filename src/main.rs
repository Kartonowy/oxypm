use std::{
    io::{self, Read},
    process::{Child, Command, Stdio},
    time::SystemTime,
};

mod logic;

fn main() {
    logic::process::init();
}
