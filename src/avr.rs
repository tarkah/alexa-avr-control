/// This module contains all the logic for converting the requested skill
/// Intent into the proper AVR command code that can be sent over telnet
/// to control the AVR. It will also validate that the response from the
/// AVR via telnet matches the expected response, confirming that the command
/// was executed successfuly.
use crate::{CHANNEL_A, CHANNEL_B};
use crossbeam_channel::select;
use failure::{bail, Error, Fail};
use log::{debug, info};
use std::time::Duration;

/// Entry point to use from skill module to request the appropriate command
pub fn process(cmd: AvrCommand) -> Result<(), Error> {
    send_and_validate(cmd)?;
    Ok(())
}

/// Commands that can be sent to AVR
#[derive(PartialEq)]
pub enum AvrCommand {
    SetVolume(u8),
    Mute,
    Unmute,
    PowerOn,
    PowerOff,
    ChangeInput(u8),
    VolumeDown,
    VolumeUp,
}

enum AvrQuery {
    Volume,
    Mute,
    Power,
    Input,
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
            AvrCommand::VolumeDown => "VD\r\n".to_owned(),
            AvrCommand::VolumeUp => "VU\r\n".to_owned(),
        }
    }

    fn query(&self) -> Result<String, Error> {
        let query_type = match &self {
            AvrCommand::SetVolume(_) => AvrQuery::Volume,
            AvrCommand::ChangeInput(_) => AvrQuery::Input,
            AvrCommand::PowerOn => AvrQuery::Power,
            AvrCommand::PowerOff => AvrQuery::Power,
            AvrCommand::Mute => AvrQuery::Mute,
            AvrCommand::Unmute => AvrQuery::Mute,
            AvrCommand::VolumeDown => AvrQuery::Volume,
            AvrCommand::VolumeUp => AvrQuery::Volume,
        };
        query_type.query()
    }

    fn expected(&self) -> String {
        match &self {
            AvrCommand::SetVolume(_) => format!("VOL{}\r\n", &self.code()[0..3]),
            AvrCommand::ChangeInput(_) => format!("FN{}\r\n", &self.code()[0..2]),
            AvrCommand::Mute => "MUT0\r\n".to_owned(),
            AvrCommand::Unmute => "MUT1\r\n".to_owned(),
            AvrCommand::PowerOn => "PWR0\r\n".to_owned(),
            AvrCommand::PowerOff => "PWR2\r\n".to_owned(),
            AvrCommand::VolumeDown => "VOL".to_owned(),
            AvrCommand::VolumeUp => "VOL".to_owned(),
        }
    }
}

impl AvrQuery {
    /// Convert enum to the appropriate telnet command supported
    /// by the AVR
    fn code(&self) -> String {
        match &self {
            AvrQuery::Volume => "?V\r".to_owned(),
            AvrQuery::Mute => "?M\r".to_owned(),
            AvrQuery::Power => "?P\r".to_owned(),
            AvrQuery::Input => "?F\r".to_owned(),
        }
    }

    fn query(&self) -> Result<String, Error> {
        send_command(&self.code())
    }
}

/// Convert volume of 1 - 10 to appropriate AVR volume code.   
///
/// 161 is equal to 0.0dB and I don't want to set any higher via this skill,
/// so I've set this as the ceiling.
///
/// Must be padded to three digits: "{:0>3}"
fn get_volume_code(n: u8) -> String {
    let ceiling = 101.0;
    let weight = f32::from(n) / 10.0;
    let volume = (weight * ceiling).ceil() as u8;
    let mut volume = format!("{:0>3}", volume);
    volume.push_str("VL\r");
    volume
}

