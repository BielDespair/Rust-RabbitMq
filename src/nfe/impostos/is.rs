#![allow(non_snake_case)]
use rust_decimal::Decimal;
use serde::Serialize;

#[derive(Debug, Default, Serialize)]
pub struct IS {
    pub CSTIS: String,
    pub cClassTribIS: String,
    #[serde(flatten)]
    pub calculo: Option<CalculoIS>,
}

/// Agrupa os campos de cálculo do Imposto Seletivo
#[derive(Debug, Default, Serialize)]
pub struct CalculoIS {
    /// Valor da Base de Cálculo do Imposto Seletivo
    pub vBCIS: Decimal,

    /// Alíquota do Imposto Seletivo (percentual)
    pub pIS: Decimal,

    /// Alíquota do Imposto Seletivo (por valor específico)
    pub pISEspec: Option<Decimal>,
    
    /// Grupo opcional para tributação por unidade de medida
    #[serde(flatten)]
    pub unidade_tributavel: Option<UnidadeTributavel>,

    /// Valor do Imposto Seletivo calculado
    pub vIS: Decimal,
}


/// Unidade de medida para apuração do Imposto Seletivo
#[derive(Debug, Default, Serialize)]
pub struct UnidadeTributavel {
    /// Unidade de medida especificada em Lei
    pub uTrib: String,

    /// Quantidade na unidade de medida informada em uTrib
    pub qTrib: Decimal,
}