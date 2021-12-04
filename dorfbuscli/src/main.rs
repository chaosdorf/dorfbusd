use anyhow::{bail, Context};
use clap::Parser;
use cli::{ReadVersion, SetDeviceAddress, SubCommand};
use dorfbusext::DorfbusExt;
use tokio_modbus::{
    client::{rtu, Context as RtuContext},
    prelude::{Slave, SlaveContext},
};
use tokio_serial::SerialStream;

use crate::cli::Opts;

mod cli;

async fn run() -> anyhow::Result<()> {
    let opts: Opts = Opts::parse();

    let builder = tokio_serial::new(opts.serial_device, opts.boud_rate);
    let port = SerialStream::open(&builder).with_context(|| "Could not open the serial device")?;

    let modbus_ctx = rtu::connect(port).await?;

    match opts.subcmd {
        SubCommand::ReadVersion(params) => read_version(modbus_ctx, &params).await?,
        SubCommand::SetDeviceAddress(params) => set_device_address(modbus_ctx, &params).await?,
    }

    Ok(())
}

async fn read_version(mut modbus_ctx: RtuContext, params: &ReadVersion) -> anyhow::Result<()> {
    modbus_ctx.set_slave(Slave(params.modbus_id));

    //let month = modbus_ctx.read_holding_registers(0x04, 1).await?;
    //eprintln!("read month {:?}", month);

    //let year = modbus_ctx.read_holding_registers(0x08, 1).await?;
    //eprintln!("read year {:?}", year);

    //let hour_minute = modbus_ctx.read_holding_registers(0x10, 1).await?;
    //eprintln!("read hour_minute {:?}", hour_minute);

    let hardware_version = modbus_ctx.read_hardware_version().await?;

    println!(
        "Hardware version of device {} is {}",
        params.modbus_id, hardware_version
    );

    Ok(())
}

async fn set_device_address(
    mut modbus_ctx: RtuContext,
    params: &SetDeviceAddress,
) -> anyhow::Result<()> {
    let slave = Slave(params.modbus_id);

    if slave.is_broadcast() {
        bail!("{} is a broadcast address!", params.modbus_id);
    }

    if slave.is_reserved() {
        bail!("{} is a reserved address!", params.modbus_id);
    }

    if !slave.is_single_device() {
        bail!("{} is not a valid device address!", params.modbus_id);
    }

    modbus_ctx.set_slave(Slave::broadcast());
    modbus_ctx.set_device_address(params.modbus_id).await?;

    println!(
        "Device ID was set to {}, test reading version from device....",
        params.modbus_id
    );

    modbus_ctx.set_slave(slave);
    let version = modbus_ctx.read_hardware_version().await?;

    println!(
        "Got a response, the hardware version is {}, address was set successfully!",
        version
    );

    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("Error: {}", err);
        if let Some(cause) = err.source() {
            eprintln!("Caused by: {}", cause);
        }
    }
}
