#![allow(non_snake_case)]
use rust_decimal::Decimal;
use serde::Serialize;

use crate::nfes::UF;


#[derive(Debug, Default, Serialize)]
pub struct Icms {
    pub tipo: TipoICMS,

    // --- CAMPOS DE IDENTIFICAÇÃO ---
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orig: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub CST: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub CSOSN: Option<String>,

    // --- CÁLCULO ICMS NORMAL (CST 00, 10, 20, 51, 70, 90, Part) ---
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modBC: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vBC: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pRedBC: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pICMS: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vICMS: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vICMSOp: Option<Decimal>, // Específico do ICMS51

    // --- FCP (Fundo de Combate à Pobreza) ---
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vBCFCP: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pFCP: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vFCP: Option<Decimal>,

    // --- ICMS ST (CÁLCULO NA OPERAÇÃO) ---
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modBCST: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pMVAST: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pRedBCST: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vBCST: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pICMSST: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vICMSST: Option<Decimal>,
    
    // --- FCP ST ---
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vBCFCPST: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pFCPST: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vFCPST: Option<Decimal>,

    // --- ICMS ST RETIDO (OPERAÇÃO ANTERIOR - CST 60, CSOSN 500) ---
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vBCSTRet: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pST: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vICMSSubstituto: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vICMSSTRet: Option<Decimal>,
    
    // --- FCP ST RETIDO ---
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vBCFCPSTRet: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pFCPSTRet: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vFCPSTRet: Option<Decimal>,

    // --- ICMS DESONERADO ---
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vICMSDeson: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub motDesICMS: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub indDeduzDeson: Option<bool>,
    
    // --- ICMS ST DESONERADO ---
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vICMSSTDeson: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub motDesICMSST: Option<String>,

    // --- ICMS EFETIVO ---
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pRedBCEfet: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vBCEfet: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pICMSEfet: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vICMSEfet: Option<Decimal>,

    // --- ICMS DIFERIMENTO (ICMS51) ---
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pDif: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vICMSDif: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cBenefRBC: Option<String>, // Também do ICMS51
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pFCPDif: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vFCPDif: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vFCPEfet: Option<Decimal>,

    // --- ICMS MONOFÁSICO (CST 02, 15, 53, 61) ---
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qBCMono: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adRemICMS: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vICMSMono: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qBCMonoReten: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adRemICMSReten: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vICMSMonoReten: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pRedAdRem: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub motRedAdRem: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qBCMonoRet: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adRemICMSRet: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vICMSMonoRet: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vICMSMonoOp: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vICMSMonoDif: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qBCMonoDif: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adRemICMSDif: Option<Decimal>,

    // --- ICMS PARTILHA ---
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pBCOp: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub UFST: Option<UF>,

    // --- ICMS ST (REPASSE) ---
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vBCSTDest: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vICMSSTDest: Option<Decimal>,

    // --- SIMPLES NACIONAL (CRÉDITO) ---
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pCredSN: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vCredICMSSN: Option<Decimal>,
}

#[derive(Debug, Serialize)]
pub enum TipoICMS {
    ICMS00,
    ICMS02,
    ICMS10,
    ICMS15,
    ICMS20,
    ICMS30,
    ICMS40,
    ICMS51,
    ICMS53,
    ICMS60,
    ICMS61,
    ICMS70,
    ICMS90,
    ICMSPART,
    ICMSST,
    ICMSSN101,
    ICMSSN102,
    ICMSSN201,
    ICMSSN202,
    ICMSSN500,
    ICMSSN900,
}

impl Default for TipoICMS {
    fn default() -> Self {
        return Self::ICMS00
    }
}