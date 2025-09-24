
#[derive(Debug, Default)]
pub struct NFe {
    ide: Ide,
    emit: Emit,
    produtos: Option<>
}


#[derive(Debug, Default)]
pub struct Ide {
    pub cUF: u8,
    pub cNF: String,
    pub natOp: String,
    pub r#mod: u8,
    pub serie: u16,
    pub nNF: u32,
    pub dhEmi: String,
    pub dhSaiEnt: Option<String>,
    pub tpNF: u8,
    pub idDest: u8,
    pub cMunFG: u32,
    pub cMunFGIBS: Option<u32>,
    pub tpImp: u8,
    pub tpEmis: u8,
    pub cDV: u8,
    pub tpAmb: u8,
    pub finNFe: u8,
    pub tpNFDebito: Option<u8>,
    pub tpNFCredito: Option<u8>,
    pub indFinal: bool,
    pub indPres: u8,
    pub indIntermed: Option<bool>,
    pub procEmi: u8,
    pub verProc: String,
    pub dhCont: Option<String>,
    pub xJust: Option<String>,
    pub NFref: Option<Vec<NFRef>>
}


#[derive(Debug, Default)]
pub struct Emit {
    pub CNPJ: String,
    pub 
    pub xNome: String,
    pub xFant: Option<String>,

}




pub struct EnderEmit {
    pub xLgr: String,
    pub nro: String,
    pub xCpl: Option<String>,
    pub xBairro: String,
    pub cMun: u32,
    pub xMun: String,
    pub UF: UF,
    pub CEP: String,
    pub cPais: Option<String>,
    pub xPais: Option<String>,
    pub fone: Option<String>
}

#[derive(Debug)]
pub enum NFRef {
    refNFe { chave: String },
    refNFeSig { chave: String },
    refNF {
        cUF: u8,
        AAMM: String,
        CNPJ: String,
        r#mod: u8,
        serie: u16,
        nNF: u32
    },
    refNFP {
        cUF: u8,
        AAMM: String,
        id: EmitenteId,
         

    }
}

impl Default for NFRef {
    fn default() -> Self {
        NFRef::refNFe { chave: String::new() }
    }
}


#[derive(Debug)]
pub enum UF {
    AC, AL, AM, AP, BA, CE, DF, ES, GO, MA,
    MG, MS, MT, PA, PB, PE, PI, PR, RJ, RN,
    RO, RR, RS, SC, SE, SP, TO,
}

#[derive(Debug)]
enum EmitenteId {
    CNPJ {CNPJ: String},
    CPF {CPF: String}
}