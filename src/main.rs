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
mod tel;

lazy_static! {
    // Send messages from AVR to Tel
    static ref CHANNEL_A: (Sender<String>, Receiver<String>) = { bounded(1) };

    // Send messages from Tel to AVR
    static ref CHANNEL_B: (Sender<String>, Receiver<String>) = { bounded(1) };
}

fn main() {
    if let Err(e) = run() {
        log_error(&e);
        std::process::exit(1);
    }
}

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

    tel::run(avr_host.to_owned())?;
    site::run(port)?;
    Ok(())
}

pub fn log_error(e: &Error) {
    error!("{}", e);
    for cause in e.iter_causes() {
        error!("Caused by: {}", cause);
    }
}
