#![allow(non_snake_case)]
use rust_decimal::Decimal;
use serde::Serialize;

#[derive(Debug, Default, Serialize)]
pub struct ISSQN {
    pub vBC: Decimal,
    pub vAliq: Decimal,
    pub vISSQN: Decimal,
    pub cMunFG: u32,
    pub cListServ: String,
    pub vDeducao: Option<Decimal>,
    pub vOutro: Option<Decimal>,
    pub vDescIncond: Option<Decimal>,
    pub vDescCond: Option<Decimal>,
    pub vISSRet: Option<Decimal>,
    pub indISS: u8,
    pub cServico: Option<String>,
    pub cMun: Option<u32>,
    pub cPais: Option<String>,
    pub nProcesso: Option<String>,
    pub indIncentivo: u8,
}