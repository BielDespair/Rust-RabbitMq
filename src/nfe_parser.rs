use core::panic;
use std::{error::Error, fmt};

use quick_xml::{
    Reader,
    events::{BytesStart, Event},
};
use rust_decimal::Decimal;

use crate::{impostos::{icms::{Icms, TributosMercadoria}, ipi::Ipi}, nfes::{Adi, Arma, Cide, Combustivel, CompraGov, Det, DetExport, Emit, EmitenteId, Encerrante, EnderEmi, ExportInd, GCred, Ide, Imposto, InfProdEmb, InfProdNFF, Medicamento, NFRef, NFe, NfeJson, OrigComb, Prod, ProdutoEspecifico, RefECFData, RefNFData, RefNFPData, Tributacao, Veiculo, DI, UF}};

type XmlReader<'a> = Reader<&'a [u8]>;

#[derive(Debug)]
pub enum ModNfe {
    Mod55,
    Mod57,
    Mod65,
    Desconhecido,
}
impl From<&str> for ModNfe {
    fn from(value: &str) -> Self {
        match value {
            "55" => ModNfe::Mod55,
            "57" => ModNfe::Mod57,
            "65" => ModNfe::Mod65,
            _ => ModNfe::Desconhecido,
        }
    }
}
#[derive(Debug)]
pub enum ParseError {
    ModeloDesconhecido,
    IdNaoEncontrado,
    UnexpectedEof(String),
    Xml(String),
    Outros(String),
}

impl Error for ParseError {}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::ModeloDesconhecido => write!(f, "Modelo de NFe desconhecido"),
            ParseError::IdNaoEncontrado => write!(f, "Id da NFe não encontrado"),
            ParseError::Xml(e) => write!(f, "XML malformado: {}", e),
            ParseError::Outros(msg) => write!(f, "Erro: {}", msg),
            ParseError::UnexpectedEof(item) => write!(f, "Unexpected Eof at {} while parsing ide", item),
        }
    }
}

pub fn parse_nfe(xml_bytes: String, company_id: i128, org_id: i128) -> Result<String, Box<dyn Error>> {
    let modelo: ModNfe = get_mod_nfe(&xml_bytes)?;

    match modelo {
        // Modelo 55 e 65 são compatíveis para o parser
        ModNfe::Mod55 => return parse_nfe_mod_65(xml_bytes, company_id, org_id),
        ModNfe::Mod65 => return parse_nfe_mod_65(xml_bytes, company_id, org_id),
        ModNfe::Mod57 => return parse_nfe_mod_57(xml_bytes),
        ModNfe::Desconhecido => Err(ParseError::ModeloDesconhecido.into()),
    }
}

pub fn parse_nfe_mod_65(xml: String, company_id: i128, org_id: i128) -> Result<String, Box<dyn Error>> {
    let mut reader: Reader<&[u8]> = Reader::from_str(&xml);
    reader.config_mut().trim_text(true);

    let mut nfe_json: NfeJson = NfeJson::default();
    nfe_json.company_id = company_id;
    nfe_json.org_id = org_id;

    reader.config_mut().trim_text(true);

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let name = e.name();

                match name.as_ref() {
                    b"enviNFe" => {
                        parse_enviNfe_65(&mut reader);
                    }
                    b"nfeProc" => {
                        let nfe: NFe = parse_nfeProc_65(&mut reader)?;
                        nfe_json.nfes.push(nfe);
                        break;
                    }
                    _ => {
                        return Err(ParseError::Xml(format!("Elemento raiz desconhecido: {}", std::str::from_utf8(name.as_ref()).unwrap_or("<inválido>"))).into());
                    }
                }
            }

            _ => {}
        }
    }
    if nfe_json.nfes.is_empty() {
        return Err(ParseError::Outros("Nenhuma NFe encontrada no XML".to_string()).into());
    }
    
    let json = serde_json::to_string(&nfe_json)?;
    return Ok(json);
}

#[allow(non_snake_case)]
fn parse_nfeProc_65(reader: &mut XmlReader) -> Result<NFe, Box<dyn Error>> {
    

    let mut nfe: NFe = NFe::default();

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => match e.name().as_ref() {
               
                b"infNFe" => nfe.Id = get_nfe_id(&e)?,
                b"ide" => nfe.ide = parse_ide(reader)?,
                b"emit" => nfe.emit = parse_emit(reader)?,
                b"det" => nfe.produtos.push(parse_det(reader)?),
                _ => {}
                }
            
            Ok(Event::Eof) => {
                return Ok(nfe);
            }

            Err(e) => return Err(e.into()),

            _ => {}
        }
    }
    //return Err(Box::<dyn Error>::from("Unexpected error while parsing nfeProc."));
     
}

#[allow(non_snake_case)]
fn parse_enviNfe_65(reader: &mut XmlReader) -> Vec<NFe> {
    return Vec::new();
}

pub fn parse_nfe_mod_57(_xml: String) -> Result<String, Box<dyn std::error::Error>> {
    Err(Box::<dyn std::error::Error>::from("Not implemented"))
}

