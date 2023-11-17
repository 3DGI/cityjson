use serde_json::{json, Value};



struct CityJSONBuilder {
    value: Value,
}

impl Default for CityJSONBuilder {
    fn default() -> Self {
        Self {
            value: json!({
                "type": "CityJSON",
                "version": "2.0",
                "transform": {
                    "scale": [1.0, 1.0, 1.0],
                    "translate": [0.0, 0.0, 0.0]
                },
                "CityObjects": {},
                "vertices": []
            })
        }
    }
}

impl CityJSONBuilder {
    fn new() -> Self {
        Self::default()
    }

    fn with_metadata(mut self, metadatabuilder: Option<MetadataBuilder>) -> Self {
        let mb = match metadatabuilder {
            None => MetadataBuilder::new(),
            Some(_mb) => _mb
        };
        self.value.as_object_mut().unwrap().insert("metadata".to_string(), mb.value);
        self
    }

    fn build_string(self) -> serde_json::Result<String> {
        serde_json::to_string(&self.value)
    }

    fn build_vec(self) -> serde_json::Result<Vec<u8>> {
        serde_json::to_vec(&self.value)
    }
}

struct MetadataBuilder {
    value: Value,
}

impl MetadataBuilder {
    fn new() -> Self {
        Self { value: Value::Object(Default::default()) }
    }

    fn with_geographical_extent(mut self) -> Self {
        self.value.as_object_mut().unwrap().insert("geographical_extent".to_string(), json!([0.0, 0.0, 0.0, 0.0, 0.0, 0.0]));
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mb = MetadataBuilder::new().with_geographical_extent();
        let cjb = CityJSONBuilder::new().with_metadata(Some(mb));
        dbg!(cjb.build_string());
    }
}
