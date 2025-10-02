#![allow(non_snake_case, non_camel_case_types)]

use rust_decimal::Decimal;
use serde::Serialize;

/// Contém o valor do IPI devolvido.
#[derive(Debug, Default, Serialize)]
pub struct IpiDevol {
    /// Valor do IPI devolvido
    pub vIPIDevol: Decimal,
}

/// Grupo de informações sobre a devolução de tributos.
#[derive(Debug, Default, Serialize)]
pub struct ImpostoDevol {
    /// Percentual de mercadoria devolvida
    pub pDevol: Decimal,

    /// Informação de IPI devolvido
    pub IPI: IpiDevol,
}