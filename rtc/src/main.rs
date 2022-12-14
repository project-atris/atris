mod compress;
mod initiator;
mod responder;
mod signal;
mod symmetric;
mod symmetric_provided;

mod comms;
//mod client_new;

use clap::{arg, command};

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
            --initiator "Run WebRTC testing as initiator"
        ))
        .arg(arg!(
            --responder "Run WebRTC testing as responder"
        ))
        .arg(arg!(
            --compress "Run compression testing"
        ))
        .arg(arg!(
            --symmetric "Run symmetric encryption testing"
        ))
        .arg(arg!(
            --symmetric_provided "Run symmetric_provided encryption testing"
        ))
        .get_matches(); // run clap, can be omitted to save layout to a variable

    let options = vec![
        "initiator",
        "responder",
        "compress",
        "symmetric",
        "symmetric_provided",
    ];
    let mut found = false;

    for option in options.iter() {
        if *args.get_one::<bool>(option).unwrap() {
            match *option {
                // "initiator" => {initiator::original().await;},
                // "responder" => {responder::original().await;},
                "initiator" => {
                    initiator::main().expect("Initiator err");
                }
                "responder" => {
                    responder::main().expect("Responder err");
                }
                "compress" => {
                    compress::main();
                }
                "symmetric" => {
                    symmetric::main();
                }
                "symmetric_provided" => {
                    symmetric_provided::main();
                }
                _ => {}
            }
            found = true;
            break;
        }
    }

    if !found {
        println!("No valid flag provided");
    }
}
