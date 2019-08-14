/// This module contains all the logic for converting the requested skill
/// Intent into the proper AVR command code that can be sent over telnet
/// to control the AVR. It will also validate that the response from the
/// AVR via telnet matches the expected response, confirming that the command
/// was executed successfuly.
use crate::{CHANNEL_A, CHANNEL_B};
use crossbeam_channel::select;
use failure::{bail, Error};
use log::{debug, info};
use std::time::Duration;

/// Entry point to use from skill module to request the appropriate command
pub fn process(cmd: AvrCommand) -> Result<(), Error> {
    send_and_validate(cmd)?;
    Ok(())
}

/// Commands that can be sent to AVR
pub enum AvrCommand {
    SetVolume(u8),
    Mute,
    Unmute,
    PowerOn,
    PowerOff,
    ChangeInput(u8),
}

impl AvrCommand {
    /// Convert enum to the appropriate telnet command supported
    /// by the AVR
    fn code(&self) -> String {
        match &self {
            AvrCommand::SetVolume(n) => get_volume_code(*n),
            AvrCommand::ChangeInput(n) => get_input_code(*n),
            AvrCommand::PowerOn => "PO\r".to_owned(),
            AvrCommand::PowerOff => "PF\r".to_owned(),
            AvrCommand::Mute => "MO\r".to_owned(),
            AvrCommand::Unmute => "MF\r".to_owned(),
        }
    }
}

/// Convert volume of 1 - 10 to appropriate AVR volume code.   
///
/// 161 is equal to 0.0dB and I don't want to set any higher via this skill,
/// so I've set this as the ceiling.
///
/// Must be padded to three digits: "{:0>3}"
fn get_volume_code(n: u8) -> String {
    let ceiling = 161.0;
    let weight = f32::from(n) / 10.0;
    let volume = (weight * ceiling).floor() as u8;
    let mut volume = format!("{:0>3}", volume);
    volume.push_str("VL\r");
    volume
}

/// Convert input to AVR input code.
fn get_input_code(n: u8) -> String {
    let code = match n {
        1 => "25",  // BD
        2 => "04",  // DVD
        3 => "05",  // TV/SAT
        4 => "15",  // DVR/BDR
        5 => "10",  // VIDEO 1(VIDEO)
        6 => "14",  // VIDEO 2
        7 => "19",  // HDMI 1
        8 => "20",  // HDMI 2
        9 => "21",  // HDMI 3
        10 => "22", // HDMI 4
        11 => "23", // HDMI 5
        12 => "24", // HDMI 6
        13 => "26", // HOME MEDIA GALLERY(Internet Radio)
        14 => "17", // iPod/USB
        15 => "01", // CD
        16 => "03", // CD-R/TAPE
        17 => "02", // TUNER
        18 => "00", // PHONO
        19 => "12", // MULTI CH IN
        20 => "33", // ADAPTER PORT
        21 => "27", // SIRIUS
        22 => "31", // HDMI (cyclic)
        _ => "",    // Should never be reached
    };
    let mut code = code.to_owned();
    code.push_str("FN\r");
    code
}

/// Convert AvrCommand to the appropriate AVR command code and then send to the
/// telnet thread, so it can be sent along to the AVR.
///
/// Telnet thread will send response back from AVR, which then can be validated
/// to give us confidence that the requested command was successful.
fn send_and_validate(cmd: AvrCommand) -> Result<(), Error> {
    let code = cmd.code();
    info!("Translated to code: {:?}", code);

    // Clear channel A if full, it shouldn't be
    if CHANNEL_A.0.is_full() {
        select! {
            recv(CHANNEL_A.1) -> _ => {}
            default => {}
        }
        debug!("Had to clear channel A");
    }
    CHANNEL_A.0.send(code.clone())?;
    debug!("Sent code via channel A: {:?}", code);

    let response = get_response()?;
    validate_response(cmd, &code, &response)
}

/// Get response code back from AVR. If this response takes longer than 1
/// second, assume error.
fn get_response() -> Result<String, Error> {
    select! {
        recv(CHANNEL_B.1) -> msg => {
            let msg = msg?;
            debug!("Response code received via channel B: {:?}", msg);
            Ok(msg)
        },
        default(Duration::from_millis(1_000)) => {
            bail!("Timeout. Didn't get response from AVR.");
        }
    }
}

/// AVR sends back code validating the request. Confirm that this response code
/// matches the expected response, per documentation. If not, the request most
/// likely wasn't succesful.
fn validate_response(cmd: AvrCommand, code: &str, response: &str) -> Result<(), Error> {
    let expected = match cmd {
        AvrCommand::SetVolume(_) => format!("VOL{}\r\n", &code[0..3]),
        AvrCommand::ChangeInput(_) => format!("FN{}\r\n", &code[0..2]),
        AvrCommand::Mute => "MUT0\r\n".to_owned(),
        AvrCommand::Unmute => "MUT1\r\n".to_owned(),
        AvrCommand::PowerOn => "PWR0\r\n".to_owned(),
        AvrCommand::PowerOff => "PWR1\r\n".to_owned(),
    };
    if response != expected {
        bail!(
            "AVR response doesn't match expected code: {:?}. Can't confirm update took place.",
            expected
        )
    }
    info!(
        "AVR response matches expected code: {:?}. Update appears to have worked.",
        expected
    );
    Ok(())
}
