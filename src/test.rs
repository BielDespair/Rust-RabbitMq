use crate::{impostos::pis::PIS, nfes::Emit};

mod nfes;
mod impostos;


fn main() {

    let teste: COFINS = COFINS::default();

    //let emit: Emit = Emit::default();

    //emit.EmitenteId = nfes::EmitenteId::CPF { CPF: "12345678000195".to_string() };
    //println!("{:?}", emit);

    let json = serde_json::to_string(&pis).unwrap();
    println!("{}", json);

}