pub fn parse_ide(reader: &mut XmlReader) -> Result<Ide, Box<dyn Error>> {
    // Começa com uma struct com valores padrão
    let mut ide: Ide = Ide::default();

    // Transformar NFRef em Option<Vec<NFRef>>
    // Ver as nuâncias de como lidar com isso

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => match e.name().as_ref() {
                b"NFref" => ide.NFref.get_or_insert_with(Vec::new).push(parse_nfref(reader)?),
                b"gCompraGov" => ide.gCompraGov = Some(parse_gCompraGov(reader)?),
                b"gPagAntecipado" => ide.gPagAntecipado = Some(parse_gPagAntecipado(reader)?),

                name => {
                    let txt: String = read_text_string(reader, &e)?;
                    match name {
                        b"cUF" => ide.cUF = txt.parse()?,
                        b"cNF" => ide.cNF = txt,
                        b"natOp" => ide.natOp = txt,
                        b"mod" => ide.r#mod = txt.parse::<u8>()?,
                        b"serie" => ide.serie = txt.parse::<u16>()?,
                        b"nNF" => ide.nNF = txt.parse::<u32>()?,
                        b"dhEmi" => ide.dhEmi = txt,
                        b"dhSaiEnt" => ide.dhSaiEnt = Some(txt),
                        b"tpNF" => ide.tpNF = txt == "1",
                        b"idDest" => ide.idDest = txt.parse::<u8>()?,
                        b"cMunFG" => ide.cMunFG = txt.parse::<u32>()?,
                        b"cMunFGIBS" => ide.cMunFGIBS = Some(txt.parse::<u32>()?),
                        b"tpImp" => ide.tpImp = txt.parse::<u8>()?,
                        b"tpEmis" => ide.tpEmis = txt.parse::<u8>()?,
                        b"cDV" => ide.cDV = txt.parse::<u8>()?,
                        b"tpAmb" => ide.tpAmb = txt.parse::<u8>()?,
                        b"finNFe" => ide.finNFe = txt.parse::<u8>()?,
                        b"tpNFDebito" => ide.tpNFDebito = Some(txt.parse::<u8>()?),
                        b"tpNFCredito" => ide.tpNFCredito = Some(txt.parse::<u8>()?),
                        b"indFinal" => ide.indFinal = txt == "1",
                        b"indPres" => ide.indPres = txt.parse::<u8>()?,
                        b"indIntermed" => ide.indIntermed = Some(txt == "1"),
                        b"procEmi" => ide.procEmi = txt.parse::<u8>()?,
                        b"verProc" => ide.verProc = txt,
                        b"dhCont" => ide.dhCont = Some(txt),
                        b"xJust" => ide.xJust = Some(txt),
                        _ => {log::warn!("Elemento ide não mapeado: {}", std::str::from_utf8(name.as_ref())?)}
                    }
                }
            }

            Ok(Event::End(e)) if e.name().as_ref() == b"ide" => {
                return Ok(ide);
            }

            Ok(Event::Eof) => {
                return Err(Box::new(ParseError::UnexpectedEof("ide".to_string())));
            }

            Err(e) => {
                log::error!("Error reading XML at Ide: {}", e);
                break;
            }
            _ => { }
        }
    }
    return Err(Box::<dyn Error>::from("Unexpected error while parsing ide."));
}

fn parse_emit(reader: &mut XmlReader) -> Result<Emit, Box<dyn Error>> {
    let mut emit: Emit = Emit::default();

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => match e.name().as_ref() {
                b"enderEmit" => emit.enderEmit = parse_enderEmit(reader)?,
            
                name => {
                    let txt: String = read_text_string(reader, &e)?;
                    match name {
                        b"CNPJ" => emit.EmitenteId = EmitenteId::CNPJ(txt),
                        b"CPF" => emit.EmitenteId = EmitenteId::CPF(txt),
                        b"xNome" => emit.xNome = txt,
                        b"xFant" => emit.xFant = Some(txt),
                        b"IE" => emit.IE = txt,
                        b"IEST" => emit.IEST = Some(txt),
                        b"IM" => emit.IM = Some(txt),
                        b"CNAE" => emit.CNAE = Some(txt),
                        b"CRT" => emit.CRT = txt.parse::<u8>()?,

                        _ => {}
                    }
                }
            },

            Ok(Event::End(e)) if e.name().as_ref() == b"emit" => return Ok(emit),

            Ok(Event::Eof) => {
                log::error!("Unexpected Eof while parsing emit")
            }
            Err(e) => {
                log::error!("Error reading XML: {}", e);
                break;
            }
            _ => {}
        }
    }
    panic!("Unexpected error while parsing emit.")
}


fn parse_det(reader:&mut XmlReader) -> Result<Det, Box<dyn Error>> {
    let mut det: Det = Det::default();
    

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => match e.name().as_ref() {
                b"prod" => det.produto =  parse_prod(reader)?,

                b"imposto" => det.imposto = parse_imposto(reader)?,
                _ => {}
            }

            Ok(Event::End(e)) if e.name().as_ref() == b"det" => return Ok(det),

            Ok(Event::Eof) => return Err(Box::new(ParseError::UnexpectedEof("det".to_string()))),
            _ => {}
        }
    }
}

