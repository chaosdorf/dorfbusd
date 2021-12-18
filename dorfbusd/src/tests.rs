use core::panic;
use std::collections::BTreeMap;

use okapi::openapi3::OpenApi;
use schemars::{
    gen::{SchemaGenerator, SchemaSettings},
    schema::{Schema, SingleOrVec},
    JsonSchema,
};
use serde_json::Value;

use pretty_assertions::assert_eq;

use crate::{
    api::ApiErrorResponse,
    bus_state::{BusState, CoilState, CoilValue, DeviceState},
    config::{CoilConfig, Config, DeviceConfig, ResetCoilStatus},
};

fn cleanup_schemar(obj: &mut schemars::schema::SchemaObject) {
    if let Some(example) = obj.extensions.get("example") {
        let mut metadata = obj.metadata.clone().unwrap_or_default();
        metadata.examples = vec![example.clone()];

        obj.metadata = Some(metadata);
    }
    obj.extensions.remove("example");

    if let Some(metadata) = &mut obj.metadata {
        if obj.enum_values.is_some() {
            metadata.default = None;
        }
    }

    if let Some(instance_type) = obj.instance_type.clone() {
        obj.instance_type = match instance_type {
            SingleOrVec::Vec(v) => v
                .into_iter()
                .next()
                .map(|instance_type| SingleOrVec::Single(Box::new(instance_type))),
            single => Some(single),
        }
    }

    if let Some(number) = &mut obj.number {
        if number.minimum == Some(0.0) {
            number.minimum = None;
        }
    }
    if let Some(array) = &mut obj.array {
        array.unique_items = None;
    }
    if let Some(obj) = &mut obj.object {
        for (_, schema) in obj.properties.iter_mut() {
            if let Schema::Object(obj) = schema {
                cleanup_schemar(obj);
            }
        }
    }
}

fn schmea_gen() -> SchemaGenerator {
    let mut schema_settings = SchemaSettings::default();
    schema_settings.definitions_path = "#/components/schemas/".to_string();

    SchemaGenerator::new(schema_settings)
}

#[test]
fn check_schemas() {
    let spec: OpenApi =
        serde_yaml::from_str(include_str!("./openapi.yml")).expect("could not parse schema");

    let spec_schemas: BTreeMap<String, Value> = spec
        .components
        .expect("components not found in schema")
        .schemas
        .iter_mut()
        .map(|(name, schema)| {
            cleanup_schemar(schema);

            let schema = serde_json::to_value(schema)
                .expect(&format!("could not convert {} to json value", name));
            (name.clone(), schema)
        })
        .collect();

    let mut schema_settings = SchemaSettings::default();
    schema_settings.definitions_path = "#/components/schemas/".to_string();

    let mut gen = schmea_gen();

    let ignored_schemas = vec!["CoilUpdate"];

    let derive_schemas: BTreeMap<String, Value> = vec![
        (BusState::schema_name(), BusState::json_schema(&mut gen)),
        (Config::schema_name(), Config::json_schema(&mut gen)),
        (CoilConfig::schema_name(), CoilConfig::json_schema(&mut gen)),
        (
            DeviceConfig::schema_name(),
            DeviceConfig::json_schema(&mut gen),
        ),
        (CoilValue::schema_name(), CoilValue::json_schema(&mut gen)),
        (
            ResetCoilStatus::schema_name(),
            ResetCoilStatus::json_schema(&mut gen),
        ),
        (CoilState::schema_name(), CoilState::json_schema(&mut gen)),
        (
            DeviceState::schema_name(),
            DeviceState::json_schema(&mut gen),
        ),
        (
            ApiErrorResponse::schema_name(),
            ApiErrorResponse::json_schema(&mut gen),
        ),
    ]
    .into_iter()
    .map(|(name, mut schema)| {
        if let Schema::Object(obj) = &mut schema {
            cleanup_schemar(obj);
        }

        let schema = serde_json::to_value(schema)
            .expect(&format!("could not convert {} to json value", name));
        (name.clone(), schema)
    })
    .collect();

    for (name, spec) in spec_schemas {
        if let Some(derived) = derive_schemas.get(&name) {
            assert_eq!(&spec, derived, "{}", name);
        } else {
            if !ignored_schemas.contains(&name.as_str()) {
                panic!("did not find {} in derived schemas", name);
            }
        }
    }
}