/// Convert input to AVR input code.
fn get_input_code(n: u8) -> String {
    let code = match n {
        1 => "25",  // BD
        2 => "49",  // Game
        3 => "19",  // HDMI 1
        4 => "15",  // DVR/BDR
        5 => "10",  // VIDEO 1(VIDEO)
        6 => "14",  // VIDEO 2
        7 => "05",  // TV/SAT
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
        23 => "04", // DVD
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
    info!("Translated to code: {:?}", &cmd.code());

    power_validation(&cmd)?;

    // Don't care about this response (unreliable), will query to confirm
    match cmd {
        AvrCommand::SetVolume(_) => {
            volume_control(cmd.code())?;
            // Sleep to allow AVR to process before querying for final Vol
            std::thread::sleep(Duration::from_millis(2_000));
        }
        AvrCommand::PowerOn => {
            let _ = send_command(&cmd.code())?;
            // Sleep to allow AVR to process before querying for final Vol
            std::thread::sleep(Duration::from_millis(1_000));
        }
        _ => {
            let _ = send_command(&cmd.code())?;
        }
    }

    let query_response = cmd.query()?;

    validate_response(cmd, query_response)
}

fn power_validation(cmd: &AvrCommand) -> Result<(), Error> {
    let current_power = AvrQuery::Power.query()?;
    if current_power.contains(&AvrCommand::PowerOff.expected()) && cmd != &AvrCommand::PowerOn {
        if cmd == &AvrCommand::PowerOff {
            return Err(AvrError::PowerAlreadyOff.into());
        } else {
            return Err(AvrError::PowerOffCantProcess.into());
        }
    } else if current_power.contains(&AvrCommand::PowerOn.expected()) && cmd == &AvrCommand::PowerOn
    {
        return Err(AvrError::PowerAlreadyOn.into());
    }
    Ok(())
}

fn volume_control(code: String) -> Result<(), Error> {
    let current_volume = AvrQuery::Volume
        .query()?
        .trim_end()
        .trim_start_matches("VOL")
        .parse::<i8>()?;
    let desired_volume = &code[0..3].parse::<i8>()?;
    let diff = desired_volume - current_volume;
    let steps = diff / 2;
    let vol_adj = if steps > 0 {
        AvrCommand::VolumeUp.code().repeat(steps as usize)
    } else {
        AvrCommand::VolumeDown.code().repeat(steps.abs() as usize)
    };

    send_command(&vol_adj)?;

    Ok(())
}

fn send_command(code: &str) -> Result<String, Error> {
    // Clear channel A if full, it shouldn't be
    if CHANNEL_A.0.is_full() {
        select! {
            recv(CHANNEL_A.1) -> _ => {}
            default => {}
        }
        debug!("Had to clear channel A");
    }
    CHANNEL_A.0.send(code.to_owned())?;
    debug!("Sent code via channel A: {:?}", code);

    get_response()
}

/// Get response code back from AVR. If this response takes longer than 1.5
/// second, assume error.
fn get_response() -> Result<String, Error> {
    select! {
        recv(CHANNEL_B.1) -> msg => {
            let msg = msg?;
            debug!("Response code received via channel B: {:?}", msg);
            Ok(msg)
        },
        default(Duration::from_millis(1_500)) => {
            bail!(AvrError::Timeout);
        }
    }
}

/// AVR sends back code validating the request. Confirm that this response code
/// matches the expected response, per documentation. If not, the request most
/// likely wasn't succesful.
fn validate_response(cmd: AvrCommand, response: String) -> Result<(), Error> {
    let expected = cmd.expected();
    if !response.contains(&expected) {
        bail!(AvrError::ResponseDoesntMatch { expected });
    }
    info!(
        "AVR response matches expected code: {:?}. Update appears to have worked.",
        expected
    );
    Ok(())
}

#[derive(Fail, Debug)]
pub enum AvrError {
    #[fail(display = "Timeout. Didn't get response from AVR.")]
    Timeout,
    #[fail(display = "Power already off.")]
    PowerAlreadyOff,
    #[fail(display = "Power already on.")]
    PowerAlreadyOn,
    #[fail(display = "Power is off, it must be turned on to execute command.")]
    PowerOffCantProcess,
    #[fail(
        display = "AVR response doesn't match expected code: {:?}. Can't confirm update took place.",
        expected
    )]
    ResponseDoesntMatch { expected: String },
}
