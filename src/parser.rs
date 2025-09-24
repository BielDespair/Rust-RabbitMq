use std::fs;
use quick_xml::{reader, Reader};
use quick_xml::events::Event;

use crate::nfes::NFe;

mod logger;
mod nfes;

fn main() {
    logger::register_logger();
    let file:Vec<u8>  = read_file_bytes("./data/Mod55.xml").unwrap();

    
    let mut durations: Vec<std::time::Duration> = Vec::new();

    for i in 0..20_000 {
        let start = std::time::Instant::now();
        let modelo: ModNfe = get_mod_nfe(&file); // sua função
        let duration = start.elapsed();

        durations.push(duration);
    }

    // calcular tempo médio
    let total: std::time::Duration = durations.iter().sum();
    let avg = total / durations.len() as u32;

    println!("Tempo médio: {:.3?}", avg);

    





    /*
    let xml: String = fs::read_to_string("./data/Mod65.xml").unwrap();
    let conf: Config = Config::new_with_custom_values(leading_zero_as_string, xml_attr_prefix, xml_text_node_prop_name, empty_element_handling);
    let json = xml_string_to_json(xml.to_owned(), &conf);
    let result =  json.expect("Malformed XML").to_string();
    fs::write("./data/Nota1.json", result);

    let conf: Config = Config::new_with_custom_values(true, "", "txt", NullValue::Null);
    let json = xml_string_to_json(xml.to_owned(), &conf);
    println!("{}", json.expect("Malformed XML").to_string());
     */
}

pub fn parse_nfe(xml_bytes: &[u8]){

    let modelo: ModNfe = get_mod_nfe(&xml_bytes);
    let mut buf: Vec<u8> = Vec::new();

    match modelo {
        ModNfe::Mod55 => parse_nfe_mod_55(&xml_bytes),
        ModNfe::Mod57 => todo!(),
        ModNfe::Mod65 => todo!(),
        ModNfe::Desconhecido => todo!(),
    }
    parse_nfe_mod_55(&xml_bytes);
}


pub fn parse_nfe_mod_55(xml_bytes: &[u8]) {

}

pub fn parse_nfe_mod_57() {

}

pub fn parse_nfe_mod_65(xml_bytes: &[u8]) {
    let mut reader: Reader<&[u8] = Reader::from_reader(xml_bytes);
    reader.config_mut().trim_text(true);

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {

                // match ... ide, emit...
                // 
            }

            Ok(Event::Text(e)) => {
                let bytes: &[u8] = e.as_ref();
                let txt: &str = str::from_utf8(bytes).unwrap();
                println!("Conteúdo: {:p}", txt.as_ptr());
                println!("Fatia ref: {:p}", &txt);
            }

            Ok(Event::Eof) => { break; }

            Err(e) => { break; }

            _ => {}

        }

    }
}


fn get_mod_nfe(xml_bytes: &[u8]) -> ModNfe {
    let mut reader: Reader<&[u8]> = Reader::from_reader(xml_bytes);
    reader.config_mut().trim_text(true);

    let mut inside_mod: bool = false;

    loop {        
        match reader.read_event() {
            Ok(Event::Start(e)) if e.name().as_ref() == b"mod" => {
            inside_mod = true;
            }

            Ok(Event::Text(e)) if inside_mod => {
                let txt: String = e.decode().unwrap().into_owned();
                return ModNfe::from(txt.as_str());
            }

            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
    }

    return ModNfe::Desconhecido;
}

#[derive(Debug)]
pub enum ModNfe {
    Mod55,
    Mod57,
    Mod65,
    Desconhecido
}

impl From<&str> for ModNfe {
    fn from(s: &str) -> Self {
        match s {
            "55" => ModNfe::Mod55,
            "57" => ModNfe::Mod57,
            "65" => ModNfe::Mod65,
            _ => ModNfe::Desconhecido,
        }
    }
}

pub struct XmlJson<T> {
    company_id: i128,
    org_id: i128,

    nfes: Vec<T>
}

