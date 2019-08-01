use alexa_sdk::response::Speech;

pub fn ok() -> Speech {
    Speech::plain("ok.")
}
