use clap::Parser;

#[derive(Parser)]
#[clap(version = "1.0", author = "Raphael Peters <rappet@rappet.de>")]
pub struct Opts {
    #[clap(
        short = 'd',
        long,
        default_value = "/dev/ttyUSB0",
        about = "Path to the serial device"
    )]
    pub serial_device: String,
    #[clap(
        short = 'b',
        long,
        default_value = "9600",
        about = "Boud rate of the serial device"
    )]
    pub boud_rate: u32,
    #[clap(subcommand)]
    pub subcmd: SubCommand,
}

#[derive(Parser)]
pub enum SubCommand {
    ReadVersion(ReadVersion),
}

#[derive(Parser)]
#[clap(about = "Read version information of a single modbus device")]
pub struct ReadVersion {
    #[clap(about = "ID of the device on the bus")]
    pub modbus_id: u8,
}
