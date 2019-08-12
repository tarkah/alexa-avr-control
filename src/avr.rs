use crate::{CHANNEL_A, CHANNEL_B};
use crossbeam_channel::select;
use failure::{bail, ensure, Error};
use log::{debug, info};
use std::time::Duration;

pub fn process(cmd: AvrCommand) -> Result<(), Error> {
    send_and_validate(cmd)?;
    Ok(())
}

pub enum AvrCommand {
    SetVolume(u8),
    Mute,
    Unmute,
    PowerOn,
    PowerOff,
    ChangeInput(u8),
}

impl AvrCommand {
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

fn get_volume_code(n: u8) -> String {
    let ceiling = 161.0; // 161 is 0.0dB
    let weight = f32::from(n) / 10.0;
    let volume = (weight * ceiling).floor() as u8;
    let mut volume = format!("{:0>3}", volume);
    volume.push_str("VL\r");
    volume
}

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

fn send_and_validate(cmd: AvrCommand) -> Result<(), Error> {
    let code = cmd.code();
    info!("Translated to code: {:?}", code);

    if !CHANNEL_A.0.is_empty() {
        bail!("Channel is full...");
    }
    CHANNEL_A.0.send(code.clone())?;
    debug!("Sent code via channel A: {:?}", code);

    let response = get_response()?;
    validate_response(cmd, &code, &response)
}

fn get_response() -> Result<String, Error> {
    select! {
        recv(CHANNEL_B.1) -> msg => {
            let msg = msg?;
            debug!("Response code received via channel B: {:?}", msg);
            Ok(msg)
        },
        default(Duration::from_millis(1_000)) => bail!("Timeout. Didn't get response from AVR."),
    }
}

// Need to change this to correctly validate response format vs code sent
fn validate_response(cmd: AvrCommand, code: &str, response: &str) -> Result<(), Error> {
    let validated = match cmd {
        AvrCommand::SetVolume(_) => {
            let vol = &code[0..3];
            let expected = format!("VOL{}\r\n", vol);
            response == expected
        }
        _ => true,
    };
    if !validated {
        bail!("AVR response doesn't match expected code. Can't confirm update took place.")
    }
    info!("AVR response matches code sent.");
    Ok(())
}
