use std::fs;

use quick_xml::Reader;

use crate::{impostos::pis::PIS, nfes::{Emit, ProdutoEspecifico}};

mod nfes;
mod impostos;
mod nfe_parser;


fn main() {

    let file: String = fs::read_to_string("./data/Mod65.xml").unwrap();
    //let obj = nfe_parser::parse_nfe(file,0,0,).unwrap();
    let json = nfe_parser::parse_nfe(file,0,0,).unwrap();
    

    //let json = serde_json::to_string_pretty(&obj).unwrap();
    fs::write("./data/output.json", json).expect("Erro ao escrever o arquivo");

}