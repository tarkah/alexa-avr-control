use failure::{bail, Error, Fail};
use log::info;

pub fn process_volume(value: u8) -> Result<(), Error> {
    let code = AvrCommand::SetVolume(value).code();
    info!("Sending code to AVR: {:?}", code);
    if value == 10 {
        bail!("Test error on modifying volume")
    }
    Ok(())
}

pub fn process_input(value: u8) -> Result<(), Error> {
    let code = AvrCommand::ChangeInput(value).code();
    info!("Sending code to AVR: {:?}", code);
    if value == 4 {
        bail!("Test error on modifying input")
    }
    Ok(())
}

pub fn process_mute() -> Result<(), Error> {
    let code = AvrCommand::Mute.code();
    info!("Sending code to AVR: {:?}", code);
    Ok(())
}

pub fn process_unmute() -> Result<(), Error> {
    let code = AvrCommand::Unmute.code();
    info!("Sending code to AVR: {:?}", code);
    Ok(())
}

pub fn process_on() -> Result<(), Error> {
    let code = AvrCommand::PowerOn.code();
    info!("Sending code to AVR: {:?}", code);
    Ok(())
}

pub fn process_off() -> Result<(), Error> {
    let code = AvrCommand::PowerOff.code();
    info!("Sending code to AVR: {:?}", code);
    bail!(AvrError::Timeout)
}

enum AvrCommand {
    SetVolume(u8),
    Mute,
    Unmute,
    PowerOn,
    PowerOff,
    ChangeInput(u8),
    QueryVolume,
    QueryMute,
    QueryPower,
    QueryInput,
}

impl<'a> AvrCommand {
    fn code(&self) -> String {
        match &self {
            AvrCommand::SetVolume(n) => get_volume_code(*n),
            AvrCommand::ChangeInput(n) => get_input_code(*n),
            AvrCommand::PowerOn => "PO\r".to_string(),
            AvrCommand::PowerOff => "PF\r".to_string(),
            AvrCommand::Mute => "MO\r".to_string(),
            AvrCommand::Unmute => "MF\r".to_string(),
            AvrCommand::QueryVolume => "?V\r".to_string(),
            AvrCommand::QueryMute => "?M\r".to_string(),
            AvrCommand::QueryPower => "?P\r".to_string(),
            AvrCommand::QueryInput => "?F\r".to_string(),
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
    let mut code = code.to_string();
    code.push_str("FN\r");
    code
}

#[derive(Debug, Fail)]
enum AvrError {
    #[fail(display = "Timeout. Could not reach AVR.")]
    Timeout,
}
