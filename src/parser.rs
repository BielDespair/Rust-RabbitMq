use std::fs;
use quick_xml::Reader;
use quick_xml::events::Event;



fn main() {

    let xml: String = fs::read_to_string("./data/Mod65.xml").unwrap();
    let conf: Config = Config::new_with_custom_values(leading_zero_as_string, xml_attr_prefix, xml_text_node_prop_name, empty_element_handling);
    let json = xml_string_to_json(xml.to_owned(), &conf);
    let result =  json.expect("Malformed XML").to_string();
    fs::write("./data/Nota1.json", result);

    let conf: Config = Config::new_with_custom_values(true, "", "txt", NullValue::Null);
    let json = xml_string_to_json(xml.to_owned(), &conf);
    println!("{}", json.expect("Malformed XML").to_string());
}



// Lê um arquivo e devolve um vector de bytes.
fn read_file_bytes(path: &str) -> std::io::Result<Vec<u8>> {
    return fs::read(path);
}


fn get_tipo_nfe(xml_bytes: &[u8]) -> TipoNFe {
    let mut reader = Reader::From
    let mut reader = Reader::from_reader(xml_bytes);
    reader
    let mut buf = Vec::new();

    TipoNfe::Unknown
}


pub enum ModeloNFe {
    EnviNFe,  // Envelope de envio (pode conter múltiplas NFe)
    NFeProc,  // Documento processado (uma NFe)
    NFCe,     // NFC-e (uma nota)
    SAT,      // SAT (uma nota)
    Desconhecido,
}