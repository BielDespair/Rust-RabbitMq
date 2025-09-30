#![allow(non_snake_case, non_camel_case_types)]
use rust_decimal::Decimal;
use serde::{Serialize};

use crate::impostos::{cofins::COFINS, cofins_st::COFINSST, ibs_cbs::IBSCBS, icms::Icms, icms_uf_dest::ICMSUFDest, ii::Ii, ipi::{Ipi}, is::IS, issqn::ISSQN, pis::PIS, pis_st::PISST};



#[derive(Debug, Default, Serialize)]
pub struct NfeJson {
    pub company_id: i128,
    pub org_id: i128, 
    pub nfes: Vec<NFe>,
}
#[derive(Debug, Default, Serialize)]
pub struct NFe {
    pub Id: String,
    pub ide: Ide,
    pub emit: Emit,
    pub produtos: Vec<Det>,
    pub avulsa: Option<Avulsa>,
    pub dest: Option<Dest>,
    pub retirada: Option<Local>,
    pub entrega: Option<Local>, 
    pub autXML: Option<Vec<EmitenteId>>,
}

#[derive(Debug, Default, Serialize)]
pub struct Det {
    pub produto: Prod,
    pub imposto: Imposto
}

#[derive(Debug, Default, Serialize)]
pub struct Prod {
    pub cProd: String,
    pub cEAN: String,
    pub cBarra: Option<String>,
    pub xProd: String,
    pub NCM: String,
    pub NVE: Option<Vec<String>>,
    pub CEST: Option<String>,
    pub indEscala: Option<String>,
    pub CNPJFab: Option<String>,
    pub cBenef: Option<String>,
    pub gCred: Option<Vec<GCred>>,
    pub EXTIPI: Option<String>,
    pub CFOP: String,
    pub uCom: String,
    pub qCom: Decimal,
    pub vUnCom: Decimal,
    pub vProd: Decimal,
    pub cEANTrib: String,
    pub cBarraTrib: Option<String>,
    pub uTrib: String,
    pub qTrib: Decimal,
    pub vUnTrib: Decimal,
    pub vFrete: Option<Decimal>,
    pub vSeg: Option<Decimal>,
    pub vDesc: Option<Decimal>,
    pub vOutro: Option<Decimal>,
    pub indTot: bool,
    pub indBemMovelUsado: Option<bool>,
    pub DI: Option<Vec<DI>>,
    pub detExport: Option<Vec<DetExport>>,
    pub xPed: Option<String>,
    pub nItemPed: Option<String>,
    pub nFCI: Option<String>,
    //pub rastro: Option<Vec<String>>,
    pub infProdNFF: Option<InfProdNFF>,
    pub infProdEmb: Option<InfProdEmb>,
    #[serde(flatten)]
    pub especifico: Option<ProdutoEspecifico>,



    // More fields can be added as needed
}

