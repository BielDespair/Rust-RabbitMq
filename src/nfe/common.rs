use std::error::Error;

use quick_xml::{events::BytesStart, Reader};



pub type XmlReader<'a> = Reader<&'a [u8]>;

#[derive(Debug)]
pub enum ParseError {
    ModeloDesconhecido,
    IdNaoEncontrado,
    CampoDesconhecido(String),
    UnexpectedEof(String),
    Xml(String),
}

impl Error for ParseError {}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::ModeloDesconhecido => write!(f, "Modelo de NFe desconhecido ou não suportado"),
            ParseError::CampoDesconhecido(campo) => {
                write!(f, "Campo não mapeado encontrando: {}", campo)
            }
            ParseError::IdNaoEncontrado => write!(f, "Id da NFe não encontrado"),
            ParseError::Xml(e) => write!(f, "XML malformado: {}", e),
            ParseError::UnexpectedEof(item) => {
                write!(f, "Unexpected Eof while parsing {}", item)
            }
        }
    }
}

#[inline]
pub fn read_text(reader: &mut XmlReader, e: &BytesStart) -> Result<String, Box<dyn Error>> {
    let txt = reader.read_text(e.name())?;
    Ok(txt.into_owned())
}

#[inline]
pub fn get_tag_attribute(e: &BytesStart, key: &[u8]) -> Result<String, ParseError> {
    for attr in e.attributes() {
        let attr = attr.map_err(|e| ParseError::Xml(e.to_string()))?;
        if attr.key.as_ref() == key {
            let value = attr
                .unescape_value()
                .map_err(|e| ParseError::Xml(e.to_string()))?;
            return Ok(value.into_owned());
        }
    }
    Err(ParseError::IdNaoEncontrado)
}