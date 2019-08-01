use failure::{bail, Error, Fail};

pub fn process_volume(value: u8) -> Result<(), Error> {
    if value == 10 {
        bail!("Test error on modifying volume")
    }
    Ok(())
}

pub fn process_input(value: u8) -> Result<(), Error> {
    if value == 4 {
        bail!("Test error on modifying input")
    }
    Ok(())
}

pub fn process_mute() -> Result<(), Error> {
    Ok(())
}

pub fn process_unmute() -> Result<(), Error> {
    Ok(())
}

pub fn process_on() -> Result<(), Error> {
    Ok(())
}

pub fn process_off() -> Result<(), Error> {
    bail!(AvrError::Timeout)
}

#[derive(Debug, Fail)]
enum AvrError {
    #[fail(display = "Timeout. Could not reach AVR.")]
    Timeout,
}
