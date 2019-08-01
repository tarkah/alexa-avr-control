use crate::speech;
use alexa_sdk::{
    request::{IntentType, ReqType},
    Request, Response,
};
use failure::{format_err, Error, ResultExt};
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
        ReqType::LaunchRequest => {
            info!("New request without intent, standby...");
            Response::new(false)
        }
        ReqType::SessionEndedRequest => silently_end(),
        _ => silently_end(),
    }
}

fn process_intent(request: Request) -> Response {
    let intent = request.intent();
    info!("Intent: {:?}", intent);

    if request.is_new() && intent == IntentType::None {
        info!("New request without intent, standby...");
        return Response::new(false);
    }

    let response_result = match intent {
        IntentType::User(s) => process_user_intent(s, request),
        _ => Ok(silently_end()),
    };

    match response_result {
        Err(e) => {
            log_error(e);
            silently_end()
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
        _ => Ok(silently_end()),
    }
}

fn volume(slot_value: Option<String>) -> Result<Response, Error> {
    let value =
        slot_value.ok_or_else(|| format_err!("No value provided for UserIntent::Volume"))?;
    info!("Slot Value: {}", value);

    let value = validate_volume_value(value)?;
    info!("Got valid volume value: {}", value);

    change_volume(value)?;
    info!("Volume successfully changed");

    Ok(alexa_sdk::Response::new(true).speech(speech::ok()))
}

fn validate_volume_value(value: String) -> Result<u8, Error> {
    let int = value
        .parse::<u8>()
        .context(format_err!("Volume value provided not a valid u8"))?;

    if int == 0 || int > 10 {
        return Err(format_err!("Volume value not between 1 and 10"));
    }
    Ok(int)
}

fn input(slot_value: Option<String>) -> Result<Response, Error> {
    let value = slot_value.ok_or_else(|| format_err!("No value provided for UserIntent::Input"))?;
    info!("Slot Value: {}", value);

    let value = validate_input_value(value)?;
    info!("Got valid input value: {}", value);

    change_input(value)?;
    info!("Input successfully changed");

    Ok(alexa_sdk::Response::new(true).speech(speech::ok()))
}

fn validate_input_value(value: String) -> Result<u8, Error> {
    let int = value
        .parse::<u8>()
        .context(format_err!("Input value provided not a valid u8"))?;

    if int == 0 || int > 4 {
        return Err(format_err!("Input value not between 1 and 4"));
    }
    Ok(int)
}

fn silently_end() -> Response {
    Response::end()
}

fn change_volume(value: u8) -> Result<(), Error> {
    if value == 10 {
        return Err(format_err!("Test error on modifying volume"));
    }
    Ok(())
}

fn change_input(value: u8) -> Result<(), Error> {
    if value == 4 {
        return Err(format_err!("Test error on modifying input"));
    }
    Ok(())
}

fn log_error(e: Error) {
    error!("{}", e);
    for cause in e.iter_causes() {
        error!("Caused by: {}", cause);
    }
}
