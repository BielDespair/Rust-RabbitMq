#![allow(non_snake_case, non_camel_case_types)]

use std::error::Error;

use quick_xml::events::Event;
use rust_decimal::Decimal;
use serde::Serialize;

use crate::{nfe::common::{read_text_string, ParseError, XmlReader}, nfes::UF};


#[derive(Debug, Default, Serialize)]
pub struct Pag {
    pub detPag: Option<Vec<DetPag>>,
    pub vTroco: Option<Decimal>,
}

#[derive(Debug, Default, Serialize)]
pub struct DetPag {
    pub indPag: Option<String>,
    pub tPag: String,
    pub xPag: Option<String>,
    pub vPag: Decimal,
    pub dPag: Option<String>,
    pub CNPJPag: Option<String>,
    pub UFPag: Option<UF>,
    pub card: Option<Card>,
}

#[derive(Debug, Default, Serialize)]
pub struct Card {
    pub tpIntegra: String,
    pub CNPJ: Option<String>,
    pub tBand: Option<String>,
    pub cAut: Option<String>,
    pub CNPJReceb: Option<String>,
    pub idTermPag: Option<String>,
}

pub fn parse_pag(reader: &mut XmlReader) -> Result<Pag, Box<dyn Error>> {
    let mut pag = Pag::default();
    loop {
        match reader.read_event()? {
            Event::Start(e) => match e.name().as_ref() {
                // Adiciona cada ocorrência de <detPag> ao vetor
                b"detPag" => {
                    pag.detPag.get_or_insert_with(Vec::new).push(parse_detPag(reader)?);
                }
                b"vTroco" => {
                    let txt: String = read_text_string(reader, &e)?;
                    if !txt.is_empty() {
                        pag.vTroco = Some(txt.parse::<Decimal>()?);
                    }
                }
                _ => (),
            },
            Event::End(e) if e.name().as_ref() == b"pag" => return Ok(pag),
            Event::Eof => return Err(Box::new(ParseError::UnexpectedEof("pag".to_string()))),
            _ => (),
        }
    }
}



fn parse_detPag(reader: &mut XmlReader) -> Result<DetPag, Box<dyn Error>> {
    let mut det_pag = DetPag::default();
    loop {
        match reader.read_event()? {
            Event::Start(e) => match e.name().as_ref() {
                b"card" => det_pag.card = Some(parse_card(reader)?),
                name => {
                    let txt = read_text_string(reader, &e)?;
                    match name {
                        b"indPag" => det_pag.indPag = Some(txt),
                        b"tPag" => det_pag.tPag = txt,
                        b"xPag" => det_pag.xPag = Some(txt),
                        b"vPag" => det_pag.vPag = txt.parse::<Decimal>()?,
                        b"dPag" => det_pag.dPag = Some(txt),
                        b"CNPJPag" => det_pag.CNPJPag = Some(txt),
                        b"UFPag" => det_pag.UFPag = Some(UF::from(txt.as_str())),
                        _ => (),
                    }
                }
            },
            Event::End(e) if e.name().as_ref() == b"detPag" => return Ok(det_pag),
            Event::Eof => return Err(Box::new(ParseError::UnexpectedEof("detPag".to_string()))),
            _ => (),
        }
    }
}

fn parse_card(reader: &mut XmlReader) -> Result<Card, Box<dyn Error>> {
    let mut card = Card::default();
    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                let txt = read_text_string(reader, &e)?;
                match e.name().as_ref() {
                    b"tpIntegra" => card.tpIntegra = txt,
                    b"CNPJ" => card.CNPJ = Some(txt),
                    b"tBand" => card.tBand = Some(txt),
                    b"cAut" => card.cAut = Some(txt),
                    b"CNPJReceb" => card.CNPJReceb = Some(txt),
                    b"idTermPag" => card.idTermPag = Some(txt),
                    _ => (),
                }
            }
            Event::End(e) if e.name().as_ref() == b"card" => return Ok(card),
            Event::Eof => return Err(Box::new(ParseError::UnexpectedEof("card".to_string()))),
            _ => (),
        }
    }
}