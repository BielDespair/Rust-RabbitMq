#![allow(non_snake_case, non_camel_case_types)]

use std::error::Error;

use quick_xml::events::Event;
use serde::Serialize;
use crate::{nfe::common::{read_text, ParseError, XmlReader}, nfes::UF};

#[derive(Debug, Default, Serialize)]
pub struct Exporta {
    pub UFSaidaPais: UF,

    pub xLocExporta: String,

    pub xLocDespacho: Option<String>,
}


pub fn parse_exporta(reader: &mut XmlReader) -> Result<Exporta, Box<dyn Error>> {
    let mut exporta = Exporta::default();
    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                let txt = read_text(reader, &e)?;
                match e.name().as_ref() {
                    b"UFSaidaPais" => exporta.UFSaidaPais = UF::from(txt.as_str()),
                    b"xLocExporta" => exporta.xLocExporta = txt,
                    b"xLocDespacho" => exporta.xLocDespacho = Some(txt),
                    _ => (),
                }
            }
            Event::End(e) if e.name().as_ref() == b"exporta" => return Ok(exporta),
            Event::Eof => return Err(Box::new(ParseError::UnexpectedEof("exporta".to_string()))),
            _ => (),
        }
    }
}