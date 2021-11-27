use anyhow::Context;
use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg};
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
                .default_value("/dev/ttyUSB0")
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
        .expect("serial path not found")
        .to_owned();

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
