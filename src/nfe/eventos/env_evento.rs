#![allow(non_snake_case, non_camel_case_types)]


// Envelope de Evento
pub struct EnvEvento {
    pub idLote: String,
    pub evento: Vec<Evento>,
}