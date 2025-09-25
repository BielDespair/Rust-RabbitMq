use quick_xml::{events::Event, Reader};




pub fn parse_nfe(xml_bytes: &String) -> Option<String> {

    let modelo: ModNfe = get_mod_nfe(&xml_bytes);

    log::info!("Modelo: {:?}", modelo);
    return Some("ABC".to_string());
    /*
    match modelo {
        ModNfe::Mod55 => {return parse_nfe_mod_55(&xml_bytes)}
        ModNfe::Mod57 => {return parse_nfe_mod_57(&xml_bytes)}
        ModNfe::Mod65 => {return parse_nfe_mod_65(&xml_bytes)}


        ModNfe::Desconhecido => todo!(),
    }
     */
}




pub fn parse_nfe_mod_55(xml_bytes: &[u8]) -> Option<String> {

    return Some("MOD55-PARSED".to_string());
}

pub fn parse_nfe_mod_57(xml_bytes: &[u8]) -> Option<String> {
    return Some("MOD56-PARSED".to_string());
}

pub fn parse_nfe_mod_65(xml_bytes: &String) -> Option<String>{
    let mut reader: Reader<&[u8]> = Reader::from_str(xml_bytes);
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

            Ok(Event::Eof) => {
                break;
            }

            Err(e) => { break; }

            _ => {}

        }
    }

    return Some("Hello".to_string());

}


pub fn get_mod_nfe(xml_bytes: &String) -> ModNfe {
    let mut reader: Reader<&[u8]> = Reader::from_str(xml_bytes);
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
    fn from(value: &str) -> Self {
        match value {
            "55" => ModNfe::Mod55,
            "57" => ModNfe::Mod57,
            "65" => ModNfe::Mod65,
            _ => ModNfe::Desconhecido,
        }
    }
}