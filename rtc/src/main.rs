mod client;
mod server;
mod signal;

use clap::{arg, command, value_parser, ArgAction, Command};


#[macro_use]
extern crate lazy_static;

fn main() {
    //let matches = command!() // requires `cargo` feature
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
        .about("testing this") // about this project
        .arg(arg!( // first arg, client
            -c --client "Run as client instead of server"
        ))
        .get_matches(); // run clap, can be omitted to save layout to a variable

    //println!("{:?}", args.get_one::<bool>("client"));

    if *args.get_one::<bool>("client").unwrap() {
        client::main();
    } else {
        server::main();
    }

}