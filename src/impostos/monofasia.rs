#![allow(non_snake_case, non_camel_case_types)]

use rust_decimal::Decimal;
use serde::Serialize;

#[derive(Debug, Default, Serialize)]
pub struct TMonofasia {
    pub gMonoPadrao: Option<GMonoPadrao>,
    pub gMonoReten: Option<GMonoReten>,
    pub gMonoRet: Option<GMonoRet>,
    pub gMonoDif: Option<GMonoDif>,
    pub vTotIBSMonoItem: Decimal,
    pub vTotCBSMonoItem: Decimal,
}

#[derive(Debug, Default, Serialize)]
pub struct GMonoPadrao {
    pub qBCMono: Decimal,
    pub adRemIBS: Decimal,
    pub adRemCBS: Decimal,
    pub vIBSMono: Decimal,
    pub vCBSMono: Decimal,
}

#[derive(Debug, Default, Serialize)]
pub struct GMonoReten {
    pub qBCMonoReten: Decimal,
    pub adRemIBSReten: Decimal,
    pub vIBSMonoReten: Decimal,
    pub adRemCBSReten: Decimal,
    pub vCBSMonoReten: Decimal,
}

#[derive(Debug, Default, Serialize)]
pub struct GMonoRet {
    pub qBCMonoRet: Decimal,
    pub adRemIBSRet: Decimal,
    pub vIBSMonoRet: Decimal,
    pub adRemCBSRet: Decimal,
    pub vCBSMonoRet: Decimal,
}

#[derive(Debug, Default, Serialize)]
pub struct GMonoDif {
    pub pDifIBS: Decimal,
    pub vIBSMonoDif: Decimal,
    pub pDifCBS: Decimal,
    pub vCBSMonoDif: Decimal,
}

