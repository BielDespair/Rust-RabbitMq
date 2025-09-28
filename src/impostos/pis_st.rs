#![allow(non_snake_case)]
use rust_decimal::Decimal;
use serde::Serialize;


#[derive(Debug, Default, Serialize)]
pub struct PISST {
    pub calculo: CalculoPisSt,
    pub vPIS: Decimal,
    pub indSomaPISST: Option<bool>,
}


#[derive(Debug, Serialize)]
pub enum CalculoPisSt {
    Aliquota {
        vBC: Decimal,
        pPIS: Decimal,
    },
    Unidade {
        qBCProd: Decimal,
        vAliqProd: Decimal,
    },
}

impl Default for CalculoPisSt {
    fn default() -> Self {
        Self::Unidade { qBCProd: (Decimal::ZERO), vAliqProd: (Decimal::ZERO) }
    }
}