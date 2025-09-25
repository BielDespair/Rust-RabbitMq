use crate::nfes::Emit;

mod nfes;


fn main() {

    let nfe: nfes::NFe = nfes::NFe::default();

    let mut nfe_clone = nfe.clone();

    nfe_clone.emit.EmitenteId = nfes::EmitenteId::CPF { CPF: "12345678000195".to_string() };
    //let emit: Emit = Emit::default();

    //emit.EmitenteId = nfes::EmitenteId::CPF { CPF: "12345678000195".to_string() };
    //println!("{:?}", emit);

    let json = serde_json::to_string(&nfe_clone).unwrap();
    println!("{}", json);

}