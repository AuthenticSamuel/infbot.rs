use crate::{Data, Error};

pub mod bot;
pub mod help;
pub mod module;
pub mod server;

pub fn all() -> Vec<poise::Command<Data, Error>> {
    return vec![bot::bot(), help::help(), module::module(), server::server()];
}
