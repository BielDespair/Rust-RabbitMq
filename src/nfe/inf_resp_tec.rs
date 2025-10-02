#![allow(non_snake_case, non_camel_case_types)]
use std::error::Error;

use quick_xml::{events::Event, Reader};
use serde::{Deserialize, Serialize};

use crate::nfe::common::{read_text_string, ParseError};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TInfRespTec {
    pub CNPJ: String,
    pub xContato: String,
    pub email: String,
    pub fone: String,

    #[serde(default)]
    pub idCSRT: Option<String>,

    #[serde(default)]
    pub hashCSRT: Option<String>, // Base64, 20 bytes
}


pub fn parse_infRespTec(reader: &mut Reader<&[u8]>) -> Result<TInfRespTec, Box<dyn Error>> {
    let mut resp: TInfRespTec = TInfRespTec::default();

    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                let txt = read_text_string(reader, &e)?;
                match e.name().as_ref() {
                    
                    b"CNPJ" => resp.CNPJ = txt,
                    b"xContato" => resp.xContato = txt,
                    b"email" => resp.email = txt,
                    b"fone" => resp.fone = txt,
                    b"idCSRT" => resp.idCSRT = Some(txt),
                    b"hashCSRT" => resp.hashCSRT = Some(txt),
                     
                    _ =>  (),
                }
            }
            Event::End(e) if e.name().as_ref() == b"infRespTec" => return Ok(resp),
            Event::Eof => return Err(Box::new(ParseError::UnexpectedEof("infRespTec".to_string()))),
            _ => (),
        }
    }
}