#[derive(Debug, Default, Serialize)]
pub struct Imposto {
    pub vTotTrib: Option<Decimal>,
    #[serde(flatten)]
    pub tributacao: Option<Tributacao>,
    pub PIS: Option<PIS>,
    pub PISST: Option<PISST>,
    pub COFINS: Option<COFINS>,
    pub COFINSST: Option<COFINSST>,
    pub ICMSUFDest: Option<ICMSUFDest>,
    pub IS: Option<IS>,
    pub IBSCBS: Option<IBSCBS>
    
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum Tributacao {
    Mercadoria {
        ICMS: Icms,
        IPI: Option<Ipi>,
        II: Option<Ii>
    },
    Servico {
        IPI: Option<Ipi>,
        ISSQN: ISSQN
    }
}

impl Default for Tributacao {
    fn default() -> Self {
        return Self::Mercadoria { ICMS: Icms::default(), IPI: None, II: None }
    }
}

#[derive(Debug, Default, Serialize)]
pub struct Veiculo {
    pub tpOp: u8,
    pub chassi: String,
    pub cCor: String,
    pub xCor: String,
    pub pot: String,
    pub cilin: String,
    pub pesoL: String,
    pub pesoB: String,
    pub nSerie: String,
    pub tpComb: u8,
    pub nMotor: String,
    pub CMT: String,
    pub dist: String,
    pub anoMod: String,
    pub anoFab: String,
    pub tpPint: String,
    pub tpVeic: String,
    pub espVeic: String,
    pub VIN: char,
    pub condVeic: String,
    pub cMod: String,
    pub cCorDENATRAN: String,
    pub lota: u32,
    pub tpRest: u8
}

#[derive(Debug, Default, Serialize)]
pub struct Medicamento {
    pub cProdANVISA: String,
    pub xMotivoIsencao: Option<String>,
    pub vPMC: Decimal,

}

#[derive(Debug, Default, Serialize)]
pub struct Arma {
    pub tpArma: String,
    pub nSerie: String,
    pub nCano: String,
    pub descr: String,
}

#[derive(Debug, Default, Serialize)]
pub struct Combustivel {
    pub cProdANP: String,
    pub descANP: String,
    pub pGLP: Option<String>,
    pub pGNn: Option<String>,
    pub pGNi: Option<String>,
    pub vPart: Option<String>,
    pub CODIF: Option<String>,
    pub qTemp: Option<String>,
    pub UFCons: UF,
    pub CIDE: Option<Cide>,
    pub encerrante: Option<Encerrante>,
    pub pBio: Option<Decimal>,
    pub origComb: Option<Vec<OrigComb>>
}

#[derive(Debug, Default, Serialize)]
pub struct Cide {
    pub qBCProd: Decimal,
    pub vAliqProd: Decimal,
    pub vCIDE: Decimal
}

#[derive(Debug, Default, Serialize)]
pub struct Encerrante {
    pub nBico: String,
    pub nBomba: Option<String>,
    pub nTanque: String,
    pub vEncIni: Decimal,
    pub vEncFin: Decimal,
}

#[derive(Debug, Default, Serialize)]
pub struct OrigComb {
    pub indImport: bool,
    pub cUFOrig: u8,
    pub pOrig: Decimal,

}


#[derive(Debug, Serialize)]
pub enum ProdutoEspecifico {
    veicProd(Veiculo),
    med(Medicamento),
    arma(Vec<Arma>),
    comb(Combustivel),
    nRECOPI(String)
}

impl Default for ProdutoEspecifico {
    fn default() -> Self {
        ProdutoEspecifico::nRECOPI("".to_string())
    }
}

#[derive(Debug, Default, Serialize)]
pub struct GCred {
    pub cCredPresumido: String,
    pub pCredPresumido: Decimal,
    pub vCredPresumido: Decimal,
}

#[derive(Debug, Default, Serialize)]
pub struct DI {
    pub nDI: String,
    pub dDI: String,
    pub xLocDesemb: String,
    pub UFDesemb: UF,
    pub dDesemb: String,
    pub tpViaTransp: u8,
    pub vAFRMM: Option<Decimal>,
    pub tpIntermedio: u8,
    pub EmitenteId: EmitenteId,
    pub UFTerceiro: Option<UF>,
    pub cExportador: Option<String>,
    pub adi: Vec<Adi>
    // More fields can be added as needed
}

#[derive(Debug, Default, Serialize)]
pub struct Adi {
    pub nAdicao: Option<String>,
    pub nSeqAdic: Option<String>,
    pub cFabricante: String,
    pub vDescDI: Option<Decimal>,
    pub nDraw: Option<String>,
}

#[derive(Debug, Default, Serialize)]
pub struct DetExport {
    pub nDraw: Option<String>,

    #[serde(flatten)]
    pub exportInd: Option<ExportInd>
}

#[derive(Debug, Default, Serialize)]
pub struct ExportInd {
    pub nRE: String,
    pub chNFe: String,
    pub qExport: Decimal,
}

#[derive(Debug, Default, Serialize)]
pub struct InfProdNFF {
    pub cProdFisco: String,
    pub cOperNFF: String,
}

#[derive(Debug, Default, Serialize)]
pub struct InfProdEmb {
    pub xEmb: String,
    pub qVolEmb: Decimal,
    pub uEmb: String
}


#[derive(Debug, Default, Serialize)]
pub struct Ide {
    pub cUF: u8,
    pub cNF: String,
    pub natOp: String,
    pub r#mod: u8,
    pub serie: u16,
    pub nNF: u32,
    pub dhEmi: String,
    pub dhSaiEnt: Option<String>,
    pub tpNF: bool,
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
    pub NFref: Option<Vec<NFRef>>,
    pub gCompraGov: Option<CompraGov>,
    pub gPagAntecipado: Option<Vec<String>>,
}


#[derive(Debug, Default, Serialize)]
pub struct Emit {
    #[serde(flatten)]
    pub EmitenteId: EmitenteId,
    pub xNome: String,
    pub xFant: Option<String>,
    pub enderEmit: EnderEmi,
    pub IE: String,
    pub IEST: Option<String>,
    pub IM: Option<String>,
    pub CNAE: Option<String>,
    pub CRT: u8,
}

#[derive(Debug, Default, Serialize)]
pub struct Avulsa {
    pub CNPJ: Option<String>,
    pub xOrgao: String,
    pub matr: String,
    pub xAgente: String,
    pub fone: Option<String>,
    pub UF: UF,
    pub nDAR: Option<String>,
    pub dEmi: Option<String>,
    pub vDAR: Option<Decimal>,
    pub repEmi: String,
    pub dPag: Option<String>,
}

#[derive(Debug, Default, Serialize)]
pub struct Dest {
    #[serde(flatten)]
    pub EmitenteId: EmitenteId,

