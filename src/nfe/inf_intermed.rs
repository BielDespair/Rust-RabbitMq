#![allow(non_snake_case, non_camel_case_types)]
use std::error::Error;

use quick_xml::events::Event;
use serde::Serialize;

use crate::nfe::common::{read_text_string, ParseError, XmlReader};



#[derive(Debug, Default, Serialize)]
pub struct InfIntermed {
    pub CNPJ: String,
    pub idCadIntTran: String,
}

pub fn parse_infIntermed(reader: &mut XmlReader) -> Result<InfIntermed, Box<dyn Error>> {
    let mut inf_intermed: InfIntermed = InfIntermed::default();
    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let txt: String = read_text_string(reader, &e)?;
                match e.name().as_ref() {
                    b"CNPJ" => inf_intermed.CNPJ = txt,
                    b"idCadIntTran" => inf_intermed.idCadIntTran = txt,
                    _ => (),
                }
            }
            Ok(Event::End(e)) if e.name().as_ref() == b"infIntermed" => return Ok(inf_intermed),
            Ok(Event::Eof) => return Err(Box::new(ParseError::UnexpectedEof("infIntermed".to_string()))),
            _ => (),
        }
    }
}