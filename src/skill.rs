/// This module contains all of the logic for processing Alexa requests and
/// returning the appropriate response.   
///
/// Once the request's intent is determined, this will call `avr::process()`
/// along with the appropriate `AvrCommand` to be executed.
use crate::{
    avr::{self, AvrCommand, AvrError},
    log_error, speech,
};
use alexa_sdk::{
    request::{IntentType, ReqType},
    Request, Response,
};
use failure::{ensure, Error, Fail};
use log::info;

/// Custom intents defined for this skill
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
    /// Convert from string to appropriate `UserIntent`
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

/// Entry point for web service to pass deserialized Request, process, &
/// return appropriate Response.   
///
/// LaunchRequests are left open, waiting for an appropriate Intent to be
/// requested. SessionEndedRequests doesn't need a verbal response, just
/// silently end. Other requests types aren't supported by this skill, it
/// will just send back "Hmm."
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

/// Processes intent from IntentRequests.
///
/// If it is one of the skills custom intents `IntentType::User`, it will
/// be passed along for futher processing.   
///
/// If an error occurs while processing the custom intent, it will be
/// logged and the appropriate response will be generated.
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
        Ok(response) => response,
        Err(e) => {
            log_error(&e);
            verbalize_error(e)
        }
    }
}

/// Process the custom intent further, getting slot values for applicable
/// intents.   
///
/// Volume and Input require a slot value, those are passed for further
/// processing. All other intents can directly call their respective
/// function.
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

/// Extract and verify the slot value for volume. It must be between
/// 1 and 10.   
///
/// Return `SkillError::Volume` if value can't be validated to notify user of
/// the correct use of this intent.   
///
/// Slot values are never None, request will pass as "?" if it is an unkown
/// value.   
///
/// `SkillError::Response` is mapped to errors returned by `avr::process`, so
/// the user is appropriately notified that their request didn't succeed.
fn volume(slot_value: Option<String>) -> Result<Response, Error> {
    let value = slot_value.unwrap();
    info!("Slot Value: {}", value);

    let value =
        validate_volume_value(value).map_err(|inner| Error::from(SkillError::Volume { inner }))?;
    info!("Got valid volume value: {}", value);

    avr::process(AvrCommand::SetVolume(value))?;
    Ok(end_ok())
}

/// Validate volume value is an integer between 1 and 10.
fn validate_volume_value(value: String) -> Result<u8, Error> {
    let int = value.parse::<u8>()?;
    ensure!(int > 0 && int < 11, "Volume not between 1 and 10");
    Ok(int)
}

/// Extract and verify the slot value for input. It must be between
/// 1 and 22.
///
/// Return `SkillError::Input` if value can't be validated to notify user of
/// the correct use of this intent.
fn input(slot_value: Option<String>) -> Result<Response, Error> {
    let value = slot_value.unwrap();
    info!("Slot Value: {}", value);

    let value =
        validate_input_value(value).map_err(|inner| Error::from(SkillError::Input { inner }))?;
    info!("Got valid input value: {}", value);

    avr::process(AvrCommand::ChangeInput(value))?;
    Ok(end_ok())
}

/// Validate input value is an integer between 1 and 22.
fn validate_input_value(value: String) -> Result<u8, Error> {
    let int = value.parse::<u8>()?;
    ensure!(int > 0 && int < 23, "Input not between 1 and 22");
    Ok(int)
}

/// Process `AvrCommand::Mute`
fn mute() -> Result<Response, Error> {
    avr::process(AvrCommand::Mute)?;
    Ok(end_ok())
}

/// Process `AvrCommand::Unmute`
fn unmute() -> Result<Response, Error> {
    avr::process(AvrCommand::Unmute)?;
    Ok(end_ok())
}

/// Process `AvrCommand::PowerOn`
fn on() -> Result<Response, Error> {
    avr::process(AvrCommand::PowerOn)?;
    Ok(end_ok())
}

/// Process `AvrCommand::PowerOff`
fn off() -> Result<Response, Error> {
    avr::process(AvrCommand::PowerOff)?;
    Ok(end_ok())
}

/// Response using `speech::hello` that is left open
fn open_hello() -> Response {
    Response::new(false).speech(speech::hello())
}

/// Response using `speech::help` that is left open
fn open_help() -> Response {
    Response::new(false).speech(speech::help())
}

/// Silent response that ends
fn end_silent() -> Response {
    Response::end()
}

/// Response using `speech::ok` that ends
fn end_ok() -> Response {
    Response::new(true).speech(speech::ok())
}

/// Response using `speech::hmm` that ends
fn end_hmm() -> Response {
    Response::new(true).speech(speech::hmm())
}

/// Response using `speech::volume_error` that notifies user their Volume
/// intent request contained an incorrect slot value.
fn end_volume_error() -> Response {
    Response::new(true).speech(speech::volume_error())
}

/// Response using `speech::input_error` that notifies user their Input
/// intent request contained an incorrect slot value.
fn end_input_error() -> Response {
    Response::new(true).speech(speech::input_error())
}

/// Response using `speech::response_error` that notifies user their request
/// didn't succeed because of some error communicating with the AVR.
fn end_response_error() -> Response {
    Response::new(true).speech(speech::response_error())
}

fn end_error_power_already_off() -> Response {
    Response::new(true).speech(speech::error_power_already_off())
}

fn end_error_power_already_on() -> Response {
    Response::new(true).speech(speech::error_power_already_on())
}

fn end_error_turn_power_on() -> Response {
    Response::new(true).speech(speech::error_turn_power_on())
}

/// Error for this module, mainly used to determine appropriate speech to
/// include in the Response
#[derive(Fail, Debug)]
enum SkillError {
    #[fail(display = "Volume error: {}", inner)]
    Volume { inner: Error },
    #[fail(display = "Input error: {}", inner)]
    Input { inner: Error },
}

fn verbalize_error(e: Error) -> Response {
    match e.downcast::<SkillError>() {
        Ok(e) => match e {
            SkillError::Volume { .. } => end_volume_error(),
            SkillError::Input { .. } => end_input_error(),
        },
        Err(e) => {
            if let Ok(e) = e.downcast::<AvrError>() {
                match e {
                    AvrError::PowerAlreadyOn => end_error_power_already_on(),
                    AvrError::PowerAlreadyOff => end_error_power_already_off(),
                    AvrError::PowerOffCantProcess => end_error_turn_power_on(),
                    _ => end_response_error(),
                }
            } else {
                end_response_error()
            }
        }
    }
}