    pub xNome: String,
    pub enderDest: EnderEmi,
    pub IE: Option<String>,
    pub ISUF: Option<String>,
    pub IM: Option<String>,
    pub email: Option<String>,

}


#[derive(Debug, Default, Serialize)]
pub struct EnderEmi {
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

#[derive(Debug, Default, Serialize)]
pub struct Local {
    #[serde(flatten)]
    pub EmitenteId: EmitenteId,

    pub xNome: Option<String>,
    pub xLgr: String,
    pub nro: String,
    pub xCpl: Option<String>,
    pub xBairro: String,
    pub cMun: u32,
    pub xMun: String,
    pub UF: UF,
    pub CEP: Option<String>,
    pub cPais: Option<String>,
    pub xPais: Option<String>,
    pub fone: Option<String>,
    pub email: Option<String>,
    pub IE: Option<String>,
}

#[derive(Debug, Default, Serialize)]
pub struct CompraGov {
    pub tpEnteGov: u8,
    pub pRedutor: Decimal,
    pub tpOperGov: u8,
}

#[derive(Debug, Serialize)]
pub enum NFRef {
    refNFe (String),
    refNFeSig (String),
    refNF (RefNFData),
    refNFP (RefNFPData),
    refCTe (String),
    refECF (RefECFData)
}

impl Default for NFRef {
    fn default() -> Self {
        NFRef::refNFe(String::new())
    }
}

#[derive(Debug, Default, Serialize)]
pub struct RefNFData {
    pub cUF: u8,
    pub AAMM: String,
    pub CNPJ: String,
    pub r#mod: u8,
    pub serie: u16,
    pub nNF: u32
}

#[derive(Debug, Default, Serialize)]
pub struct RefNFPData {
    pub cUF: u8,
    pub AAMM: String,
    #[serde(flatten)]
    pub EmitenteId: EmitenteId,
    pub IE: String,
    pub r#mod: u8,
    pub serie: u16,
    pub nNF: u32
}

#[derive(Debug, Default, Serialize)]
pub struct RefECFData {
    pub r#mod: String,
    pub nECF: String,
    pub nCOO: String
}


#[derive(Debug, Serialize)]
pub enum UF {
    AC, AL, AM, AP, BA, CE, DF, ES, GO, MA,
    MG, MS, MT, PA, PB, PE, PI, PR, RJ, RN,
    RO, RR, RS, SC, SE, SP, TO, EX,
}

impl Default for UF {
    fn default() -> Self {
        UF::MG
    }
    
}

impl From<&str> for UF {
    fn from(s: &str) -> Self {
        match s {
            "AC" => UF::AC,
            "AL" => UF::AL,
            "AM" => UF::AM,
            "AP" => UF::AP,
            "BA" => UF::BA,
            "CE" => UF::CE,
            "DF" => UF::DF,
            "ES" => UF::ES,
            "GO" => UF::GO,
            "MA" => UF::MA,
            "MG" => UF::MG,
            "MS" => UF::MS,
            "MT" => UF::MT,
            "PA" => UF::PA,
            "PB" => UF::PB,
            "PE" => UF::PE,
            "PI" => UF::PI,
            "PR" => UF::PR,
            "RJ" => UF::RJ,
            "RN" => UF::RN,
            "RO" => UF::RO,
            "RR" => UF::RR,
            "RS" => UF::RS,
            "SC" => UF::SC,
            "SE" => UF::SE,
            "SP" => UF::SP,
            "TO" => UF::TO,
            "EX" => UF::EX,
            _ => UF::default(),
        }
    }
}

#[derive(Debug, Serialize)]
pub enum EmitenteId {
    CNPJ(String),
    CPF (String),
    idEstrangeiro(String),
}

impl Default for EmitenteId {
    fn default() -> Self {
        EmitenteId::CNPJ(String::new())
    }
}