use std::{fmt, io};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum McLogError {
    #[error("Failed to get the logs file metadata: {0:?}")]
    FailedToGetMetadata(io::Error),
    #[error("Failed to seek buf to {0} in log file: {1:?}")]
    FailedToSeekTo(u64, io::Error),
    #[error("Invalid line: {0:?}")]
    InvalidLine(io::Error),
    #[error("Failed to write escaped characters to line: {0:?}")]
    FailedToWriteEscapedChars(fmt::Error),
    #[error("Not enough chars to get identifier: {0}")]
    NotEnoughForIdentifier(String),
    #[error("Not enough chars to get the start of the SNBT: {0}")]
    NotEnoughForSnbtStart(String),
    #[error("Missing {0} in the parsed snbt: {1:?}")]
    MissingField(&'static str, vanity_nbt::snbt::Compound),
    #[error("While extracting snbt, expected {0} but got: {1}")]
    MismatchedType(&'static str, vanity_nbt::snbt::Value),
    #[error("Unknown log level: {0}")]
    UnknownLevel(String),
    #[error(transparent)]
    InvalidSnbt(#[from] vanity_nbt::VanityError),
}
