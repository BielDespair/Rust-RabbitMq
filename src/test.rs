use std::fs;

use bytes::Bytes;
use dotenv::dotenv;
use quick_xml::{events::Event, Reader};

use crate::nfe_parser::parse_nfe;

mod logger;
mod nfes;
mod det;
mod nfe;
mod impostos;
mod nfe_parser;

mod minio_client;


#[tokio::main]
async fn main() {
    dotenv().ok();
    logger::register_logger();

    let variables: minio_client::MinioVariables = minio_client::initialize_variables();
    minio_client::init_client(&variables);

    let file: Bytes = minio_client::download_object("NFCe33250627708310000111650010006827679001864260.xml", &variables.bucket_name).await.expect("Failed");

    let json = nfe_parser::parse_nfe(file,0,0,);

    let json: String = match json {
        Ok(j) => j,
        Err(e) => {
            log::error!("Erro: {}", e);
            panic!();
        }
    };

    fs::write("./data/output.json", json).expect("Erro ao escrever o arquivo");



    /*
    let file: String = fs::read_to_string("./data/Mod65.xml").unwrap();
    //let obj = nfe_parser::parse_nfe(file,0,0,).unwrap();
    let start = std::time::Instant::now();
    let json = nfe_parser::parse_nfe(file,0,0,);

    let json: String = match json {
        Ok(j) => j,
        Err(e) => {
            log::error!("Erro: {}", e);
            panic!();
        }
    };
    print!("Total: {:?}", start.elapsed());

    //let json = serde_json::to_string_pretty(&obj).unwrap();
    fs::write("./data/output.json", json).expect("Erro ao escrever o arquivo");
 */
}