pub fn parse_prod(reader: &mut XmlReader) -> Result<Prod, Box<dyn Error>> {
    let mut prod: Prod = Prod::default();

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => match e.name().as_ref() {
                b"gCred" => prod.gCred.get_or_insert_default().push(parse_gCred(reader)?),
                b"DI" => prod.DI.get_or_insert_default().push(parse_DI(reader)?),
                b"detExport" => prod.detExport.get_or_insert_default().push(parse_detExport(reader)?),
                b"infProdNFF" => prod.infProdNFF = Some(parse_infProdNFF(reader)?),
                b"veicProd" => prod.especifico = Some(ProdutoEspecifico::veicProd(parse_veicProd(reader)?)),
                b"med" => prod.especifico = Some(ProdutoEspecifico::med(parse_med(reader)?)),
                b"arma" => {
                    let arma_parseada: Arma = parse_arma(reader)?;
                    if let Some(ProdutoEspecifico::arma(ref mut vec)) = prod.especifico {
                        vec.push(arma_parseada);
                    } else {
                        prod.especifico = Some(ProdutoEspecifico::arma(vec![arma_parseada]));
                    }
                },
                b"comb" => prod.especifico = Some(ProdutoEspecifico::comb(parse_comb(reader)?)),

                name => {
                    let txt: String = read_text_string(reader, &e)?;
                    match name {
                        b"cProd" => prod.cProd = txt,
                        b"cEAN" => prod.cEAN = txt,
                        b"cBarra" => prod.cBarra = Some(txt),
                        b"xProd" => prod.xProd = txt,
                        b"NCM" => prod.NCM = txt,
                        b"NVE" => prod.NVE.get_or_insert_with(Vec::new).push(txt),
                        b"CEST" => prod.CEST = Some(txt),
                        b"indEscala" => prod.indEscala = Some(txt),
                        b"CNPJFab" => prod.CNPJFab = Some(txt),
                        b"cBenef" => prod.cBenef = Some(txt),
                        b"EXTIPI" => prod.EXTIPI = Some(txt),
                        b"CFOP" => prod.CFOP = txt,
                        b"uCom" => prod.uCom = txt,
                        b"qCom" => prod.qCom = txt.parse::<Decimal>()?,
                        b"vUnCom" => prod.vUnCom = txt.parse::<Decimal>()?,
                        b"vProd" => prod.vProd = txt.parse::<Decimal>()?,
                        b"cEANTrib" => prod.cEANTrib = txt,
                        b"cBarraTrib" => prod.cBarraTrib = Some(txt),
                        b"uTrib" => prod.uTrib = txt,
                        b"qTrib" => prod.qTrib = txt.parse::<Decimal>()?,
                        b"vUnTrib" => prod.vUnTrib = txt.parse::<Decimal>()?,
                        b"vFrete" => prod.vFrete = Some(txt.parse::<Decimal>()?),
                        b"vSeg" => prod.vSeg = Some(txt.parse::<Decimal>()?),
                        b"vDesc" => prod.vDesc = Some(txt.parse::<Decimal>()?),
                        b"vOutro" => prod.vOutro = Some(txt.parse::<Decimal>()?),
                        b"indTot" => prod.indTot = txt == "1",
                        b"indBemMovelUsado" => prod.indBemMovelUsado = Some(true),
                        b"xPed" => prod.xPed = Some(txt),
                        b"nItemPed" => prod.nItemPed = Some(txt),
                        b"nFCI" => prod.nFCI = Some(txt),
                        b"rastro" => {} // Não tem Struct
                        b"nRECOPI" => prod.especifico = Some(ProdutoEspecifico::nRECOPI(txt)),
                        _ => {}
                    }
                }
            }

            Ok(Event::End(e)) if e.name().as_ref() == b"prod" => return Ok(prod),

            Ok(Event::Eof) => return Err(Box::new(ParseError::UnexpectedEof("prod".to_string()))),

            _ => {}
        }
    }
}

#[allow(non_snake_case)]
fn parse_gCred(reader: &mut XmlReader) -> Result<GCred, Box<dyn Error>> {
    let mut gCred: GCred = GCred::default();

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let txt = read_text_string(reader, &e)?;

                match e.name().as_ref() {
                    b"cCredPresumido" => gCred.cCredPresumido = txt,
                    b"pCredPresumido" => gCred.pCredPresumido = txt.parse::<Decimal>()?,
                    b"vCredPresumido" => gCred.vCredPresumido = txt.parse::<Decimal>()?,

                    _ => {}
                }
                
            }
            
            Ok(Event::End(e)) if e.name().as_ref() == b"gCred" => return Ok(gCred),

            Ok(Event::Eof) => return Err(Box::new(ParseError::UnexpectedEof("gCred".to_string()))),

            _ => {}
        }
    }
}

#[allow(non_snake_case)]
fn parse_DI(reader: &mut XmlReader) -> Result<DI, Box<dyn Error>> {
    let mut DI: DI = DI::default();

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => match e.name().as_ref() {
                b"adi" => DI.adi.push(parse_adi(reader)?),

                name => {
                    let txt: String = read_text_string(reader, &e)?;
                    match name {
                        b"nDI" => DI.nDI = txt,
                        b"dDI" => DI.dDI = txt,
                        b"xLocDesemb" => DI.xLocDesemb = txt,
                        b"UFDesemb" => DI.UFDesemb = UF::from(txt.as_str()),
                        b"dDesemb" => DI.dDesemb = txt,
                        b"tpViaTransp" => DI.tpViaTransp = txt.parse::<u8>()?,
                        b"vAFRMM" => DI.vAFRMM = Some(txt.parse::<Decimal>()?),
                        b"tpIntermedio" => DI.tpIntermedio = txt.parse::<u8>()?,
                        b"CNPJ" => DI.EmitenteId = EmitenteId::CNPJ(txt),
                        b"CPF" => DI.EmitenteId = EmitenteId::CPF(txt),
                        b"UFTerceiro" => DI.UFTerceiro = Some(UF::from(txt.as_str())),
                        b"cExportador" => DI.cExportador = Some(txt),
                        _ => {}
                    }
                }
            }

            Ok(Event::End(e)) if e.name().as_ref() == b"DI" => return Ok(DI),

            Ok(Event::Eof) => return Err(Box::new(ParseError::UnexpectedEof("DI".to_string()))),
            _ => {}
        }
    }
}

#[allow(non_snake_case)]
fn parse_detExport(reader: &mut XmlReader) -> Result<DetExport, Box<dyn Error>> {
    let mut detExport: DetExport = DetExport::default();
    

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => match e.name().as_ref() {
                b"exportInd" => {
                    let mut exportInd: ExportInd = ExportInd::default();
                    loop {
                        match reader.read_event() {
                            Ok(Event::Start(e)) => {
                                let txt: String = read_text_string(reader, &e)?;
                                match e.name().as_ref() {
                                    b"nRE" => exportInd.nRE = txt,
                                    b"chNFe" => exportInd.chNFe = txt,
                                    b"qExport" => exportInd.qExport = txt.parse::<Decimal>()?,
                                    _ => break
                                }
                            }
                            Ok(Event::End(e)) if e.name().as_ref() == b"exportInd" => {
                                detExport.exportInd = Some(exportInd);
                                break;
                            }
                            Ok(Event::Eof) => return Err(Box::new(ParseError::UnexpectedEof("exportInd".to_string()))),
                            _ => {}
                        }
                    }
                }
                name => {
                    let txt: String = read_text_string(reader, &e)?;
                    match name {
                        b"nDraw" => detExport.nDraw = Some(txt),
                        _ => {}
                    }
                }
            }

            Ok(Event::End(e)) if e.name().as_ref() == b"detExport" => return Ok(detExport),

            Ok(Event::Eof) => return Err(Box::new(ParseError::UnexpectedEof("detExport".to_string()))),
            _ => {}
        }
    }
}

