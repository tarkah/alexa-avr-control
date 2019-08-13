/// This program hosts a custom web service for processing requests for the
/// Alexa AVR Control skill.   
/// 
/// At a high level, the program will launch a thread to manage the telnet
/// connection to the networked AVR device and another thread for the Rouille
/// server, which will have a single route to receive json POST requests from
/// the Alexa skill.   
/// 
/// When requests are received from Alexa, the request will be verified,
/// deserialized and processed into the approriate command needing to be sent
/// to the AVR. The request thread will send a message to the telnet thread
/// with the appropriate command via a crossbeam channel. The telnet thread
/// blocks while waiting for these messages, and once received will write it
/// over the telnet connection, then wait for a response back from the AVR.
/// This response code is then sent back via crossbeam to the request thread
/// for futher processing. If the response from the AVR matches the expected
/// response, verifying the requested change went through, the request thread
/// will respond with a success message back to the user.
use clap::{App, Arg};
use crossbeam_channel::{bounded, Receiver, Sender};
use env_logger::Env;
use failure::Error;
use lazy_static::{initialize, lazy_static};
use log::error;

mod avr;
mod site;
mod skill;
mod speech;
mod telnet;

lazy_static! {
    /// Send messages from skills request thread to telnet thread
    static ref CHANNEL_A: (Sender<String>, Receiver<String>) = { bounded(1) };

    /// Send messages from telnet thread back to skills request thread
    static ref CHANNEL_B: (Sender<String>, Receiver<String>) = { bounded(1) };
}

fn main() {
    if let Err(e) = run() {
        log_error(&e);
        std::process::exit(1);
    }
}

/// Run the program...   
/// 
/// Setup the logger, intialize the crossbeam channels, process command line
/// arguments and kick off the telnet and Rouille threads.
fn run() -> Result<(), Error> {
    env_logger::from_env(Env::default().default_filter_or("alexa_avr_control=info")).init();

    initialize(&CHANNEL_A);
    initialize(&CHANNEL_B);

    let matches = App::new("Alexa AVR Control")
                          .version("0.1.0")
                          .author("Cory F. <cforsstrom18@gmail.com>")
                          .about("A self hosted Alexa skill to control a network-enabled Pioneer AVR through telnet commands.")
                          .arg(Arg::with_name("HOST").required(true)
                                                     .index(1)
                                                     .help("Specify the host / ip of the AVR"))
                          .arg(Arg::with_name("port").short("p")
                                                     .takes_value(true)
                                                     .help("Specify the port to run the skill web service on")
                                                     .default_value("8080")
                                                     .validator(|p| {
                                                            let p = p.parse::<u16>().map_err(|_| "Port provided not valid");
                                                            match p {
                                                                Ok(_) => Ok(()),
                                                                Err(e) => Err(e.to_owned())
                                                            }                                                        
                                                        }))
                          .get_matches();
    let avr_host = matches.value_of("HOST").unwrap();
    let port = matches.value_of("port").unwrap();

    telnet::run(avr_host.to_owned())?;
    site::run(port)?;

    Ok(())
}

/// Log any errors and causes
pub fn log_error(e: &Error) {
    error!("{}", e);
    for cause in e.iter_causes() {
        error!("Caused by: {}", cause);
    }
}
