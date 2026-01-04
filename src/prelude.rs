use std::fmt::{Debug, Display};

pub use crate::CLI_OPTS;
pub use crate::EVENTS;
pub use crate::LOGGER;

pub use crate::events::EventSource;

pub fn log<S: Into<String> + Debug + Display>(msg: S) {
    LOGGER.log(msg)
}
