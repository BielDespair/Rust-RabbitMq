#![allow(non_snake_case, non_camel_case_types)]
use std::error::Error;

use quick_xml::{
    Reader,
    events::{BytesStart, Event},
};
use rust_decimal::Decimal;
use serde::Serialize;

use crate::nfe::common::{ParseError, read_text};

#[derive(Debug, Default, Serialize)]
pub struct Agropecuario {
    #[serde(flatten)]
    pub item: AgropecuarioItem,
}

#[derive(Debug, Serialize)]
pub enum AgropecuarioItem {
    defensivo(Vec<Defensivo>),
    guiaTransito(GuiaTransito),
}

impl Default for AgropecuarioItem {
    fn default() -> Self {
        Self::guiaTransito(GuiaTransito::default())
    }
}

#[derive(Debug, Default, Serialize)]
pub struct Defensivo {
    pub nReceituario: String,
    pub CPFRespTec: String,
}

#[derive(Debug, Default, Serialize)]
pub struct GuiaTransito {
    pub tpGuia: String,
    pub UFGuia: String,
    pub serieGuia: Option<String>,
    pub nGuia: String,
}

pub fn parse_agropecuario(reader: &mut Reader<&[u8]>) -> Result<Agropecuario, Box<dyn Error>> {
    let mut agro: Agropecuario = Agropecuario::default();

    loop {
        match reader.read_event()? {
            Event::Start(e) => match e.name().as_ref() {
                b"defensivo" => {
                    let def: Defensivo = parse_defensivo(reader)?;
                    if let AgropecuarioItem::defensivo(ref mut vec) = agro.item {
                        vec.push(def);
                    } else {
                        agro.item = AgropecuarioItem::defensivo(vec![def]);
                    }
                }
                b"guiaTransito" => agro.item = AgropecuarioItem::guiaTransito(parse_guiaTransito(reader)?),

                _ => (),
            },

            Event::End(e) if e.name().as_ref() == b"agropecuario" => return Ok(agro),
            Event::Eof => return Err(Box::new(ParseError::UnexpectedEof("agropecuario".to_string()))),
            _ => {}
        }
    }
}

fn parse_defensivo(reader: &mut Reader<&[u8]>) -> Result<Defensivo, Box<dyn Error>> {
    let mut def: Defensivo = Defensivo::default();

    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                let txt: String = read_text(reader, &e)?;
                match e.name().as_ref() {
                    b"nReceituario" => def.nReceituario = txt,
                    b"CPFRespTec" => def.CPFRespTec = txt,
                    _ => (),
                }
            }
            Event::End(e) if e.name().as_ref() == b"defensivo" => return Ok(def),
            Event::Eof => return Err(Box::new(ParseError::UnexpectedEof("defensivo".to_string()))),
            _ => (),
        }
    }
}


fn parse_guiaTransito(reader: &mut Reader<&[u8]>) -> Result<GuiaTransito, Box<dyn Error>> {
    let mut guia: GuiaTransito = GuiaTransito::default();

    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                let txt: String = read_text(reader, &e)?;
                match e.name().as_ref() {
                    b"tpGuia" => guia.tpGuia = txt,
                    b"UFGuia" => guia.UFGuia = txt,
                    b"serieGuia" => guia.serieGuia = Some(txt),
                    b"nGuia" => guia.nGuia = txt,
                    _ => (),
                }
            }
            Event::End(e) if e.name().as_ref() == b"guiaTransito" => return Ok(guia),
            Event::Eof => return Err(Box::new(ParseError::UnexpectedEof("guiaTransito".to_string()))),
            _ => (),
        }
    }
}
