/// All Alexa speech options go here
use alexa_sdk::response::Speech;

pub fn hello() -> Speech {
    Speech::plain("What can I do for you?")
}

pub fn ok() -> Speech {
    Speech::plain("Ok.")
}

pub fn hmm() -> Speech {
    Speech::plain("Hmm.")
}

pub fn help() -> Speech {
    Speech::plain("Try commands such as: on, off, mute, unmute, volume 2, input3.")
}

pub fn volume_error() -> Speech {
    Speech::plain("Volume must be between 1 and 10.")
}

pub fn input_error() -> Speech {
    Speech::plain("Input must be between 1 and 22.")
}

pub fn response_error() -> Speech {
    Speech::plain("Don't think it worked...")
}
