#![allow(non_snake_case)]
use rust_decimal::Decimal;
use serde::Serialize;

#[derive(Debug, Default, Serialize)]
pub struct PIS {
    
    #[serde(flatten)]
    pub tributacao: Tributacao,
}

#[derive(Debug, Serialize)]
pub enum Tributacao {
    PISAliq(PISAliq),
    PISQtde(PISQtde),
    PISNT {CST: String},
    PISOutr(PISOutr)
}

impl Default for Tributacao {
    fn default() -> Self {
        Self::PISNT { CST: (String::new()) }
    }
}


#[derive(Debug, Default, Serialize)]
pub struct PISAliq {
    pub CST: String,
    pub vBC: Decimal,
    pub pPIS: Decimal,
    pub vPIS: Decimal,
}

#[derive(Debug, Default, Serialize)]
pub struct PISQtde {
    pub CST: String,
    pub qBCProd: Decimal,
    pub vAliqProd: Decimal,
    pub vPIS: Decimal,
}

#[derive(Debug, Default, Serialize)]
pub struct PISOutr {
    pub CST: String,
    pub calculo: CalculoPISOutr,
    pub vPIS: Decimal
}

/// Enum para a escolha de cálculo DENTRO de PISOutr
#[derive(Debug, Serialize)]
pub enum CalculoPISOutr {
    Aliquota {
        vBC: Decimal,
        pPIS: Decimal,
    },
    Unidade {
        qBCProd: Decimal,
        vAliqProd: Decimal,
    },
}

impl Default for CalculoPISOutr {
    fn default() -> Self {
        Self::Unidade { qBCProd: (Decimal::ZERO), vAliqProd: (Decimal::ZERO) }
    }
}