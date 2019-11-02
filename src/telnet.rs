/// This module is responsible over maintaining a telnet connection
/// to the AVR device, and receiving commands that need to be sent over
/// that telnet connection.   
///
/// Crossbeam channels are used for communicating between the skill's request
/// and this thread.   
///
/// The AVR device will always respond to the telnet command with a response
/// code, which needs to be sent back via crossbeam channel to finish
/// procsesing on the skill side.
use crate::{log_error, CHANNEL_A, CHANNEL_B};
use crossbeam_channel::select;
use failure::{bail, Error, ResultExt};
use log::{debug, info};
use std::{
    thread::{self, sleep},
    time::Duration,
};
use telnet::{Telnet, TelnetEvent};

/// Spawn a new thread to run telnet communication between AVR.   
///
/// Attempt to reconnect if error occurs, logging error.
pub fn run(addrs: String, port: u16) -> Result<(), Error> {
    thread::spawn(move || loop {
        if let Err(e) = connect(&addrs, port) {
            log_error(&e);
            sleep(Duration::from_secs(10));
        }
    });

    Ok(())
}

/// Connects to AVR and waits for commands from skill.   
///
/// Upon receiving command, it will send to AVR over telnet connection.
/// It will then try to get response from AVR, which should be some data code,
/// and send that back to the skill for further processing.   
///
/// If this response doesn't occur (timeout), or if the response type isn't valid
/// (could happen from connection error), assume connection is broken and bail to
/// reconnect.
///
/// Also clears the telnet channel every 1 second, as AVR will send a heartbeat
/// signal every 30 seconds: "R\r\n". We don't want this present in the response
/// from AVR after we send our command.
fn connect(addrs: &str, port: u16) -> Result<(), Error> {
    let mut conn =
        Telnet::connect((addrs, port), 256).context("Could not connect to AVR via telnet")?;
    info!("Successful connection to AVR via telnet");

    loop {
        select! {
            recv(CHANNEL_A.1) -> code => {
                let code = code?;
                debug!("Code received via channel A: {:?}", code);

                conn.write(code.as_bytes()).context("Could not write to AVR via telnet")?;

                let mut resp_buffer = String::new();

                // AVR responds twice with Power On request, the first being useless. We need to capture it to keep 2nd
                // response from being missed and populating later requests.
                thread::sleep(Duration::from_millis(500));
                let resp = conn.read_timeout(Duration::from_millis(500)).context("Error reading from telnet connection")?;
                match resp {
                    TelnetEvent::Data(d) => {
                        let s = std::str::from_utf8(&d).context(format!("Could not convert response to UTF-8: {:?}", d))?;
                        resp_buffer.push_str(s);
                    },
                    TelnetEvent::TimedOut => {},
                    _ => {
                        bail!("Unknown response from AVR, resetting connection: {:?}", resp);
                    }
                }

                info!("Code sent to AVR: {:?}. Received back: {:?}", code, resp_buffer);
                if let Err(e) = send_response(&resp_buffer) {
                    log_error(&e);
                }
            },
            // Clear telnet connection of any "R\r\n" heartbeat messages
            default(Duration::from_millis(1000)) => {
                let resp = conn.read_nonblocking().context("Error reading from telnet connection")?;
                if let TelnetEvent::Data(d) = resp {
                    let s = std::str::from_utf8(&d).context(format!("Could not convert response to UTF-8: {:?}", d))?;
                    debug!("Cleared from connection: {:?}", s);
                }
            }
        }
    }
}

/// Send response code back to skill for further processing.
fn send_response(s: &str) -> Result<(), Error> {
    // Clear channel B if full, it shouldn't be
    if CHANNEL_B.0.is_full() {
        select! {
            recv(CHANNEL_B.1) -> _ => {}
            default() => {}
        }
        debug!("Had to clear channel B");
    }
    CHANNEL_B.0.send(s.to_owned())?;
    debug!("Sent response code via channel B: {:?}", s);
    Ok(())
}
