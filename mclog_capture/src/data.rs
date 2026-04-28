use serde::Serialize;
use tracing::Level;
use uuid::Uuid;
use vanity_nbt::snbt::{Compound, Value};

use crate::error::McLogError;

#[derive(Debug, Serialize, Clone)]
pub struct LogMessage {
    pub dimension: String,
    pub entity: LogEntity,
    pub function: String,
    pub level: LogLevel,
    pub message: Value,
    pub pos: [f64; 3],
    pub rotation: [f32; 2],
    pub tick: i32,
}

#[inline]
fn int_array_to_uuid(l: [i32; 4]) -> Uuid {
    let f1: [u8; 4] = l[0].to_be_bytes().try_into().unwrap();
    let f2: [u8; 4] = l[1].to_be_bytes().try_into().unwrap();
    let f3: [u8; 4] = l[2].to_be_bytes().try_into().unwrap();
    let f4: [u8; 4] = l[3].to_be_bytes().try_into().unwrap();
    #[rustfmt::skip]
    let f = [
        f1[0], f1[1], f1[2], f1[3], 
        f2[0], f2[1], f2[2], f2[3], 
        f3[0], f3[1], f3[2], f3[3], 
        f4[0], f4[1], f4[2], f4[3],
    ];

    Uuid::from_bytes(f)
}

impl LogMessage {
    pub fn from_snbt(nbt: Value) -> Result<Self, McLogError> {
        let mut comp = match nbt {
            Value::Compound(v) => v,
            _ => return Err(McLogError::MismatchedType("compound", nbt)),
        };

        let tick = {
            let v_tick = match comp.swap_remove("tick") {
                Some(v) => v,
                None => return Err(McLogError::MissingField("tick", comp)),
            };

            match v_tick {
                Value::Int(v) => v,
                _ => return Err(McLogError::MismatchedType("int", v_tick)),
            }
        };

        let function = {
            let v_function = match comp.swap_remove("function") {
                Some(v) => v,
                None => return Err(McLogError::MissingField("function", comp)),
            };

            match v_function {
                Value::String(v) => v,
                _ => return Err(McLogError::MismatchedType("string", v_function)),
            }
        };

        let dimension = {
            let v_dimension = match comp.swap_remove("dimension") {
                Some(v) => v,
                None => return Err(McLogError::MissingField("dimension", comp)),
            };

            match v_dimension {
                Value::String(v) => v,
                _ => return Err(McLogError::MismatchedType("string", v_dimension)),
            }
        };

        let rotation = {
            let v_rotation = match comp.swap_remove("rotation") {
                Some(v) => v,
                None => return Err(McLogError::MissingField("rotation", comp)),
            };

            let l_rotation = match v_rotation {
                Value::List(v) => v,
                _ => return Err(McLogError::MismatchedType("list", v_rotation)),
            };

            let rotation = l_rotation
                .into_iter()
                .map(|s| s.into_float().unwrap())
                .collect::<Vec<f32>>();

            [
                *rotation.get(0).unwrap_or(&0.),
                *rotation.get(1).unwrap_or(&0.),
            ]
        };

        let pos = {
            let v_position = match comp.swap_remove("pos") {
                Some(v) => v,
                None => return Err(McLogError::MissingField("pos", comp)),
            };

            let l_position = match v_position {
                Value::List(v) => v,
                _ => return Err(McLogError::MismatchedType("list", v_position)),
            };

            let position = l_position
                .into_iter()
                .map(|s| s.into_double().unwrap())
                .collect::<Vec<f64>>();

            [
                *position.get(0).unwrap_or(&0.),
                *position.get(1).unwrap_or(&0.),
                *position.get(2).unwrap_or(&0.),
            ]
        };

        let message = {
            match comp.swap_remove("message") {
                Some(v) => v,
                None => return Err(McLogError::MissingField("message", comp)),
            }
        };

        let level = {
            let v_level = match comp.swap_remove("level") {
                Some(v) => v,
                None => return Err(McLogError::MissingField("level", comp)),
            };

            match v_level {
                Value::String(v) => v,
                _ => return Err(McLogError::MismatchedType("string", v_level)),
            }
        };
        let level = LogLevel::from_str(&level).ok_or(McLogError::UnknownLevel(level))?;

        let mut entity_comp = {
            let v_entity_comp = match comp.swap_remove("entity") {
                Some(v) => v,
                None => return Err(McLogError::MissingField("entity", comp)),
            };

            match v_entity_comp {
                Value::Compound(v) => v,
                _ => return Err(McLogError::MismatchedType("compound", v_entity_comp)),
            }
        };

        let data = {
            let v_data = match entity_comp.swap_remove("data") {
                Some(v) => v,
                None => return Err(McLogError::MissingField("data", entity_comp)),
            };

            match v_data {
                Value::Compound(v) => v,
                _ => return Err(McLogError::MismatchedType("compound", v_data)),
            }
        };
        let uuid = {
            let v_uuid = match entity_comp.swap_remove("uuid") {
                Some(v) => v,
                None => return Err(McLogError::MissingField("uuid", entity_comp)),
            };

            let uuid = match v_uuid {
                Value::IntArray(v) => v,
                _ => return Err(McLogError::MismatchedType("int_array", v_uuid)),
            };

            let uuid = if uuid.is_empty() {
                [0, 0, 0, 0]
            } else {
                [uuid[0], uuid[1], uuid[2], uuid[3]]
            };

            int_array_to_uuid(uuid)
        };

        let r#type = {
            let v_type = match entity_comp.swap_remove("type") {
                Some(v) => v,
                None => return Err(McLogError::MissingField("type", entity_comp)),
            };

            match v_type {
                Value::String(v) => v,
                _ => return Err(McLogError::MismatchedType("compound", v_type)),
            }
        };

        let entity = LogEntity { data, uuid, r#type };

        Ok(LogMessage {
            dimension,
            entity,
            function,
            level,
            message,
            pos,
            rotation,
            tick,
        })
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct LogEntity {
    pub data: Compound,
    pub uuid: Uuid,
    pub r#type: String,
}

#[derive(Debug, Serialize, Clone)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    #[inline]
    pub fn from_str(v: &str) -> Option<Self> {
        match v {
            "trace" => Some(Self::Trace),
            "debug" => Some(Self::Debug),
            "info" => Some(Self::Info),
            "warn" => Some(Self::Warn),
            "error" => Some(Self::Error),
            _ => None,
        }
    }

    #[inline]
    pub fn to_tracing(&self) -> Level {
        match self {
            Self::Trace => Level::TRACE,
            Self::Debug => Level::DEBUG,
            Self::Info => Level::INFO,
            Self::Warn => Level::WARN,
            Self::Error => Level::ERROR,
        }
    }
}
