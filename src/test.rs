use std::fs;

mod logger;
mod nfes;
mod det;
mod nfe;
mod impostos;
mod nfe_parser;


fn main() {
    
    logger::register_logger();
    let file: String = fs::read_to_string("./data/Mod55.xml").unwrap();
    //let obj = nfe_parser::parse_nfe(file,0,0,).unwrap();
    let start = std::time::Instant::now();
    let json = nfe_parser::parse_nfe(file,0,0,).unwrap();
    print!("Total: {:?}", start.elapsed());

    //let json = serde_json::to_string_pretty(&obj).unwrap();
    fs::write("./data/output.json", json).expect("Erro ao escrever o arquivo");

}