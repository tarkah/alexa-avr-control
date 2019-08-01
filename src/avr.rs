use failure::{format_err, Error};

pub fn change_volume(value: u8) -> Result<(), Error> {
    if value == 10 {
        return Err(format_err!("Test error on modifying volume"));
    }
    Ok(())
}

pub fn change_input(value: u8) -> Result<(), Error> {
    if value == 4 {
        return Err(format_err!("Test error on modifying input"));
    }
    Ok(())
}
