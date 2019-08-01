use env_logger::Env;

mod site;
mod skill;
mod speech;

fn main() {
    env_logger::from_env(Env::default().default_filter_or("rustyassistant=info")).init();
    site::run().unwrap();
}
