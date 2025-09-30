#![allow(non_snake_case, non_camel_case_types)]

use rust_decimal::Decimal;
use serde::Serialize;


#[derive(Debug, Default, Serialize)]
pub struct TCIBS {
    pub vBC: Decimal,
    pub gIBSUF: GIBSUF,
    pub gIBSMun: GIBSMun,
    pub vIBS: Decimal,
    pub gCBS: GCBS,
    pub gTribRegular: Option<TTribRegular>,
    pub gIBSCredPres: Option<TCredPres>,
    pub gCBSCredPres: Option<TCredPres>,
    pub gTribCompraGov: Option<TTribCompraGov>,
}

#[derive(Debug, Default, Serialize)]
pub struct TDif {
    pub pDif: Decimal,
    pub vDif: Decimal,
}

#[derive(Debug, Default, Serialize)]
pub struct TDevTrib {
    pub vDevTrib: Decimal,
}

#[derive(Debug, Default, Serialize)]
pub struct TRed {
    pub pRedAliq: Decimal,
    pub pAliqEfet: Decimal,
}

#[derive(Debug, Default, Serialize)]
pub struct TCredPres {
    pub cCredPres: String,
    pub pCredPres: Decimal,
    #[serde(flatten)]
    pub valor: ValorCredPres,
}

#[derive(Debug, Serialize)]
pub enum ValorCredPres {
    vCredPres(Decimal),
    vCredPresCondSus(Decimal),
}

impl Default for ValorCredPres {
    fn default() -> Self {
        Self::vCredPres(Decimal::ZERO)
    }
}

#[derive(Debug, Default, Serialize)]
pub struct TTribRegular {
    pub CSTReg: String,
    pub cClassTribReg: String,
    pub pAliqEfetRegIBSUF: Decimal,
    pub vTribRegIBSUF: Decimal,
    pub pAliqEfetRegIBSMun: Decimal,
    pub vTribRegIBSMun: Decimal,
    pub pAliqEfetRegCBS: Decimal,
    pub vTribRegCBS: Decimal,
}

#[derive(Debug, Default, Serialize)]
pub struct TTribCompraGov {
    pub pAliqIBSUF: Decimal,
    pub vTribIBSUF: Decimal,
    pub pAliqIBSMun: Decimal,
    pub vTribIBSMun: Decimal,
    pub pAliqCBS: Decimal,
    pub vTribCBS: Decimal,
}

#[derive(Debug, Default, Serialize)]
pub struct GIBSUF {
    pub pIBSUF: Decimal,
    pub gDif: Option<TDif>,
    pub gDevTrib: Option<TDevTrib>,
    pub gRed: Option<TRed>,
    pub vIBSUF: Decimal,
}

#[allow(non_snake_case)]
#[derive(Debug, Default, Serialize)]
pub struct GIBSMun {
    pub pIBSMun: Decimal,
    pub gDif: Option<TDif>,
    pub gDevTrib: Option<TDevTrib>,
    pub gRed: Option<TRed>,
    pub vIBSMun: Decimal,
}

#[allow(non_snake_case)]
#[derive(Debug, Default, Serialize)]
pub struct GCBS {
    pub pCBS: Decimal,
    pub gDif: Option<TDif>,
    pub gDevTrib: Option<TDevTrib>,
    pub gRed: Option<TRed>,
    pub vCBS: Decimal,
}