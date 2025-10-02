#![allow(non_snake_case, non_camel_case_types)]

use std::error::Error;

use quick_xml::events::Event;
use serde::Serialize;

use crate::nfe::common::{read_text_string, ParseError, XmlReader};

/// Informações de compras (Nota de Empenho, Pedido e Contrato)
#[derive(Debug, Default, Serialize)]
pub struct Compra {
    /// Informação da Nota de Empenho de compras públicas.
    pub xNEmp: Option<String>,
    /// Informação do pedido.
    pub xPed: Option<String>,
    /// Informação do contrato.
    pub xCont: Option<String>,
}


pub fn parse_compra(reader: &mut XmlReader) -> Result<Compra, Box<dyn Error>> {
    let mut compra: Compra = Compra::default();
    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let txt = read_text_string(reader, &e)?;
                match e.name().as_ref() {
                    b"xNEmp" => compra.xNEmp = Some(txt),
                    b"xPed" => compra.xPed = Some(txt),
                    b"xCont" => compra.xCont = Some(txt),
                    _ => (), // Ignora tags desconhecidas dentro de <compra>
                }
            }
            Ok(Event::End(e)) if e.name().as_ref() == b"compra" => return Ok(compra),
            Ok(Event::Eof) => return Err(Box::new(ParseError::UnexpectedEof("compra".to_string()))),
            _ => (),
        }
    }
}