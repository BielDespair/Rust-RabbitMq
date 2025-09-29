#![allow(non_snake_case)]

use rust_decimal::Decimal;
use serde::Serialize;

use crate::impostos::issqn::ISSQN;

#[derive(Debug, Default, Serialize)]
pub struct TributosServico {
    pub IPI: Option<Ipi>,
    pub ISSQN: ISSQN,
}

// O struct principal que representa a tag <IPI>
#[derive(Debug, Default, Serialize)]
pub struct Ipi {
    pub CNPJProd: Option<String>,
    pub cSelo: Option<String>,
    pub qSelo: Option<String>,
    pub cEnq: String,

    pub Tributacao: Tributacao,
}


#[derive(Debug, Serialize)]
pub enum Tributacao {
    IPITrib(IPITrib),
    IPINT {CST: String}

}

impl Default for Tributacao {
    fn default() -> Self {
        return Tributacao::IPINT { CST: (String::new()) }
    }
}


#[derive(Debug, Default, Serialize)]
pub struct IPITrib {
    pub CST: String,
    #[serde(flatten)]
    pub calculo: CalculoIpi,
    pub vIPI: Decimal,
}

#[derive(Debug, Serialize)]
pub enum CalculoIpi  {
    Aliquota  {
        vBC: Decimal,
        pIPI: Decimal
    },
    Unidade  {
        qUnid: Decimal,
        vUnid: Decimal
    }
}

impl Default for CalculoIpi {
    fn default() -> Self {
        return CalculoIpi::Aliquota { vBC: ((Decimal::ZERO)), pIPI: (Decimal::ZERO) }
    }
}