#[allow(non_snake_case)]
fn parse_infProdNFF(reader: &mut XmlReader) -> Result<InfProdNFF, Box<dyn Error>> {
    let mut infProdNFF: InfProdNFF = InfProdNFF::default();

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let txt = read_text_string(reader, &e)?;

                match e.name().as_ref() {
                    b"cProdFisco" => infProdNFF.cProdFisco = txt,
                    b"cOperNFF" => infProdNFF.cOperNFF = txt,
                    _ => {}
                }
            }
            
            Ok(Event::End(e)) if e.name().as_ref() == b"infProdNFF" => return Ok(infProdNFF),

            Ok(Event::Eof) => return Err(Box::new(ParseError::UnexpectedEof("infProdNFF".to_string()))),

            _ => {}
        }
    }
}

#[allow(non_snake_case)]
fn parse_infProdEmb(reader: &mut XmlReader) -> Result<InfProdEmb, Box<dyn Error>> {
    let mut infProdEmb: InfProdEmb = InfProdEmb::default();

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let txt = read_text_string(reader, &e)?;

                match e.name().as_ref() {
                    b"xEmb" => infProdEmb.xEmb = txt,
                    b"qVolEmb" => infProdEmb.qVolEmb = txt.parse::<Decimal>()?,
                    b"uEmb" => infProdEmb.uEmb = txt,
                    _ => {}
                }
            }
            
            Ok(Event::End(e)) if e.name().as_ref() == b"infProdEmb" => return Ok(infProdEmb),

            Ok(Event::Eof) => return Err(Box::new(ParseError::UnexpectedEof("infProdEmb".to_string()))),

            _ => {}
        }
    }
}

#[allow(non_snake_case)]
fn parse_veicProd(reader: &mut XmlReader) -> Result<Veiculo, Box<dyn Error>> {
        let mut veicProd: Veiculo = Veiculo::default();

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let txt = read_text_string(reader, &e)?;

                match e.name().as_ref() {
                    b"tpOp" => veicProd.tpOp = txt.parse::<u8>()?,
                    b"chassi" => veicProd.chassi = txt,
                    b"cCor" => veicProd.cCor = txt,
                    b"xCor" => veicProd.xCor = txt,
                    b"pot" => veicProd.pot = txt,
                    b"cilin" => veicProd.cilin = txt,
                    b"pesoL" => veicProd.pesoL = txt,
                    b"pesoB" => veicProd.pesoB = txt,
                    b"nSerie" => veicProd.nSerie = txt,
                    b"tpComb" => veicProd.tpComb = txt.parse::<u8>()?,
                    b"nMotor" => veicProd.nMotor = txt,
                    b"CMT" => veicProd.CMT = txt,
                    b"dist" => veicProd.dist = txt,
                    b"anoMod" => veicProd.anoMod = txt,
                    b"anoFab" => veicProd.anoFab = txt,
                    b"tpPint" => veicProd.tpPint = txt,
                    b"tpVeic" => veicProd.tpVeic = txt,
                    b"espVeic" => veicProd.espVeic = txt,
                    b"VIN" => veicProd.VIN = txt.parse::<char>()?,
                    b"condVeic" => veicProd.condVeic = txt,
                    b"cMod" => veicProd.cMod = txt,
                    b"cCorDENATRAN" => veicProd.cCorDENATRAN = txt,
                    b"lota" => veicProd.lota = txt.parse::<u32>()?,
                    b"tpRest" => veicProd.tpRest = txt.parse::<u8>()?,
                    _ => {}
                }
                
            }
            
            Ok(Event::End(e)) if e.name().as_ref() == b"veicProd" => return Ok(veicProd),

            Ok(Event::Eof) => return Err(Box::new(ParseError::UnexpectedEof("veicProd".to_string()))),

            _ => {}
        }
    }
}

#[allow(non_snake_case)]
fn parse_med(reader: &mut XmlReader) -> Result<Medicamento, Box<dyn Error>> {
    let mut med = Medicamento::default();

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let txt = read_text_string(reader, &e)?;

                match e.name().as_ref() {
                    b"cProdANVISA" => med.cProdANVISA = txt,
                    b"xMotivoIsencao" => med.xMotivoIsencao = Some(txt),
                    b"vPMC" => med.vPMC = txt.parse::<Decimal>()?,
                    _ => (),
                }
            }
            Ok(Event::End(e)) if e.name().as_ref() == b"med" => return Ok(med),
            Ok(Event::Eof) => return Err(Box::new(ParseError::UnexpectedEof("med".to_string()))),
            _ => (),
        }
    }
}

#[allow(non_snake_case)]
fn parse_arma(reader: &mut XmlReader) -> Result<Arma, Box<dyn Error>> {
    let mut arma = Arma::default();

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let txt = read_text_string(reader, &e)?;
                match e.name().as_ref() {
                    b"tpArma" => arma.tpArma = txt.parse()?,
                    b"nSerie" => arma.nSerie = txt,
                    b"nCano" => arma.nCano = txt,
                    b"descr" => arma.descr = txt,
                    _ => (),
                }
            }

            Ok(Event::End(e)) if e.name().as_ref() == b"arma" => return Ok(arma),
            
            Ok(Event::Eof) => return Err(Box::new(ParseError::UnexpectedEof("arma".to_string()))),
            
            _ => (),
        }
    }
}

