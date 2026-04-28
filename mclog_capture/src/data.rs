use serde::Serialize;
use uuid::Uuid;
use vanity_nbt::snbt::{Compound, Value};

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

impl LogMessage {
    pub fn from_snbt(nbt: Value) -> Option<Self> {
        let mut comp = nbt.into_compound()?;

        let tick = comp.swap_remove("tick")?.into_int()?;
        let function = comp.swap_remove("function")?.into_string()?;
        let dimension = comp.swap_remove("dimension")?.into_string()?;

        let rotation = comp
            .swap_remove("rotation")?
            .into_list()?
            .into_iter()
            .map(|s| s.into_float().unwrap())
            .collect::<Vec<f32>>();
        let rotation = [
            *rotation.get(0).unwrap_or(&0.),
            *rotation.get(1).unwrap_or(&0.),
        ];

        let positon = comp
            .swap_remove("pos")?
            .into_list()?
            .into_iter()
            .map(|s| s.into_double().unwrap())
            .collect::<Vec<f64>>();
        let pos = [
            *positon.get(0).unwrap_or(&0.),
            *positon.get(1).unwrap_or(&0.),
            *positon.get(2).unwrap_or(&0.),
        ];

        let message = comp.swap_remove("message")?;
        let level = comp.swap_remove("level")?.into_string()?;
        let level = LogLevel::from_str(&level)?;

        let mut entity_comp = comp.swap_remove("entity")?.into_compound()?;
        let data = entity_comp.swap_remove("data")?.into_compound()?;
        let uuid = entity_comp.swap_remove("uuid")?.into_int_array()?;
        let uuid = if uuid.is_empty() {
            [0, 0, 0, 0]
        } else {
            [uuid[0], uuid[1], uuid[2], uuid[3]]
        };
        let uuid = int_array_to_uuid(uuid);
        let r#type = entity_comp.swap_remove("type")?.into_string()?;
        let entity = LogEntity { data, uuid, r#type };

        Some(LogMessage {
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
}
