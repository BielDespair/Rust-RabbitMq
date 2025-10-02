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
    Outros(String),
}

impl Error for ParseError {}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::ModeloDesconhecido => write!(f, "Modelo de NFe desconhecido"),
            ParseError::CampoDesconhecido(campo) => {
                write!(f, "Campo não mapeado encontrando: {}", campo)
            }
            ParseError::IdNaoEncontrado => write!(f, "Id da NFe não encontrado"),
            ParseError::Xml(e) => write!(f, "XML malformado: {}", e),
            ParseError::Outros(msg) => write!(f, "Erro: {}", msg),
            ParseError::UnexpectedEof(item) => {
                write!(f, "Unexpected Eof while parsing {}", item)
            }
        }
    }
}

#[inline]
pub fn read_text_string(reader: &mut XmlReader, e: &BytesStart) -> Result<String, Box<dyn Error>> {
    let txt = reader.read_text(e.name())?;
    Ok(txt.into_owned())
}