#[allow(non_snake_case)]
fn parse_comb(reader: &mut XmlReader) -> Result<Combustivel, Box<dyn Error>> {
    let mut combustivel = Combustivel::default();

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => match e.name().as_ref() {
                // Tags que são grupos aninhados
                b"CIDE" => combustivel.CIDE = Some(parse_cide(reader)?),
                b"encerrante" => combustivel.encerrante = Some(parse_encerrante(reader)?),
                b"origComb" => {
                    let orig = parse_orig_comb(reader)?;
                    combustivel.origComb.get_or_insert_default().push(orig);
                }

                // Tags com valores simples
                name => {
                    let txt = read_text_string(reader, &e)?;
                    match name {
                        b"cProdANP" => combustivel.cProdANP = txt,
                        b"descANP" => combustivel.descANP = txt,
                        b"pGLP" => combustivel.pGLP = Some(txt.parse()?),
                        b"pGNn" => combustivel.pGNn = Some(txt.parse()?),
                        b"pGNi" => combustivel.pGNi = Some(txt.parse()?),
                        b"vPart" => combustivel.vPart = Some(txt.parse()?),
                        b"CODIF" => combustivel.CODIF = Some(txt),
                        b"qTemp" => combustivel.qTemp = Some(txt.parse()?),
                        b"UFCons" => combustivel.UFCons = UF::from(txt.as_str()),
                        b"pBio" => combustivel.pBio = Some(txt.parse()?),
                        _ => (),
                    }
                }
            },
            Ok(Event::End(e)) if e.name().as_ref() == b"comb" => return Ok(combustivel),
            Ok(Event::Eof) => return Err(Box::new(ParseError::UnexpectedEof("comb".to_string()))),
            _ => (),
        }
    }
}

// --- Funções Auxiliares ---

#[allow(non_snake_case)]
fn parse_cide(reader: &mut XmlReader) -> Result<Cide, Box<dyn Error>> {
    let mut cide: Cide = Cide::default();
    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let txt = read_text_string(reader, &e)?;
                match e.name().as_ref() {
                    b"qBCProd" => cide.qBCProd = txt.parse()?,
                    b"vAliqProd" => cide.vAliqProd = txt.parse()?,
                    b"vCIDE" => cide.vCIDE = txt.parse()?,
                    _ => (),
                }
            }
            Ok(Event::End(e)) if e.name().as_ref() == b"CIDE" => return Ok(cide),
            Ok(Event::Eof) => return Err(Box::new(ParseError::UnexpectedEof("CIDE".to_string()))),
            _ => (),
        }
    }
}

#[allow(non_snake_case)]
fn parse_encerrante(reader: &mut XmlReader) -> Result<Encerrante, Box<dyn Error>> {
    let mut encerrante: Encerrante = Encerrante::default();
    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let txt = read_text_string(reader, &e)?;
                match e.name().as_ref() {
                    b"nBico" => encerrante.nBico = txt.parse()?,
                    b"nBomba" => encerrante.nBomba = Some(txt.parse()?),
                    b"nTanque" => encerrante.nTanque = txt.parse()?,
                    b"vEncIni" => encerrante.vEncIni = txt.parse()?,
                    b"vEncFin" => encerrante.vEncFin = txt.parse()?,
                    _ => (),
                }
            }
            Ok(Event::End(e)) if e.name().as_ref() == b"encerrante" => return Ok(encerrante),
            Ok(Event::Eof) => return Err(Box::new(ParseError::UnexpectedEof("encerrante".to_string()))),
            _ => (),
        }
    }
}

#[allow(non_snake_case)]
fn parse_orig_comb(reader: &mut XmlReader) -> Result<OrigComb, Box<dyn Error>> {
    let mut orig: OrigComb = OrigComb::default();
    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let txt = read_text_string(reader, &e)?;
                match e.name().as_ref() {
                    b"indImport" => orig.indImport = txt.parse()?,
                    b"cUFOrig" => orig.cUFOrig = txt.parse::<u8>()?,
                    b"pOrig" => orig.pOrig = txt.parse()?,
                    _ => (),
                }
            }
            Ok(Event::End(e)) if e.name().as_ref() == b"origComb" => return Ok(orig),
            Ok(Event::Eof) => return Err(Box::new(ParseError::UnexpectedEof("origComb".to_string()))),
            _ => (),
        }
    }
}
fn parse_adi(reader: &mut XmlReader) -> Result<Adi, Box<dyn Error>> {
    let mut adi: Adi = Adi::default();
    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let txt = read_text_string(reader, &e)?;

                match e.name().as_ref() {
                    b"nAdicao" => adi.nAdicao = Some(txt),
                    b"nSeqAdic" => adi.nSeqAdic = Some(txt),
                    b"cFabricante" => adi.cFabricante = txt,
                    b"vDescDI" => adi.vDescDI = Some(txt.parse::<Decimal>()?),
                    b"nDraw" => adi.nDraw = Some(txt),
                    _ => {}
                }
                
            }
            
            Ok(Event::End(e)) if e.name().as_ref() == b"adi" => return Ok(adi),

            Ok(Event::Eof) => return Err(Box::new(ParseError::UnexpectedEof("adi".to_string()))),
            _ => {}
        }
    }
}
fn parse_imposto(reader: &mut XmlReader) -> Result<Imposto, Box<dyn Error>> {
    let mut imposto: Imposto = Imposto::default();

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => match e.name().as_ref() {
                b"ICMS" => imposto.tributacao = Some(Tributacao::Mercadoria(parse_mercadoria(reader)?)),

                name => {
                    let txt: String = read_text_string(reader, &e)?;
                    match name {
                        b"vTotTrib" => imposto.vTotTrib = Some(txt.parse::<Decimal>()?),
                        _ => {}
                    }
                }
            }

            Ok(Event::End(e)) if e.name().as_ref() == b"imposto" => return Ok(imposto),

            Ok(Event::Eof) => return Err(Box::new(ParseError::UnexpectedEof("imposto".to_string()))),

            _ => {}
        }
    }
}

