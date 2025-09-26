use std::fs;
use std::time::Instant;

use crate::minio_client::MinioVariables;
use dotenv::dotenv;
mod logger;
mod nfes;
mod nfe_parser;
mod minio_client;

#[tokio::main]
async fn main() {
    dotenv().ok();
    logger::register_logger();

    // medir tempo do init_client
    let t1 = Instant::now();
    let minio_variables: MinioVariables = minio_client::initialize_variables();
    minio_client::init_client(&minio_variables);
    let init_duration = t1.elapsed();
    println!("Init client: {:?}", init_duration);

    // medir tempo do download
    let t2 = Instant::now();
    //let object = String::from("NFCE33250800935769000100652010002482761002499990.xml");
    //let file: String = minio_client::download_object(&object, &minio_variables).await.expect("Failed to download file");
    //fs::write("./dump.xml", &file).expect("Failed to write dump.xml");
    let file: String = fs::read_to_string("./data/Mod65.xml").unwrap();

    println!("Download: {:?}", t2.elapsed());
    // medir tempo do parser
    let t3: Instant = Instant::now();
    let json: String = nfe_parser::parse_nfe(file).expect("Failed to parse XML");
    
    println!("Parse modelo: {:?}", t3.elapsed());
    println!("JSON: {}", json);
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
