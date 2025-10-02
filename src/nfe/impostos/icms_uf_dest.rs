#![allow(non_snake_case)]
use rust_decimal::Decimal;
use serde::Serialize;


/// Representa o grupo de Partilha do ICMS entre a UF de origem e destino (<ICMSUFDest>).
#[derive(Debug, Default, Serialize)]
pub struct ICMSUFDest {
    pub vBCUFDest: Decimal,
    
    pub vBCFCPUFDest: Option<Decimal>,
    pub pFCPUFDest: Option<Decimal>,
    
    pub pICMSUFDest: Decimal,
    pub pICMSInter: String,
    pub pICMSInterPart: Decimal,

    pub vFCPUFDest: Option<Decimal>,

    pub vICMSUFDest: Decimal,
    pub vICMSUFRemet: Decimal,
}