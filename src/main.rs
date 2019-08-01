use env_logger::Env;

mod avr;
mod site;
mod skill;
mod speech;

fn main() {
    env_logger::from_env(Env::default().default_filter_or("alexa_avr_control=info")).init();
    site::run().unwrap();
}
