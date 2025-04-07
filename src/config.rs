use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use std::fs;

pub struct ConfigBuilder {
    source: String,
}

impl ConfigBuilder {
    pub fn new(source: String) -> Self {
        Self { source }
    }

    pub fn build(self) -> Result<Config, crate::error::Error> {
        let content = fs::read_to_string(&self.source)?;
        let config: Config = serde_json::from_str(&content)?;
        Ok(config)
    }
}

#[derive(Debug)]
pub enum ConfigValue {
    Int(i32),
    Float(f64),
    Bool(bool),
    String(String),
    Array(Vec<ConfigValue>),
    Object(Box<std::collections::HashMap<String, ConfigValue>>),
}

impl<'de> serde::Deserialize<'de> for ConfigValue {
    fn deserialize<D>(deserializer: D) -> Result<ConfigValue, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Value {
            Int(i32),
            Float(f64),
            Bool(bool),
            String(String),
            Array(Vec<ConfigValue>),
            Object(HashMap<String, ConfigValue>),
        }

        match Value::deserialize(deserializer)? {
            Value::Int(i) => Ok(ConfigValue::Int(i)),
            Value::Float(f) => Ok(ConfigValue::Float(f)),
            Value::Bool(b) => Ok(ConfigValue::Bool(b)),
            Value::String(s) => Ok(ConfigValue::String(s)),
            Value::Array(arr) => Ok(ConfigValue::Array(arr)),
            Value::Object(map) => Ok(ConfigValue::Object(Box::new(map))),
        }
    }
}

impl ConfigValue {
    pub fn as_int(&self) -> Option<i32> {
        if let ConfigValue::Int(i) = self {
            Some(*i)
        } else {
            None
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        if let ConfigValue::Float(f) = self {
            Some(*f)
        } else {
            None
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        if let ConfigValue::String(s) = self {
            Some(s)
        } else {
            None
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        if let ConfigValue::Bool(b) = self {
            Some(*b)
        } else {
            None
        }
    }

    pub fn as_array(&self) -> Option<&Vec<ConfigValue>> {
        if let ConfigValue::Array(arr) = self {
            Some(arr)
        } else {
            None
        }
    }

    pub fn as_object(&self) -> Option<&HashMap<String, ConfigValue>> {
        if let ConfigValue::Object(obj) = self {
            Some(obj)
        } else {
            None
        }
    }

    pub fn get_path(&self, path: &str) -> Option<&ConfigValue> {
        let mut current = self;
        for segment in path.split('.') {
            match current {
                ConfigValue::Object(map) => {
                    current = map.get(segment)?;
                }
                _ => return None,
            }
        }
        Some(current)
    }
}

#[macro_export]
macro_rules! config_get {
    ($config:expr, $path:expr, $method:ident) => {
        $config.get_path($path).and_then(|v| v.$method())
    };
}

#[macro_export]
macro_rules! config_get_array {
    ($config:expr, $path:expr, $method:ident, $inner_method:ident) => {
        $config
            .get_path($path)
            .and_then(|v| v.$method())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| item.$inner_method())
                    .collect::<Vec<_>>()
            })
    };
}

pub type Config = ConfigValue;
