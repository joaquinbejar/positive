/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 01/10/26
******************************************************************************/

//! `Positive` implements `utoipa::ToSchema`, so downstream crates can use it
//! in their OpenAPI schemas.

#![cfg(feature = "utoipa")]

use positive::Positive;
use utoipa::{PartialSchema, ToSchema};

#[test]
fn positive_schema_name() {
    assert_eq!(<Positive as ToSchema>::name(), "Positive");
}

#[test]
fn positive_schema_is_a_decimal_string() {
    let schema = serde_json::to_value(<Positive as PartialSchema>::schema())
        .expect("schema serialises to JSON");
    assert_eq!(schema["type"], "string");
}

#[derive(utoipa::ToSchema)]
#[allow(dead_code)]
struct Order {
    price: Positive,
}

#[test]
fn positive_composes_into_a_derived_schema() {
    let schema = serde_json::to_value(<Order as PartialSchema>::schema())
        .expect("schema serialises to JSON");
    assert_eq!(
        schema["properties"]["price"]["$ref"],
        "#/components/schemas/Positive"
    );
}

#[test]
fn strictly_positive_schema_name() {
    assert_eq!(
        <positive::StrictlyPositive as ToSchema>::name(),
        "StrictlyPositive"
    );
}

#[test]
fn strictly_positive_schema_is_a_decimal_string() {
    let schema = serde_json::to_value(<positive::StrictlyPositive as PartialSchema>::schema())
        .expect("schema serialises to JSON");
    assert_eq!(schema["type"], "string");
    let description = schema["description"].as_str().unwrap_or_default();
    assert!(description.contains("strictly greater than zero"));
}

#[derive(utoipa::ToSchema)]
#[allow(dead_code)]
struct Instrument {
    price: positive::StrictlyPositive,
    daily_volume: Positive,
}

#[test]
fn strictly_positive_composes_next_to_positive() {
    let schema = serde_json::to_value(<Instrument as PartialSchema>::schema())
        .expect("schema serialises to JSON");
    assert_eq!(
        schema["properties"]["price"]["$ref"],
        "#/components/schemas/StrictlyPositive"
    );
    assert_eq!(
        schema["properties"]["daily_volume"]["$ref"],
        "#/components/schemas/Positive"
    );
}
