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
