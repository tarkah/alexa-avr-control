use crate::{avr, speech};
use alexa_sdk::{
    request::{IntentType, ReqType},
    Request, Response,
};
use failure::{bail, format_err, Error, ResultExt};
use log::{error, info};

enum UserIntent {
    Volume,
    Mute,
    Unmute,
    On,
    Off,
    Input,
    Other,
}

impl<'a> From<&'a str> for UserIntent {
    fn from(s: &'a str) -> UserIntent {
        match s {
            "Volume" => UserIntent::Volume,
            "Mute" => UserIntent::Mute,
            "Unmute" => UserIntent::Unmute,
            "On" => UserIntent::On,
            "Off" => UserIntent::Off,
            "Input" => UserIntent::Input,
            _ => UserIntent::Other,
        }
    }
}

impl From<&String> for UserIntent {
    fn from(s: &String) -> UserIntent {
        UserIntent::from(s.as_str())
    }
}

pub fn process_request(request: Request) -> Response {
    let reqtype = request.reqtype();
    info!("Request Type: {:?}", reqtype);

    match reqtype {
        ReqType::IntentRequest => process_intent(request),
        ReqType::LaunchRequest => open_hello(),
        ReqType::SessionEndedRequest => end_silent(),
        _ => end_hmm(),
    }
}

fn process_intent(request: Request) -> Response {
    let intent = request.intent();
    info!("Intent: {:?}", intent);

    let response_result = match intent {
        IntentType::User(s) => process_user_intent(s, request),
        IntentType::Help => Ok(open_help()),
        IntentType::Cancel => Ok(end_ok()),
        IntentType::Stop => Ok(end_ok()),
        IntentType::NavigateHome => Ok(end_ok()),
        _ => Ok(end_hmm()),
    };

    match response_result {
        Err(e) => {
            log_error(e);
            end_hmm()
        }
        Ok(response) => response,
    }
}

fn process_user_intent(mut s: String, request: Request) -> Result<Response, Error> {
    let user_intent = UserIntent::from(&s);
    s.push_str("_slot");
    let maybe_slot_value = request.slot_value(&s);

    match user_intent {
        UserIntent::Volume => volume(maybe_slot_value),
        UserIntent::Input => input(maybe_slot_value),
        UserIntent::Mute => mute(),
        UserIntent::Unmute => unmute(),
        UserIntent::On => on(),
        UserIntent::Off => off(),
        _ => Ok(end_hmm()),
    }
}

fn volume(slot_value: Option<String>) -> Result<Response, Error> {
    let value =
        slot_value.ok_or_else(|| format_err!("No value provided for UserIntent::Volume"))?;
    info!("Slot Value: {}", value);

    let value = validate_volume_value(value)?;
    info!("Got valid volume value: {}", value);

    avr::process_volume(value)?;
    info!("Volume successfully changed");

    Ok(end_ok())
}

fn validate_volume_value(value: String) -> Result<u8, Error> {
    let int = value
        .parse::<u8>()
        .context(format_err!("Volume value provided not a valid u8"))?;

    if int == 0 || int > 10 {
        bail!("Volume value not between 1 and 10")
    }
    Ok(int)
}

fn input(slot_value: Option<String>) -> Result<Response, Error> {
    let value = slot_value.ok_or_else(|| format_err!("No value provided for UserIntent::Input"))?;
    info!("Slot Value: {}", value);

    let value = validate_input_value(value)?;
    info!("Got valid input value: {}", value);

    avr::process_input(value)?;
    info!("Input successfully changed");

    Ok(end_ok())
}

fn validate_input_value(value: String) -> Result<u8, Error> {
    let int = value
        .parse::<u8>()
        .context(format_err!("Input value provided not a valid u8"))?;

    if int == 0 || int > 22 {
        bail!("Input value not between 1 and 22")
    }
    Ok(int)
}

fn mute() -> Result<Response, Error> {
    avr::process_mute()?;

    info!("AVR successfully muted");
    Ok(end_ok())
}

fn unmute() -> Result<Response, Error> {
    avr::process_unmute()?;

    info!("AVR successfully unmuted");
    Ok(end_ok())
}

fn on() -> Result<Response, Error> {
    avr::process_on()?;

    info!("AVR successfully powered on");
    Ok(end_ok())
}

fn off() -> Result<Response, Error> {
    avr::process_off()?;

    info!("AVR successfully powered off");
    Ok(end_ok())
}

fn open_hello() -> Response {
    Response::new(false).speech(speech::hello())
}

fn open_help() -> Response {
    Response::new(false).speech(speech::help())
}

fn end_silent() -> Response {
    Response::end()
}

fn end_ok() -> Response {
    Response::new(true).speech(speech::ok())
}

fn end_hmm() -> Response {
    Response::new(true).speech(speech::hmm())
}

fn log_error(e: Error) {
    error!("{}", e);
    for cause in e.iter_causes() {
        error!("Caused by: {}", cause);
    }
}