fn parse_mercadoria(reader: &mut XmlReader) -> Result<TributosMercadoria, Box<dyn Error>> {
    let mut mercadoria: TributosMercadoria = TributosMercadoria::default();

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => match e.name().as_ref() {
                b"ICMS" => mercadoria.ICMS = parse_ICMS(reader)?,
                b"IPI" => mercadoria.IPI = Some(parse_IPI(reader)?),
                b"II" => todo!(),

                _ => {
                    return Err((Box::new(ParseError::ModeloDesconhecido)));
                }
            }

            Ok(Event::End(e)) if e.name().as_ref() == b"ICMS" => return Ok(mercadoria),

            Ok(Event::Eof) => return Err(Box::new(ParseError::UnexpectedEof("ICMS".to_string()))),

            _ => {}
        }
    }
}

#[allow(non_snake_case)]
fn parse_ICMS(reader: &mut XmlReader) -> Result<Icms, Box<dyn Error>> {
    let mut ICMS: Icms = Icms::default();
    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let txt: String = read_text_string(reader, &e)?;
                match e.name().as_ref() {
                    // --- CAMPOS DE IDENTIFICAÇÃO ---
                    b"orig" => ICMS.orig = Some(txt),
                    b"CST" => ICMS.cst = Some(txt),
                    b"CSOSN" => ICMS.csosn = Some(txt),

                    // --- CÁLCULO ICMS NORMAL ---
                    b"modBC" => ICMS.modBC = Some(txt),
                    b"vBC" => ICMS.vBC = Some(txt.parse()?),
                    b"pRedBC" => ICMS.pRedBC = Some(txt.parse()?),
                    b"pICMS" => ICMS.pICMS = Some(txt.parse()?),
                    b"vICMS" => ICMS.vICMS = Some(txt.parse()?),
                    b"vICMSOp" => ICMS.vICMSOp = Some(txt.parse()?),

                    // --- FCP (Fundo de Combate à Pobreza) ---
                    b"vBCFCP" => ICMS.vBCFCP = Some(txt.parse()?),
                    b"pFCP" => ICMS.pFCP = Some(txt.parse()?),
                    b"vFCP" => ICMS.vFCP = Some(txt.parse()?),

                    // --- ICMS ST (CÁLCULO NA OPERAÇÃO) ---
                    b"modBCST" => ICMS.modBCST = Some(txt),
                    b"pMVAST" => ICMS.pMVAST = Some(txt.parse()?),
                    b"pRedBCST" => ICMS.pRedBCST = Some(txt.parse()?),
                    b"vBCST" => ICMS.vBCST = Some(txt.parse()?),
                    b"pICMSST" => ICMS.pICMSST = Some(txt.parse()?),
                    b"vICMSST" => ICMS.vICMSST = Some(txt.parse()?),
                    
                    // --- FCP ST ---
                    b"vBCFCPST" => ICMS.vBCFCPST = Some(txt.parse()?),
                    b"pFCPST" => ICMS.pFCPST = Some(txt.parse()?),
                    b"vFCPST" => ICMS.vFCPST = Some(txt.parse()?),

                    // --- ICMS ST RETIDO (OPERAÇÃO ANTERIOR) ---
                    b"vBCSTRet" => ICMS.vBCSTRet = Some(txt.parse()?),
                    b"pST" => ICMS.pST = Some(txt.parse()?),
                    b"vICMSSubstituto" => ICMS.vICMSSubstituto = Some(txt.parse()?),
                    b"vICMSSTRet" => ICMS.vICMSSTRet = Some(txt.parse()?),
                    
                    // --- FCP ST RETIDO ---
                    b"vBCFCPSTRet" => ICMS.vBCFCPSTRet = Some(txt.parse()?),
                    b"pFCPSTRet" => ICMS.pFCPSTRet = Some(txt.parse()?),
                    b"vFCPSTRet" => ICMS.vFCPSTRet = Some(txt.parse()?),

                    // --- ICMS DESONERADO ---
                    b"vICMSDeson" => ICMS.vICMSDeson = Some(txt.parse()?),
                    b"motDesICMS" => ICMS.motDesICMS = Some(txt),
                    b"indDeduzDeson" => ICMS.indDeduzDeson = Some(txt == "1"),
                    
                    // --- ICMS ST DESONERADO ---
                    b"vICMSSTDeson" => ICMS.vICMSSTDeson = Some(txt.parse()?),
                    b"motDesICMSST" => ICMS.motDesICMSST = Some(txt),

                    // --- ICMS EFETIVO ---
                    b"pRedBCEfet" => ICMS.pRedBCEfet = Some(txt.parse()?),
                    b"vBCEfet" => ICMS.vBCEfet = Some(txt.parse()?),
                    b"pICMSEfet" => ICMS.pICMSEfet = Some(txt.parse()?),
                    b"vICMSEfet" => ICMS.vICMSEfet = Some(txt.parse()?),

                    // --- ICMS DIFERIMENTO ---
                    b"pDif" => ICMS.pDif = Some(txt.parse()?),
                    b"vICMSDif" => ICMS.vICMSDif = Some(txt.parse()?),
                    b"cBenefRBC" => ICMS.cBenefRBC = Some(txt),
                    b"pFCPDif" => ICMS.pFCPDif = Some(txt.parse()?),
                    b"vFCPDif" => ICMS.vFCPDif = Some(txt.parse()?),
                    b"vFCPEfet" => ICMS.vFCPEfet = Some(txt.parse()?),

                    // --- ICMS MONOFÁSICO ---
                    b"qBCMono" => ICMS.qBCMono = Some(txt.parse()?),
                    b"adRemICMS" => ICMS.adRemICMS = Some(txt.parse()?),
                    b"vICMSMono" => ICMS.vICMSMono = Some(txt.parse()?),
                    b"qBCMonoReten" => ICMS.qBCMonoReten = Some(txt.parse()?),
                    b"adRemICMSReten" => ICMS.adRemICMSReten = Some(txt.parse()?),
                    b"vICMSMonoReten" => ICMS.vICMSMonoReten = Some(txt.parse()?),
                    b"pRedAdRem" => ICMS.pRedAdRem = Some(txt.parse()?),
                    b"motRedAdRem" => ICMS.motRedAdRem = Some(txt),
                    b"qBCMonoRet" => ICMS.qBCMonoRet = Some(txt.parse()?),
                    b"adRemICMSRet" => ICMS.adRemICMSRet = Some(txt.parse()?),
                    b"vICMSMonoRet" => ICMS.vICMSMonoRet = Some(txt.parse()?),
                    b"vICMSMonoOp" => ICMS.vICMSMonoOp = Some(txt.parse()?),
                    b"vICMSMonoDif" => ICMS.vICMSMonoDif = Some(txt.parse()?),
                    b"qBCMonoDif" => ICMS.qBCMonoDif = Some(txt.parse()?),
                    b"adRemICMSDif" => ICMS.adRemICMSDif = Some(txt.parse()?),

                    // --- ICMS PARTILHA ---
                    b"pBCOp" => ICMS.pBCOp = Some(txt.parse()?),
                    b"UFST" => ICMS.ufst = Some(UF::from(txt.as_str())),

                    // --- ICMS ST (REPASSE) ---
                    b"vBCSTDest" => ICMS.vBCSTDest = Some(txt.parse()?),
                    b"vICMSSTDest" => ICMS.vICMSSTDest = Some(txt.parse()?),

                    // --- SIMPLES NACIONAL (CRÉDITO) ---
                    b"pCredSN" => ICMS.pCredSN = Some(txt.parse()?),
                    b"vCredICMSSN" => ICMS.vCredICMSSN = Some(txt.parse()?),
                    
                    _ => {}
                }

            }

            Ok(Event::End(e)) if e.name().as_ref() == b"ICMS" => return Ok(ICMS),

            Ok(Event::Eof) => return Err(Box::new(ParseError::UnexpectedEof("ICMS".to_string()))),

            _ => {}
        }
    }
}
#[allow(non_snake_case)]
fn parse_IPI(reader: &mut XmlReader) -> Result<Ipi, Box<dyn Error>> {
    let mut ipi: Ipi = Ipi::default();
    todo!();
    
    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => match e.name().as_ref() {
                //b"ICMS" => imposto.tributacao = parse_ICMS(reader)?,
                //b"IPI" => imposto.tributacao = parse_IPI(reader)?,

                name => {
                    let txt: String = read_text_string(reader, &e)?;
                    match name {
                        //b"vTotTrib" => imposto.vTotTrib = Some(txt.parse::<Decimal>()?),
                        _ => {}
                    }
                }
            }

            Ok(Event::End(e)) if e.name().as_ref() == b"ICMS" => return Ok(prod),

            Ok(Event::Eof) => return Err(Box::new(ParseError::UnexpectedEof("ICMS".to_string()))),

            _ => {}
        }
    }
}
#[allow(non_snake_case)]
fn parse_enderEmit(reader: &mut XmlReader) -> Result<EnderEmi, Box<dyn Error>> {
    let mut enderEmi: EnderEmi = EnderEmi::default();
    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let name = e.name();
                let txt: String = read_text_string(reader, &e)?;

                match name.as_ref() {
                    b"xLgr" => enderEmi.xLgr = txt,
                    b"nro" => enderEmi.nro = txt,
                    b"xCpl" => enderEmi.xCpl = Some(txt),
                    b"xBairro" => enderEmi.xBairro = txt,
                    b"cMun" => enderEmi.cMun = txt.parse::<u32>()?,
                    b"xMun" => enderEmi.xMun = txt,
                    b"UF" => enderEmi.UF = UF::from(txt.as_str()),
                    b"CEP" => enderEmi.CEP = txt,
                    b"cPais" => enderEmi.cPais = Some(txt),
                    b"xPais" => enderEmi.xPais = Some(txt),
                    b"fone" => enderEmi.fone = Some(txt),
                    _ => {}
                }
            }

            Ok(Event::End(e)) if e.name().as_ref() == b"enderEmit" => return Ok(enderEmi),

            Ok(Event::Eof) => {
                log::error!("Unexpected Eof while parsing EnderEmi");
                break;
            }

            _ => {}
        }
    }
    panic!("Unexpected error while parsing EnderEmi.");
}

