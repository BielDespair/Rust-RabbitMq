#![allow(non_snake_case, non_camel_case_types)]

use std::error::Error;

use quick_xml::events::{BytesStart, Event};
use serde::Serialize;

use crate::nfe::common::{read_text_string, ParseError, XmlReader};

#[derive(Debug, Default, Serialize)]
pub struct InfAdic {
    pub infAdFisco: Option<String>,
    pub infCpl: Option<String>,
    pub obsCont: Option<Vec<ObsCont>>,
    pub obsFisco: Option<Vec<ObsFisco>>,
    pub procRef: Option<Vec<ProcRef>>,
}

#[derive(Debug, Default, Serialize)]
pub struct ObsCont {
    pub xCampo: String,
    pub xTexto: String,
}

#[derive(Debug, Default, Serialize)]
pub struct ObsFisco {
    pub xCampo: String,
    pub xTexto: String,
}

#[derive(Debug, Default, Serialize)]
pub struct ProcRef {
    pub nProc: String,
    pub indProc: String,
    pub tpAto: Option<String>,
}

pub fn parse_infAdic(reader: &mut XmlReader) -> Result<InfAdic, Box<dyn Error>> {
    let mut inf_adic = InfAdic::default();
    loop {
        match reader.read_event()? {
            Event::Start(e) => match e.name().as_ref() {
                b"obsCont" => {
                    inf_adic.obsCont.get_or_insert_with(Vec::new).push(parse_obsCont(reader, &e)?);
                }
                b"obsFisco" => {
                    inf_adic.obsFisco.get_or_insert_with(Vec::new).push(parse_obsFisco(reader, &e)?);
                }
                b"procRef" => {
                    inf_adic.procRef.get_or_insert_with(Vec::new).push(parse_procRef(reader)?);
                }
                name => {
                    let txt = read_text_string(reader, &e)?;
                    match name {
                        b"infAdFisco" => inf_adic.infAdFisco = Some(txt),
                        b"infCpl" => inf_adic.infCpl = Some(txt),
                        _ => (),
                    }
                }
            },
            Event::End(e) if e.name().as_ref() == b"infAdic" => return Ok(inf_adic),
            Event::Eof => return Err(Box::new(ParseError::UnexpectedEof("infAdic".to_string()))),
            _ => (),
        }
    }
}

fn parse_obsCont(reader: &mut XmlReader, e: &BytesStart) -> Result<ObsCont, Box<dyn Error>> {
    let mut obs: ObsCont = ObsCont::default();

    let attr = e.try_get_attribute(b"xCampo")?
        .ok_or("Atributo 'xCampo' obrigatório não encontrado em <obsCont>")?;
    obs.xCampo = attr.unescape_value()?.into_owned();

    // 2. Loop para ler o conteúdo interno
    loop {
        match reader.read_event()? {
            Event::Start(e) if e.name().as_ref() == b"xTexto" => {
                obs.xTexto = read_text_string(reader, &e)?;
            }
            // Encerra ao encontrar a tag de fechamento </obsCont>
            Event::End(e) if e.name().as_ref() == b"obsCont" => return Ok(obs),
            Event::Eof => return Err(Box::new(ParseError::UnexpectedEof("obsCont".to_string()))),
            _ => (),
        }
    }
}

fn parse_obsFisco(reader: &mut XmlReader, e: &BytesStart) -> Result<ObsFisco, Box<dyn Error>> {
    let mut obs: ObsFisco = ObsFisco::default();

    let attr = e.try_get_attribute(b"xCampo")?
        .ok_or("Atributo 'xCampo' obrigatório não encontrado em <obsFisco>")?;
    obs.xCampo = attr.unescape_value()?.into_owned();

    loop {
        match reader.read_event()? {
            Event::Start(e) if e.name().as_ref() == b"xTexto" => {
                obs.xTexto = read_text_string(reader, &e)?;
            }
            Event::End(e) if e.name().as_ref() == b"obsFisco" => return Ok(obs),

            Event::Eof => return Err(Box::new(ParseError::UnexpectedEof("obsFisco".to_string()))),
            _ => (),
        }
    }
}

fn parse_procRef(reader: &mut XmlReader) -> Result<ProcRef, Box<dyn Error>> {
    let mut proc_ref = ProcRef::default();
    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                let txt = read_text_string(reader, &e)?;
                match e.name().as_ref() {
                    b"nProc" => proc_ref.nProc = txt,
                    b"indProc" => proc_ref.indProc = txt,
                    b"tpAto" => proc_ref.tpAto = Some(txt),
                    _ => (),
                }
            }
            Event::End(e) if e.name().as_ref() == b"procRef" => return Ok(proc_ref),
            Event::Eof => return Err(Box::new(ParseError::UnexpectedEof("procRef".to_string()))),
            _ => (),
        }
    }
}

