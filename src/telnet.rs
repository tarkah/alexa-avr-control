use crate::{log_error, CHANNEL_A, CHANNEL_B};
use crossbeam_channel::select;
use failure::{bail, Error, ResultExt};
use log::{debug, error, info};
use std::{
    thread::{self, sleep},
    time::Duration,
};
use telnet::{Telnet, TelnetEvent};

pub fn run(addrs: String) -> Result<(), Error> {
    thread::spawn(move || loop {
        if let Err(e) = connect(&addrs) {
            log_error(&e);
            sleep(Duration::from_secs(10));
        }
    });

    Ok(())
}

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
                        error!("Unknown response from AVR: {:?}", resp);
                    }
                }
            },
        }
    }
}

fn send_response(s: &str) -> Result<(), Error> {
    if CHANNEL_B.0.is_full() {
        bail!("Channel is full...");
    }
    CHANNEL_B.0.send(s.to_owned())?;
    debug!("Sent response code via channel B: {:?}", s);
    Ok(())
}
