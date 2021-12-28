use std::fs::read_dir;

use anyhow::Context;
use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg};
use tracing::warn;
#[derive(Clone)]
pub struct Params {
    pub port: u16,
    pub serial_path: String,
    pub serial_boud: u32,
    pub config_path: String,
}

pub fn app() -> anyhow::Result<Params> {
    let matches = App::new(crate_name!())
        .author(crate_authors!())
        .about(crate_description!())
        .version(crate_version!())
        .arg(
            Arg::from_usage(
                "-p, --port=[HTTP_PORT] 'Sets the TCP port to listen for HTTP connections'",
            )
            .default_value("8080")
            .env("HTTP_PORT"),
        )
        .arg(
            Arg::from_usage("-s, --serial-path=[SERIAL_PATH] 'Path to the serial device'")
                .env("SERIAL_PATH"),
        )
        .arg(
            Arg::from_usage("-b, --serial-boud=[SERIAL_PATH] 'Boudrate for the serial device'")
                .default_value("9600")
                .env("SERIAL_BOUD"),
        )
        .arg(
            Arg::from_usage("-c, --config=[CONFIG] 'Path to the configuration file'")
                .default_value("config.toml")
                .env("CONFIG"),
        )
        .get_matches();

    let port = matches
        .value_of("port")
        .expect("port number not found")
        .parse()?;

    let serial_path = matches
        .value_of("serial-path")
        .map(|s| s.to_owned())
        .or_else(guess_serial_device)
        .expect("serial path not found");

    let serial_boud = matches
        .value_of("serial-boud")
        .expect("serial boudrate not found")
        .parse()
        .with_context(|| "The specified boud rate is not a valid integer")?;

    let config_path = matches
        .value_of("config")
        .expect("config path not found")
        .to_owned();

    Ok(Params {
        port,
        serial_path,
        serial_boud,
        config_path,
    })
}

fn guess_serial_device() -> Option<String> {
    warn!("Detecting serial device. The serial device should be configured explicitly in a production environment.");
    read_dir("/dev")
        .unwrap()
        .into_iter()
        .filter_map(|dir_entry| dir_entry.ok())
        .filter(|dir_entry| {
            let name = dir_entry.file_name();
            let name = name.to_string_lossy();
            // USB devices
            name.starts_with("ttyUSB")
                || name.starts_with("ttyACM")
                // ARM UART (RaspberryPi, etc.)
                || name.starts_with("ttyAMA")
                // MacOS USB
                || name.starts_with("tty.usbserial-")
        })
        .map(|dir_entry| {
            dir_entry
                .path()
                .as_os_str()
                .to_str()
                .expect("serial path is not a valid String")
                .to_string()
        })
        .next()
}
