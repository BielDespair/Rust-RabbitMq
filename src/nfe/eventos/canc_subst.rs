#![allow(non_snake_case, non_camel_case_types)]
use std::error::Error;

use quick_xml::{events::{BytesStart, Event}, Reader};
use serde::Serialize;

use crate::nfe::common::{read_text_string, ParseError}; // Supondo que você tenha estas helpers

// --- Estruturas de Envio ---

#[derive(Debug, Default, Serialize)]
pub struct TEnvEvento {
    pub versao: String,
    pub idLote: String,
    pub eventos: Vec<TEvento>,
}

#[derive(Debug, Default, Serialize)]
pub struct TEvento {
    pub versao: String,
    pub infEvento: InfEvento,
    pub signature: Option<Signature>, // Assinatura é complexa, aqui simplificada
}

#[derive(Debug, Default, Serialize)]
pub struct InfEvento {
    pub Id: String,
    pub cOrgao: String,
    pub tpAmb: String,
    pub autor: Option<Autor>,
    pub chNFe: String,
    pub dhEvento: String,
    pub tpEvento: String,
    pub nSeqEvento: String,
    pub verEvento: String,
    pub detEvento: DetEvento,
}

#[derive(Debug, Serialize)]
pub enum Autor {
    CNPJ(String),
    CPF(String),
}

#[derive(Debug, Default, Serialize)]
pub struct DetEvento {
    pub versao: String,
    pub descEvento: String,
    pub cOrgaoAutor: String,
    pub tpAutor: String,
    pub verAplic: String,
    pub nProt: String,
    pub xJust: String,
    pub chNFeRef: String,
}

// --- Estruturas de Retorno ---

#[derive(Debug, Default, Serialize)]
pub struct TRetEnvEvento {
    pub versao: String,
    pub idLote: String,
    pub tpAmb: String,
    pub verAplic: String,
    pub cOrgao: String,
    pub cStat: String,
    pub xMotivo: String,
    pub retEventos: Vec<TRetEvento>,
}

#[derive(Debug, Default, Serialize)]
pub struct TRetEvento {
    pub versao: String,
    pub infEvento: InfRetEvento,
    pub signature: Option<Signature>,
}

#[derive(Debug, Default, Serialize)]
pub struct InfRetEvento {
    pub Id: Option<String>,
    pub tpAmb: String,
    pub verAplic: String,
    pub cOrgao: String,
    pub cStat: String,
    pub xMotivo: String,
    pub chNFe: Option<String>,
    pub tpEvento: Option<String>,
    pub xEvento: Option<String>,
    pub nSeqEvento: Option<String>,
    pub dhRegEvento: String,
    pub nProt: Option<String>,
}

// Estrutura simplificada para a assinatura digital
#[derive(Debug, Default, Serialize)]
pub struct Signature {
    // A implementação completa seria muito mais detalhada
    pub placeholder: bool,
}