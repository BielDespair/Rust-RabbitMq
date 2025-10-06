#![allow(non_snake_case, non_camel_case_types)]

use std::error::Error;

use bytes::Bytes;
use quick_xml::{Reader, events::Event};
use serde::Serialize;

use crate::{
    nfe::common::{ParseError, XmlReader, get_tag_attribute, read_text},
    nfes::EmitenteId,
};

#[derive(Debug, Default, Serialize)]
pub struct EventoJson {
    pub company_id: i64,
    pub org_id: i64,
    pub eventos: Vec<Evento>,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum Evento {
    evento(TEvento),
    procEventoNFe(TProcEvento),
}

impl Default for Evento {
    fn default() -> Self {
        return Self::evento(TEvento::default());
    }
}

//
#[derive(Debug, Default, Serialize)]
pub struct TEvento {
    pub Id: String,

    #[serde(flatten)]
    pub infEvento: InfEvento,
}

// infEvento de TEvento
#[derive(Debug, Default, Serialize)]
pub struct InfEvento {
    pub cOrgao: String,
    pub tpAmb: String,

    #[serde(flatten)]
    pub CpfCnpj: EmitenteId,

    pub chNFe: String,
    pub dhEvento: String,
    pub tpEvento: String,
    pub nSeqEvento: String,
    pub verEvento: String,

    pub descEvento: String,
    pub cOrgaoAutor: String,
    pub tpAutor: String,
    pub verAplic: String,
    pub nProt: String,
    pub xJust: String,
    pub chNFeRef: String,
}

// infEvento de TRetEvento.
#[derive(Debug, Default, Serialize)]
pub struct TRetEvento {
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
    pub cOrgaoAutor: Option<String>,
    pub dhRegEvento: String,
    pub nProt: Option<String>,
}

#[derive(Debug, Default, Serialize)]
pub struct TProcEvento {
    pub evento: TEvento,
    pub retEvento: TRetEvento,
}

pub fn parse_evento_nfe(xml: Bytes) -> Result<EventoJson, Box<dyn Error>> {
    let mut evento_json: EventoJson = EventoJson::default();

    let mut reader: Reader<&[u8]> = Reader::from_reader(&xml);

    loop {
        match reader.read_event()? {
            Event::Start(e) => match e.name().as_ref() {
                b"evento" => {
                    let evento: TEvento = parse_evento(&mut reader)?;
                    evento_json.eventos.push(Evento::evento(evento));
                    return Ok(evento_json);
                }

                b"procEventoNFe" => {
                    let evento: TProcEvento = parse_procEventoNFe(&mut reader)?;
                    evento_json.eventos.push(Evento::procEventoNFe(evento));
                }

                tag => {
                    let tag = String::from_utf8_lossy(tag).to_string();
                    return Err(Box::new(ParseError::CampoDesconhecido(tag)));
                }
            },

            Event::Eof => {
                return Err(Box::new(ParseError::Xml(
                    "Nao foi possivel parsear Evento".to_string(),
                )));
            }

            _ => (),
        }
    }
}

fn parse_evento(reader: &mut XmlReader) -> Result<TEvento, Box<dyn Error>> {
    let mut evento: TEvento = TEvento::default();
    let infEvento = &mut evento.infEvento;

    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                match e.name().as_ref() {
                    //Ignora para não tentar ler complexType com read_text
                    b"infEvento" => evento.Id = get_tag_attribute(&e, b"Id")?,
                    b"detEvento" => (),
                    b"Signature" => (),

                    name => {
                        let txt: String = read_text(reader, &e)?;
                        match name {
                            b"cOrgao" => infEvento.cOrgao = txt,
                            b"tpAmb" => infEvento.tpAmb = txt,
                            b"CNPJ" => infEvento.CpfCnpj = EmitenteId::CNPJ(txt),
                            b"CPF" => infEvento.CpfCnpj = EmitenteId::CPF(txt),
                            b"chNFe" => infEvento.chNFe = txt,
                            b"dhEvento" => infEvento.dhEvento = txt,
                            b"tpEvento" => infEvento.tpEvento = txt,
                            b"nSeqEvento" => infEvento.nSeqEvento = txt,
                            b"verEvento" => infEvento.verEvento = txt,
                            b"descEvento" => infEvento.descEvento = txt,
                            b"cOrgaoAutor" => infEvento.cOrgaoAutor = txt,
                            b"tpAutor" => infEvento.tpAutor = txt,
                            b"verAplic" => infEvento.verAplic = txt,
                            b"nProt" => infEvento.nProt = txt,
                            b"xJust" => infEvento.xJust = txt,
                            b"chNFeRef" => infEvento.chNFeRef = txt,
                            _ => (),
                        }
                    }
                }
            }
            Event::End(e) if e.name().as_ref() == b"evento" => return Ok(evento),
            Event::Eof => return Err(Box::new(ParseError::UnexpectedEof("evento".to_string()))),

            _ => (),
        }
    }
}

fn parse_procEventoNFe(reader: &mut XmlReader) -> Result<TProcEvento, Box<dyn Error>> {
    let mut proc_evento: TProcEvento = TProcEvento::default();

    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                match e.name().as_ref() {
                    b"evento" => proc_evento.evento = parse_evento(reader)?,
                    b"retEvento" => proc_evento.retEvento = parse_retEvento(reader)?,
                    _ => (),
                }
            }
            
            Event::End(e) if e.name().as_ref() == b"procEventoNFe" => return Ok(proc_evento),
            Event::Eof => return Err(Box::new(ParseError::UnexpectedEof("procEventoNFe".to_string()))),
            _ => (),
        }
    }
}

fn parse_retEvento(reader: &mut XmlReader) -> Result<TRetEvento, Box<dyn Error>> {
    let mut ret: TRetEvento = TRetEvento::default();

    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                match e.name().as_ref() {
                    //Ignora para não tentar ler complexType com read_text
                    b"infEvento" => {
                        if let Ok(id) = get_tag_attribute(&e, b"Id") {
                            ret.Id = Some(id);
                        }
                    }
                    b"Signature" => (),

                    name => {
                        let txt: String = read_text(reader, &e)?;
                        match name {
                            b"tpAmb" => ret.tpAmb = txt,
                            b"verAplic" => ret.verAplic = txt,
                            b"cOrgao" => ret.cOrgao = txt,
                            b"cStat" => ret.cStat = txt,
                            b"xMotivo" => ret.xMotivo = txt,
                            b"chNFe" => ret.chNFe = Some(txt),
                            b"tpEvento" => ret.tpEvento = Some(txt),
                            b"xEvento" => ret.xEvento = Some(txt),
                            b"nSeqEvento" => ret.nSeqEvento = Some(txt),
                            b"cOrgaoAutor" => ret.cOrgaoAutor = Some(txt),
                            b"dhRegEvento" => ret.dhRegEvento = txt,
                            b"nProt" => ret.nProt = Some(txt),
                            _ => (),
                        }
                    }
                }
            }
            Event::End(e) if e.name().as_ref() == b"infEvento" => return Ok(ret),
            Event::Eof => return Err(Box::new(ParseError::UnexpectedEof("infEvento".to_string()))),

            _ => (),
        }
    }
}
