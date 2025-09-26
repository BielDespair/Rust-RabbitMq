#![allow(non_snake_case, non_camel_case_types)]
use rust_decimal::Decimal;
use serde::{Serialize};




#[derive(Debug, Default, Serialize)]
pub struct NfeJson {
    pub company_id: u128,
    pub org_id: u128, 
    pub nfes: Vec<NFe>,
}
#[derive(Debug, Default, Serialize)]
pub struct NFe {
    pub ide: Ide,
    pub emit: Emit,
    pub avulsa: Option<Avulsa>,
    pub dest: Option<Dest>,
    pub retirada: Option<Local>,
    pub entrega: Option<Local>, 
    pub autXML: Option<Vec<EmitenteId>>,
    pub produtos: Vec<Prod>,

}

#[derive(Debug, Default, Serialize)]
pub struct Prod {
    pub nItem: u32,
    pub cProd: String,
    pub cEAN: Option<String>,
    pub xProd: String,
    pub NCM: String,
    pub CFOP: String,
    pub uCom: String,
    pub qCom: f64,
    pub vUnCom: f64,
    pub vProd: f64,
    pub cEANTrib: Option<String>,
    pub uTrib: String,
    pub qTrib: f64,
    pub vUnTrib: f64,
    pub indTot: u8,
    // More fields can be added as needed
}

#[derive(Debug, Default, Serialize)]
pub struct Ide {
    pub cUF: u8,
    pub cNF: String,
    pub natOp: String,
    pub r#mod: u8,
    pub serie: u16,
    pub nNF: u32,
    pub dhEmi: String,
    pub dhSaiEnt: Option<String>,
    pub tpNF: bool,
    pub idDest: u8,
    pub cMunFG: u32,
    pub cMunFGIBS: Option<u32>,
    pub tpImp: u8,
    pub tpEmis: u8,
    pub cDV: u8,
    pub tpAmb: u8,
    pub finNFe: u8,
    pub tpNFDebito: Option<u8>,
    pub tpNFCredito: Option<u8>,
    pub indFinal: bool,
    pub indPres: u8,
    pub indIntermed: Option<bool>,
    pub procEmi: u8,
    pub verProc: String,
    pub dhCont: Option<String>,
    pub xJust: Option<String>,
    pub NFref: Option<Vec<NFRef>>,
    pub gCompraGov: Option<CompraGov>,
    pub gPagAntecipado: Option<Vec<String>>,
}


#[derive(Debug, Default, Serialize)]
pub struct Emit {
    #[serde(flatten)]
    pub EmitenteId: EmitenteId,
    pub xNome: String,
    pub xFant: Option<String>,
    pub enderEmit: EnderEmi,
    pub IE: String,
    pub IEST: Option<String>,
    pub IM: Option<String>,
    pub CNAE: Option<String>,
    pub CRT: u8,


}

#[derive(Debug, Default, Serialize)]
pub struct Avulsa {
    pub CNPJ: Option<String>,
    pub xOrgao: String,
    pub matr: String,
    pub xAgente: String,
    pub fone: Option<String>,
    pub UF: UF,
    pub nDAR: Option<String>,
    pub dEmi: Option<String>,
    pub vDAR: Option<Decimal>,
    pub repEmi: String,
    pub dPag: Option<String>,
}

#[derive(Debug, Default, Serialize)]
pub struct Dest {
    #[serde(flatten)]
    pub EmitenteId: EmitenteId,

    pub xNome: String,
    pub enderDest: EnderEmi,
    pub IE: Option<String>,
    pub ISUF: Option<String>,
    pub IM: Option<String>,
    pub email: Option<String>,

}


#[derive(Debug, Default, Serialize)]
pub struct EnderEmi {
    pub xLgr: String,
    pub nro: String,
    pub xCpl: Option<String>,
    pub xBairro: String,
    pub cMun: u32,
    pub xMun: String,
    pub UF: UF,
    pub CEP: String,
    pub cPais: Option<String>,
    pub xPais: Option<String>,
    pub fone: Option<String>
}

#[derive(Debug, Default, Serialize)]
pub struct Local {
    #[serde(flatten)]
    pub EmitenteId: EmitenteId,

    pub xNome: Option<String>,
    pub xLgr: String,
    pub nro: String,
    pub xCpl: Option<String>,
    pub xBairro: String,
    pub cMun: u32,
    pub xMun: String,
    pub UF: UF,
    pub CEP: Option<String>,
    pub cPais: Option<String>,
    pub xPais: Option<String>,
    pub fone: Option<String>,
    pub email: Option<String>,
    pub IE: Option<String>,
}

#[derive(Debug, Default, Serialize)]
pub struct CompraGov {
    pub tpEnteGov: u8,
    pub pRedutor: Decimal,
    pub tpOperGov: u8,
}

#[derive(Debug, Serialize)]
pub enum NFRef {
    refNFe (String),
    refNFeSig (String),
    refNF (RefNFData),
    refNFP (RefNFPData),
    refCTe (String),
    refECF (RefECFData)
}

impl Default for NFRef {
    fn default() -> Self {
        NFRef::refNFe(String::new())
    }
}

#[derive(Debug, Default, Serialize)]
pub struct RefNFData {
    pub cUF: u8,
    pub AAMM: String,
    pub CNPJ: String,
    pub r#mod: u8,
    pub serie: u16,
    pub nNF: u32
}

#[derive(Debug, Default, Serialize)]
pub struct RefNFPData {
    pub cUF: u8,
    pub AAMM: String,
    #[serde(flatten)]
    pub EmitenteId: EmitenteId,
    pub IE: String,
    pub r#mod: u8,
    pub serie: u16,
    pub nNF: u32
}

#[derive(Debug, Default, Serialize)]
pub struct RefECFData {
    pub r#mod: String,
    pub nECF: String,
    pub nCOO: String
}


#[derive(Debug, Serialize)]
pub enum UF {
    AC, AL, AM, AP, BA, CE, DF, ES, GO, MA,
    MG, MS, MT, PA, PB, PE, PI, PR, RJ, RN,
    RO, RR, RS, SC, SE, SP, TO,
}

impl Default for UF {
    fn default() -> Self {
        UF::MG
    }
    
}

impl From<&str> for UF {
    fn from(s: &str) -> Self {
        match s {
            "AC" => UF::AC,
            "AL" => UF::AL,
            "AM" => UF::AM,
            "AP" => UF::AP,
            "BA" => UF::BA,
            "CE" => UF::CE,
            "DF" => UF::DF,
            "ES" => UF::ES,
            "GO" => UF::GO,
            "MA" => UF::MA,
            "MG" => UF::MG,
            "MS" => UF::MS,
            "MT" => UF::MT,
            "PA" => UF::PA,
            "PB" => UF::PB,
            "PE" => UF::PE,
            "PI" => UF::PI,
            "PR" => UF::PR,
            "RJ" => UF::RJ,
            "RN" => UF::RN,
            "RO" => UF::RO,
            "RR" => UF::RR,
            "RS" => UF::RS,
            "SC" => UF::SC,
            "SE" => UF::SE,
            "SP" => UF::SP,
            "TO" => UF::TO,
            _ => UF::default(),
        }
    }
}

#[derive(Debug, Serialize)]
pub enum EmitenteId {
    CNPJ(String),
    CPF (String),
    idEstrangeiro(String),
}

impl Default for EmitenteId {
    fn default() -> Self {
        EmitenteId::CNPJ(String::new())
    }
}