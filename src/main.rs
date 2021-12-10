#![warn(clippy::all, clippy::pedantic)]

use clap::{
  app_from_crate,
  App,
  Arg,
  ArgMatches,
  // crate_* macros used by app_from_crate.
  crate_name,
  crate_version,
  crate_authors,
  crate_description,
  SubCommand,
};
use qusb2snes_client;

#[tokio::main]
async fn main() {
  let matches = cli_app().get_matches();

  match matches.subcommand() {
    ("device-list", Some(_)) => list_devices().await,
    ("info", Some(info_matches)) => info(info_matches.value_of("device").unwrap()).await,
    ("get-address", Some(get_address_matches)) => get_address(get_address_matches).await,
    _ => println!("{}", matches.usage()),
  }
}

fn cli_app() -> App<'static, 'static> {
  app_from_crate!()
    .subcommand(
      SubCommand::with_name("device-list")
    )
    .subcommand(
      SubCommand::with_name("info")
        .arg(
          Arg::with_name("device")
            .help("Device string returned from `device-list`")
            .required(true)
        )
    )
    .subcommand(
      SubCommand::with_name("get-address")
        .arg(
          Arg::with_name("device")
            .help("Device string returned from `device-list`")
            .required(true)
        )
        .arg(
          Arg::with_name("address")
            .help("Starting address to read")
            .required(true)
        )
        .arg(
          Arg::with_name("length")
            .help("Number of bytes to read")
            .required(true)
        )
    )
}

async fn get_client() -> qusb2snes_client::Client {
  qusb2snes_client::Client::new().await.unwrap()
}

async fn list_devices() {
  let mut client = get_client().await;
  println!("{:#?}", client.device_list().await);
}

async fn info(device: &str) {
  let mut client = get_client().await;
  client.attach(device).await;
  println!("{:#?}", client.info().await);
}

async fn get_address(matches: &ArgMatches<'_>) {
  let device = matches.value_of("device").unwrap();
  let address_str = matches.value_of("address").unwrap();
  let length_str = matches.value_of("length").unwrap();

  let address = if address_str.starts_with("0x") {
    usize::from_str_radix(address_str.trim_start_matches("0x"), 16).unwrap()
  } else {
    usize::from_str_radix(address_str, 10).unwrap()
  };

  let length = if length_str.starts_with("0x") {
    usize::from_str_radix(length_str.trim_start_matches("0x"), 16).unwrap()
  } else {
    usize::from_str_radix(length_str, 10).unwrap()
  };

  let mut client = get_client().await;
  client.attach(device).await;
  println!("{:#?}", client.get_address(address, length).await);
}
