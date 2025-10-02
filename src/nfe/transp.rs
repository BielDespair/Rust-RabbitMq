#![allow(non_snake_case, non_camel_case_types)]

use rust_decimal::Decimal;
use serde::Serialize;

use crate::nfes::{EmitenteId, UF};



#[derive(Debug, Default, Serialize)]
pub struct Transp {
    /// Modalidade do frete.
    pub modFrete: Decimal,
    pub transporta: Option<Transporta>,
    pub retTransp: Option<RetTransp>,
    #[serde(flatten)]
    pub veiculo: Option<VeiculoTransporte>,
    pub vol: Option<Vec<Vol>>,
}


#[derive(Debug, Default, Serialize)]
pub struct TVeiculo {
    pub placa: String,
    pub UF: Option<UF>,
    pub RNTC: Option<String>,
}

#[derive(Debug, Default, Serialize)]
pub struct Transporta {
    #[serde(flatten)]
    pub identificacao: Option<EmitenteId>,
    pub xNome: Option<String>,
    pub IE: Option<String>,
    pub xEnder: Option<String>,
    pub xMun: Option<String>,
    pub UF: Option<UF>,
}

#[derive(Debug, Default, Serialize)]
pub struct RetTransp {
    pub vServ: Decimal,
    pub vBCRet: Decimal,
    pub pICMSRet: Decimal,
    pub vICMSRet: Decimal,
    pub CFOP: String,
    pub cMunFG: u32,
}


#[derive(Debug, Default, Serialize)]
pub struct TransporteRodoviario {
    pub veicTransp: Option<TVeiculo>,
    pub reboque: Option<Vec<TVeiculo>>,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum VeiculoTransporte {
    Rodoviario(TransporteRodoviario), 
    Vagao {vagao: String},
    Balsa {balsa: String}
}

#[derive(Debug, Default, Serialize)]
pub struct Lacre {
    pub nLacre: String,
}


#[derive(Debug, Default, Serialize)]
pub struct Vol {
    pub qVol: Option<String>,
    pub esp: Option<String>,
    pub marca: Option<String>,
    pub nVol: Option<String>,
    pub pesoL: Option<Decimal>,
    pub pesoB: Option<Decimal>,
    pub lacres: Option<Vec<Lacre>>,
}
