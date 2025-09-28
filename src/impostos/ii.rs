#![allow(non_snake_case)]
use rust_decimal::Decimal;
use serde::Serialize;

#[derive(Debug, Default, Serialize)]
pub struct Ii {
    pub vBC: Decimal,
    pub vDespAdu: Decimal,
    pub vII: Decimal,
    pub vIOF: Decimal,

}