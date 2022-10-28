mod client;
mod server;
mod compress;
mod signal;

use clap::{arg, command, value_parser, ArgAction, Command};


#[macro_use]
extern crate lazy_static;

fn main() {
    //let matches = command!() // requires `cargo` feature
    //    .arg(arg!(
    //        <location> "Mandatory field"
    //    ))
    //    .arg(arg!(
    //        [name] "Optional name to operate on"
    //    ))
    //    .arg(arg!(
    //        -c --config <FILE> "Sets a custom config file"
    //    ))
    //    .arg(arg!(
    //        -d --debug ... "Turn debugging information on"
    //    ))
    //    .subcommand(
    //        Command::new("test")
    //            .about("does testing things")
    //            .arg(arg!(-l --list "lists test values").action(ArgAction::SetTrue)),
    //    )
    //    .get_matches();
    //}

    // Process the arguments with clap
    let args = command!()
        .arg(arg!(
            --server "Run WebRTC testing as server"
        ))
        .arg(arg!(
            --client "Run WebRTC testing as client"
        ))
        .arg(arg!(
            --compress "Run compression testing"
        ))
        .get_matches(); // run clap, can be omitted to save layout to a variable

    let options = vec!["client", "server", "compress"];

    for option in options.iter() {
        if *args.get_one::<bool>(option).unwrap() {
            match *option {
                "server" => {server::main();},
                "client" => {client::main();},
                "compress" => {compress::main();},
                _ => (),
            }
            break;
        }
    }
}