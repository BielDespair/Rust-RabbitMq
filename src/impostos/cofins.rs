#![allow(non_snake_case)]
use rust_decimal::Decimal;
use serde::Serialize;

/// Estrutura principal que representa a tag <COFINS>
#[derive(Debug, Default, Serialize)]
pub struct COFINS {
    #[serde(flatten)]
    pub tributacao: Tributacao,
}

/// Representa a escolha principal do tipo de COFINS
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum Tributacao {
    COFINSAliq(COFINSAliq),
    COFINSQtde(COFINSQtde),
    COFINSNT{CST: String},
    COFINSOutr(COFINSOutr),
}

impl Default for Tributacao {
    fn default() -> Self {
        Self::COFINSNT { CST: (String::new()) }
    }
}


// --- Definições para cada tipo de COFINS ---

/// COFINS Tributado pela Alíquota (CST 01, 02)
#[derive(Debug, Default, Serialize)]
pub struct COFINSAliq {
    pub CST: String,
    pub vBC: Decimal,
    pub pCOFINS: Decimal,
    pub vCOFINS: Decimal,
}

/// COFINS Tributado por Quantidade (CST 03)
#[derive(Debug, Default, Serialize)]
pub struct COFINSQtde {
    pub CST: String,
    pub qBCProd: Decimal,
    pub vAliqProd: Decimal,
    pub vCOFINS: Decimal,
}

/// COFINS Outras Operações (CST 49 a 99)
#[derive(Debug, Default, Serialize)]
pub struct COFINSOutr {
    pub CST: String,
    #[serde(flatten)]
    pub calculo: CalculoCOFINSOutr,
    pub vCOFINS: Decimal,
}

/// Enum para a escolha de cálculo DENTRO de COFINSOutr
#[derive(Debug, Serialize)]
pub enum CalculoCOFINSOutr {
    Aliquota {
        vBC: Decimal,
        pCOFINS: Decimal,
    },
    Unidade {
        qBCProd: Decimal,
        vAliqProd: Decimal,
    },
}

impl Default for CalculoCOFINSOutr {
    fn default() -> Self {
        Self::Unidade { qBCProd: Decimal::ZERO, vAliqProd: Decimal::ZERO }
    }
}