fn parse_nfref(reader: &mut XmlReader) -> Result<NFRef, Box<dyn Error>> {
    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => match e.name().as_ref() {
                b"refNF" => {
                    return parse_refNF(reader)
                },
                b"refNFP" => {
                    return parse_refNFP(reader)
                },
                b"refECF" => {
                    return parse_refECF(reader)
                }

                name => {
                    let txt: String = read_text_string(reader, &e)?;
                    match name {
                        b"refNFe" => return Ok(NFRef::refNFe(txt)),
                        b"refNFeSig" => return Ok(NFRef::refNFeSig(txt)),
                        b"refCTe" => return Ok(NFRef::refCTe(txt)),
                        _ => {break;} // Desconhecido. Forçar erro
                    }
                }
            }

            Ok(Event::Eof) =>  return Err(Box::new(ParseError::UnexpectedEof("NFref".to_string()))),
            _ => {}
        }
    }
    panic!("Unexpected error while parsing NFRef.");
}

#[allow(non_snake_case)]
fn parse_refNF(reader: &mut XmlReader) -> Result<NFRef, Box<dyn Error>> {
    let mut refNF: RefNFData = RefNFData::default();
    
    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let txt = read_text_string(reader, &e)?;
                match e.name().as_ref() {
                    b"cUF" => refNF.cUF = txt.parse::<u8>()?,
                    b"AAMM" => refNF.AAMM = txt,
                    b"CNPJ" => refNF.CNPJ = txt,
                    b"mod" => refNF.r#mod = txt.parse::<u8>()?,
                    b"serie" => refNF.serie = txt.parse::<u16>()?,
                    b"nNF" => refNF.nNF = txt.parse::<u32>()?,
                    _ => {break;}
                }
            }

            Ok(Event::End(e)) if e.name().as_ref() == b"refNF" => {
                return Ok(NFRef::refNF(refNF));
            }

            Ok(Event::Eof) => {
                log::error!("Unexpected Eof while parsing refNF");
                break;
            }
            _ => {}
        }
    }
    panic!("Unexpected error while parsing refNF.");
}


