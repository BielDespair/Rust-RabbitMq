use std::fs;
use std::time::{Duration, Instant};

use crate::minio_client::MinioVariables;
use dotenv::dotenv;
mod logger;
mod nfes;
mod nfe_parser;
mod minio_client;
mod impostos;

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
    let file: String = fs::read_to_string("./data/Mod55.xml").unwrap();

    println!("Download: {:?}", t2.elapsed());
    // medir tempo do parser
    
    
    const NUM_ITERATIONS: u32 = 20_000;
    let mut parse_times: Vec<Duration> = Vec::new();

    for _ in 0..NUM_ITERATIONS {
        let t_start = Instant::now();
        let _json = nfe_parser::parse_nfe(file.clone(), 7, 2).unwrap();
        parse_times.push(t_start.elapsed());
    }

    let total: Duration = parse_times.iter().sum(); // requires nightly ou manualmente
    let avg = parse_times.iter().sum::<Duration>() / NUM_ITERATIONS;
    println!("Total: {:?}, Média: {:?}", total, avg);
    //println!("JSON: {}", json);

    /*
    const NUM_ITERATIONS: u32 = 1;
    let mut parse_times: Vec<Duration> = Vec::new();

    println!("Iniciando benchmark com {} execuções...", NUM_ITERATIONS);

    // 1. Repete o processo 20 vezes
    for i in 1..=NUM_ITERATIONS {
        // 2. Lê o arquivo a cada iteração
        let file: String = fs::read_to_string("./data/Mod65.xml").unwrap();

        // 3. Mede o tempo de parse
        let t_start: Instant = Instant::now();
        let _json: String = nfe_parser::parse_nfe(file).expect("Failed to parse XML");
        let elapsed: Duration = t_start.elapsed();

        // 4. Guarda o tempo gasto no vetor
        parse_times.push(elapsed);

        // 5. Printe o tempo gasto em cada iteração usando log::info!
        log::info!("Iteração {}/{}: Parse levou {:?}", i, NUM_ITERATIONS, elapsed);
    }

    // 6. Calcula a média e o total no final
    if !parse_times.is_empty() {
        let total_duration_micros: u128 = parse_times.iter().map(|d| d.as_micros()).sum();
        let average_micros = total_duration_micros / parse_times.len() as u128;
        let average_millis = average_micros as f64 / 1000.0;
        
        // NOVO: Cria um objeto Duration a partir do total de microssegundos para exibição
        let total_duration = Duration::from_micros(total_duration_micros as u64);

        println!("\n--- Resultados do Benchmark ---");
        println!("Número de execuções: {}", NUM_ITERATIONS);
        // NOVO: Exibe o tempo total formatado
        println!("Tempo total para as {} execuções: {:?}", NUM_ITERATIONS, total_duration);
        println!("Média de tempo de parse: {:.3} ms ({} µs)", average_millis, average_micros);

    } else {
        println!("Nenhum tempo de parse foi registrado.");
    }
     */

    
}
