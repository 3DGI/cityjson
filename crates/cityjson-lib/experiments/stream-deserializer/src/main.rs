//! From https://users.rust-lang.org/t/step-past-errors-in-serde-json-streamdeserializer/84228/8?u=balazsdukai

use core::fmt;
use serde::Deserialize; // 1.0.147
use serde_json::de::{Deserializer, StrRead, StreamDeserializer};
use serde_json::{Error, Value};

#[derive(Deserialize, Debug)]
struct Final {
    id: String,
}

#[derive(Debug)]
struct JsonError {
    error: Error,
    value: Option<Value>, // Some(_) if JSON was syntactically valid
}

impl fmt::Display for JsonError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.error)?;

        if let Some(value) = self.value.as_ref() {
            write!(formatter, ", value: {}", value)?;
        }

        Ok(())
    }
}

impl std::error::Error for JsonError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
    }
}

struct ResilientStreamDeserializer<'de, T> {
    json: &'de str,
    stream: StreamDeserializer<'de, StrRead<'de>, T>,
    last_ok_pos: usize,
}

impl<'de, T> ResilientStreamDeserializer<'de, T>
where
    T: Deserialize<'de>,
{
    fn new(json: &'de str) -> Self {
        let stream = Deserializer::from_str(json).into_iter();
        let last_ok_pos = 0;

        ResilientStreamDeserializer {
            json,
            stream,
            last_ok_pos,
        }
    }
}

impl<'de, T> Iterator for ResilientStreamDeserializer<'de, T>
where
    T: Deserialize<'de>,
{
    type Item = Result<T, JsonError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.stream.next()? {
            Ok(value) => {
                self.last_ok_pos = self.stream.byte_offset();
                Some(Ok(value))
            }
            Err(error) => {
                // If an error happened, check whether it's a type error, i.e.
                // whether the next thing in the stream was at least valid JSON.
                // If so, return it as a dynamically-typed `Value` and skip it.
                let err_json = &self.json[self.last_ok_pos..];
                let mut err_stream = Deserializer::from_str(err_json).into_iter::<Value>();
                let value = err_stream.next()?.ok();
                let next_pos = if value.is_some() {
                    self.last_ok_pos + err_stream.byte_offset()
                } else {
                    self.json.len() // when JSON has a syntax error, prevent infinite stream of errors
                };
                self.json = &self.json[next_pos..];
                self.stream = Deserializer::from_str(self.json).into_iter();
                self.last_ok_pos = 0;
                Some(Err(JsonError { error, value }))
            }
        }
    }
}

fn main() {
    let feature_sequence = r#"{"id":"this_is_ok_1"}
    {"id":1, "abc": 2}
    {"id":"this_is_ok_2"}
    {"id": lol wrong JSON"#;

    for result in ResilientStreamDeserializer::<Final>::new(feature_sequence) {
        dbg!(result);
    }
}
