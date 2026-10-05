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
