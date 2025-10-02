#![allow(non_snake_case, non_camel_case_types)]

use rust_decimal::Decimal;
use serde::Serialize;

#[derive(Debug, Default, Serialize)]
pub struct Cobr {
    pub fat: Option<Fat>,
    pub dup: Option<Vec<Dup>>,
}

#[derive(Debug, Default, Serialize)]
pub struct Fat {
    pub nFat: Option<String>,
    pub vOrig: Option<Decimal>,
    pub vDesc: Option<Decimal>,
    pub vLiq: Option<Decimal>,
}

#[derive(Debug, Default, Serialize)]
pub struct Dup {
    pub nDup: Option<String>,
    pub dVenc: Option<String>,
    pub vDup: Decimal,
}
