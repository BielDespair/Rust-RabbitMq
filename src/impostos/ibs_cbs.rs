 /*
#![allow(non_snake_case, non_camel_case_types)]
use rust_decimal::Decimal;
use serde::Serialize;


/// Representa o grupo de Partilha do ICMS entre a UF de origem e destino (<ICMSUFDest>).
#[derive(Debug, Default, Serialize)]
pub struct IBSCBS {
    pub CST: String,
    pub cClassTrib: String,
    #[serde(flatten)]
    pub tributacao: Option<TributacaoIBS>,

}

#[derive(Debug, Serialize)]
pub enum TributacaoIBS {
    gIBSCBS(TCIBS),

    /// Tributação Monofásica.
    #[serde(rename = "gIBSCBSMono")]
    Monofasia(TMonofasia),

    /// Transferência de Crédito.
    #[serde(rename = "gTransfCred")]
    TransferenciaCredito(TTransfCred),
}
impl Default for TributacaoIBS {
    fn default() -> Self {
        todo!()
    }
}


#[derive(Debug, Default, Serialize)]
pub struct TCIBS {
    pub vBC: Decimal,
    
}
     */