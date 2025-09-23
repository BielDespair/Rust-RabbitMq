use std::fs;
use quick_xml::Reader;
use quick_xml::events::Event;

mod logger;

fn main() {
    logger::register_logger();
    let file:Vec<u8>  = read_file_bytes("./data/Mod65.xml").unwrap();

    let modelo: ModNfe = get_mod_nfe(&file);

    println!("Modelo: {:?}", modelo);

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



// Lê um arquivo e devolve um vector de bytes.
fn read_file_bytes(path: &str) -> std::io::Result<Vec<u8>> {
    return fs::read(path);
}


fn get_mod_nfe(xml_bytes: &[u8]) -> ModNfe {
    let mut reader: Reader<&[u8]> = Reader::from_reader(xml_bytes);
    reader.config_mut().trim_text(true);




    log::info!("Searching mod...");
    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) if e.name().as_ref() == b"mod" => {
            log::info!("Found mod!!");
            }

            Ok(Event::Text(e)) => {
                let txt: String = e.decode().unwrap().into_owned();
                log::info!("Text: {}", txt);
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


