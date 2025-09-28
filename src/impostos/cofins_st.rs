#![allow(non_snake_case)]
use rust_decimal::Decimal;
use serde::Serialize;

#[derive(Debug, Default, Serialize)]
pub struct COFINSST {
    #[serde(flatten)]
    pub calculo: CalculoCofinsSt,

    pub vCOFINS: Decimal,

    pub indSomaCOFINSST: Option<u8>,
}

#[derive(Debug, Serialize)]
pub enum CalculoCofinsSt {
    /// Cálculo por valor (Base de Cálculo x Alíquota).
    Aliquota {
        vBC: Decimal,
        pCOFINS: Decimal,
    },
    /// Cálculo por quantidade (Quantidade x Valor por Unidade).
    Unidade {
        qBCProd: Decimal,
        vAliqProd: Decimal,
    },
}


impl Default for CalculoCofinsSt {
    fn default() -> Self {
        Self::Unidade { qBCProd: (Decimal::ZERO), vAliqProd: (Decimal::ZERO) }
    }
}