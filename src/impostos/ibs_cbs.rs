 #![allow(non_snake_case, non_camel_case_types)]
use rust_decimal::Decimal;
use serde::Serialize;

use crate::impostos::{cibs::TCIBS, monofasia::TMonofasia};


/// Representa o grupo de Partilha do ICMS entre a UF de origem e destino (<ICMSUFDest>).
#[derive(Debug, Default, Serialize)]
pub struct IBSCBS {
    pub CST: String,
    pub cClassTrib: String,
    #[serde(flatten)]
    pub tributacao: Option<TributacaoIBS>,
    pub gCredPresIBSZFM: Option<TCredPresIBSZFM>

}

#[derive(Debug, Serialize)]
pub enum TributacaoIBS {
    gIBSCBS(TCIBS),
    gIBSCBSMono(TMonofasia),
    gTransfCred(TTransfCred),
}
impl Default for TributacaoIBS {
    fn default() -> Self {
        return Self::gTransfCred(TTransfCred::default());
    }
}

#[derive(Debug, Default, Serialize)]
pub struct TTransfCred {
    pub vIBS: Decimal,
    pub vCBS: Decimal,
}

#[derive(Debug, Default, Serialize)]
pub struct TCredPresIBSZFM {
    pub tpCredPresIBSZFM: String,
    pub vCredPresIBSZFM: Option<Decimal>,
}