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
pub fn run(addrs: String) -> Result<(), Error> {
    thread::spawn(move || loop {
        if let Err(e) = connect(&addrs) {
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
fn connect(addrs: &str) -> Result<(), Error> {
    let mut conn =
        Telnet::connect((addrs, 5555), 256).context("Could not connect to AVR via telnet")?;

    loop {
        select! {
            recv(CHANNEL_A.1) -> code => {
                let code = code?;
                debug!("Code received via channel A: {:?}", code);

                conn.write(code.as_str().as_bytes()).context("Could not write to AVR via telnet")?;

                let resp = conn.read_timeout(Duration::from_millis(500)).context("Telnet response error")?;

                // Data should always be received back from AVR. Assume connection
                // is broken otherwise, and bail to attempt reconnect.
                match resp {
                    TelnetEvent::Data(d) => {
                        let s = std::str::from_utf8(&d).context(format!("Could not convert response to UTF-8: {:?}", d))?;
                        info!("Code sent to AVR: {:?}. Received back: {:?}", code, s);
                        if let Err(e) = send_response(s) {
                            log_error(&e);
                        }
                    },
                    TelnetEvent::TimedOut => {
                        bail!("Timeout... Resetting connection to AVR");
                    },
                    _ => {
                        bail!("Unknown response from AVR, resetting connection: {:?}", resp);
                    }
                }
            },
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
