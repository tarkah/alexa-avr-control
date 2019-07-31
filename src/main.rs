extern crate self as assistant;

mod site;
mod skill;

fn main() {
    assistant::site::run().unwrap();
}
