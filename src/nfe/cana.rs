#![allow(non_snake_case, non_camel_case_types)]
use std::error::Error;

use quick_xml::{events::{BytesStart, Event}, Reader};
use rust_decimal::Decimal;
use serde::Serialize;

use crate::nfe::common::{read_text_string, ParseError};

#[derive(Debug, Default, Serialize)]
pub struct Cana {
    pub safra: String,
    pub r#ref: String,
    pub forDia: Vec<ForDia>,
    pub qTotMes: Decimal,
    pub qTotAnt: Decimal,
    pub qTotGer: Decimal,
    pub deduc: Option<Vec<Deduc>>,
    pub vFor: Decimal,
    pub vTotDed: Decimal,
    pub vLiqFor: Decimal,
}

#[derive(Debug, Default, Serialize)]
pub struct ForDia {
    pub dia: String,
    pub qtde: Decimal,
}

#[derive(Debug, Default, Serialize)]
pub struct Deduc {
    pub xDed: String,
    pub vDed: Decimal,
}

pub fn parse_cana(reader: &mut Reader<&[u8]>) -> Result<Cana, Box<dyn Error>> {
    let mut cana: Cana = Cana::default();

    loop {
        match reader.read_event()? {
            Event::Start(e) => match e.name().as_ref() {
                b"forDia" => cana.forDia.push(parse_forDia(reader, &e)?),
                b"deduc" => cana.deduc.get_or_insert_default().push(parse_deduc(reader)?),

                name => {
                    let txt: String = read_text_string(reader, &e)?;
                    match name {
                        b"safra" => cana.safra = txt,
                        b"ref" => cana.r#ref = txt,
                        b"qTotMes" => cana.qTotMes = txt.parse::<Decimal>()?,
                        b"qTotAnt" => cana.qTotAnt = txt.parse::<Decimal>()?,
                        b"qTotGer" => cana.qTotGer = txt.parse::<Decimal>()?,
                        b"vFor" => cana.vFor = txt.parse::<Decimal>()?,
                        b"vTotDed" => cana.vTotDed = txt.parse::<Decimal>()?,
                        b"vLiqFor" => cana.vLiqFor = txt.parse::<Decimal>()?,
                        _ => (),
                    }
                }
            },

            Event::End(e) if e.name().as_ref() == b"cana" => return Ok(cana),
            Event::Eof => return Err(Box::new(ParseError::UnexpectedEof("cana".to_string()))),
            _ => (),
        }
    }
}

fn parse_forDia(reader: &mut Reader<&[u8]>, e: &BytesStart) -> Result<ForDia, Box<dyn Error>> {
    let mut f: ForDia = ForDia::default();
    
    let attr = e.try_get_attribute(b"dia")?
        .ok_or("Atributo 'dia' obrigatório não encontrado em <obsCont>")?;
    f.dia = attr.unescape_value()?.into_owned();

    loop {
        match reader.read_event()? {
            Event::Start(e) if e.name().as_ref() == b"qtde" => {
                let txt: String = read_text_string(reader, &e)?;
                f.qtde = txt.parse::<Decimal>()?;
            }
            Event::End(e) if e.name().as_ref() == b"forDia" => return Ok(f),
            Event::Eof => return Err(Box::new(ParseError::UnexpectedEof("forDia".to_string()))),
            _ => (),
        }
    }
}

fn parse_deduc(reader: &mut Reader<&[u8]>) -> Result<Deduc, Box<dyn Error>> {
    let mut d: Deduc = Deduc::default();

    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                let txt = read_text_string(reader, &e)?;
                match e.name().as_ref() {
                    b"xDed" => d.xDed = txt,
                    b"vDed" => d.vDed = txt.parse()?,
                    _ => (),
                }
            }
            Event::End(e) if e.name().as_ref() == b"deduc" => return Ok(d),
            Event::Eof => return Err(Box::new(ParseError::UnexpectedEof("deduc".to_string()))),
            _ => (),
        }
    }
}