#[allow(non_snake_case)]
fn parse_refNFP(reader: &mut XmlReader) -> Result<NFRef, Box<dyn Error>> {
        let mut refNFP: RefNFPData = RefNFPData::default();
    
    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let txt = read_text_string(reader, &e)?;
                match e.name().as_ref() {
                    b"cUF" => refNFP.cUF = txt.parse::<u8>()?,
                    b"AAMM" => refNFP.AAMM = txt,
                    b"CNPJ" => refNFP.EmitenteId = EmitenteId::CNPJ(txt),
                    b"CPF" => refNFP.EmitenteId = EmitenteId::CPF(txt),
                    b"IE" => refNFP.IE = txt,
                    b"mod" => refNFP.r#mod = txt.parse::<u8>().unwrap(),
                    b"serie" => refNFP.serie = txt.parse::<u16>().unwrap(),
                    b"nNF" => refNFP.nNF = txt.parse::<u32>().unwrap(),
                    _ => {break;}
                }
            }

            Ok(Event::End(e)) if e.name().as_ref() == b"refNFP" => {
                return Ok(NFRef::refNFP(refNFP));
            }

            Ok(Event::Eof) => {
                log::error!("Unexpected Eof while parsing refNFP");
                break;
            }
            _ => {}
        }
    }
    panic!("Unexpected error while parsing refNFP.");
}

#[allow(non_snake_case)]
fn parse_refECF(reader: &mut XmlReader) -> Result<NFRef, Box<dyn Error>> {
    let mut refECF: RefECFData = RefECFData::default();
    
    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let txt = read_text_string(reader, &e)?;
                match e.name().as_ref() {
                    b"mod" => refECF.r#mod = txt,
                    b"nECF" => refECF.nECF = txt,
                    b"nCOO" => refECF.nCOO = txt,
                    _ => {break;}
                }
            }

            Ok(Event::End(e)) if e.name().as_ref() == b"refECF" => {
                return Ok(NFRef::refECF(refECF));
            }

            Ok(Event::Eof) => {
                log::error!("Unexpected Eof while parsing refECF");
                break;
            }
            _ => {}
        }
    }
    panic!("Unexpected error while parsing refECF.");
}

#[allow(non_snake_case)]
pub fn parse_gCompraGov(reader: &mut XmlReader) -> Result<CompraGov, Box<dyn Error>> {
    let mut cg: CompraGov = CompraGov::default();

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let name = e.name();
                let txt: String = read_text_string(reader, &e)?;

                match name.as_ref() {
                    b"tpEnteGov" => cg.tpEnteGov = txt.parse()?,
                    b"pRedutor" => cg.pRedutor = txt.parse()?,
                    b"tpOperGov" => cg.tpOperGov = txt.parse()?,
                    _ => {
                        log::warn!("Elemento CompraGov não mapeado: {}", std::str::from_utf8(e.name().as_ref())?);
                        break;
                    }
                }
            }

            Ok(Event::End(e)) if e.name().as_ref() == b"gCompraGov" => {
                return Ok(cg);
            }

            Ok(Event::Eof) => {
                return Err(Box::new(ParseError::UnexpectedEof("gCompraGov".to_string())));
            }

            Err(e) => log::error!("Error reading gCompraGov: {}", e),
            _ => {}
        }
    }
    panic!("Unexpected error while parsing gCompraGov.");
}


#[allow(non_snake_case)]
pub fn parse_gPagAntecipado(reader: &mut XmlReader) -> Result<Vec<String>, Box<dyn Error>> {
    let mut refNfes: Vec<String> = Vec::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => if e.name().as_ref() == b"refNFe" {
                refNfes.push(read_text_string(reader, &e)?);
            }

            // Tag terminou
            Ok(Event::End(e)) => if e.name().as_ref() == b"gPagAntecipado" {
                return Ok(refNfes);
            }

            Ok(Event::Eof) => {
                return Err(Box::new(ParseError::UnexpectedEof("gPagAntecipado".to_string())));
            }

            Err(e) => return Err(Box::new(e)),
            _ => {}
        }
    }
}

pub fn get_mod_nfe(xml_bytes: &String) -> Result<ModNfe, Box<dyn Error>> {
    let mut reader: Reader<&[u8]> = Reader::from_str(xml_bytes);
    reader.config_mut().trim_text(true);

    let mut inside_mod: bool = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) if e.name().as_ref() == b"mod" => {
                inside_mod = true;
            }

            Ok(Event::Text(e)) if inside_mod => {
                let txt: String = e.decode().unwrap().into_owned();
                return Ok(ModNfe::from(txt.as_str()));
            }

            Ok(Event::Eof) => break,
            Err(e) => return Err(Box::new(e)),
            _ => {}
        }
    }

    return Ok(ModNfe::Desconhecido);
}



fn get_nfe_id(e: &BytesStart) -> Result<String, ParseError> {
    for attr in e.attributes() {
        let attr = attr.map_err(|e| ParseError::Xml(e.to_string()))?;
        let key = String::from_utf8_lossy(attr.key.as_ref());
        let value = attr
            .unescape_value()
            .map_err(|e| ParseError::Xml(e.to_string()))?;
        if attr.key.as_ref() == b"Id" {
            let value = attr
                .unescape_value()
                .map_err(|e| ParseError::Xml(e.to_string()))?;
            return Ok(value.into_owned());
        }
    }
    Err(ParseError::Outros("Id não encontrado".to_string()))
}

#[inline]
fn read_text_string(reader: &mut XmlReader, e: &BytesStart) -> Result<String, Box<dyn Error>> {
    let txt = reader.read_text(e.name())?;
    Ok(txt.into_owned())
}