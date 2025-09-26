use quick_xml::{
    Reader,
    events::{BytesStart, Event},
};

use crate::nfes::{CompraGov, Emit, EmitenteId, EnderEmi, Ide, NFRef, NFe, NfeJson, RefECFData, RefNFData, RefNFPData, UF};

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

pub fn parse_nfe(xml_bytes: String) -> Option<String> {
    let modelo: ModNfe = get_mod_nfe(&xml_bytes);

    match modelo {
        ModNfe::Mod55 => return parse_nfe_mod_65(xml_bytes),
        ModNfe::Mod65 => return parse_nfe_mod_65(xml_bytes),
        ModNfe::Mod57 => return parse_nfe_mod_57(xml_bytes),
        ModNfe::Desconhecido => todo!(),
    }
}

pub fn parse_nfe_mod_65(xml: String) -> Option<String> {
    let mut reader: Reader<&[u8]> = Reader::from_str(&xml);
    reader.config_mut().trim_text(true);

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let name = e.name();

                match name.as_ref() {
                    b"enviNFe" => {
                        return parse_enviNfe_65(&mut reader);
                    }
                    b"nfeProc" => return parse_nfeProc_65(&mut reader),

                    _ => return None,
                }
            }

            _ => {}
        }
    }
}

#[allow(non_snake_case)]
fn parse_nfeProc_65(reader: &mut XmlReader) -> Option<String> {
    let mut json: NfeJson = NfeJson::default();

    let mut nfe: NFe = NFe::default();

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let name = e.name();
                match name.as_ref() {
                    b"ide" => nfe.ide = parse_ide(reader),
                    b"emit" => nfe.emit = parse_emit(reader),
                    _ => {}
                }
            }

            Ok(Event::Eof) => {
                break;
            }

            Err(_) => return None,

            _ => {}
        }
    }
    json.nfes.push(nfe);

    let maybe_json_string: String = serde_json::to_string(&json).expect("Failed to serialize struct");
    return Some(maybe_json_string);
}

#[allow(non_snake_case)]
fn parse_enviNfe_65(reader: &mut XmlReader) -> Option<String> {
    return None;
}

pub fn parse_nfe_mod_57(xml: String) -> Option<String> {
    return Some("MOD56-PARSED".to_string());
}

pub fn parse_ide(reader: &mut XmlReader) -> Ide {
    // Começa com uma struct com valores padrão
    let mut ide: Ide = Ide::default();

    // Transformar NFRef em Option<Vec<NFRef>>
    // Ver as nuâncias de como lidar com isso

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => match e.name().as_ref() {
                b"NFref" => {
                    let nfref: NFRef = parse_nfref(reader);
                    ide.NFref.get_or_insert_with(Vec::new).push(nfref);
                }

                b"gCompraGov" => {
                    ide.gCompraGov = Some(parse_gCompraGov(reader));
                }

                b"gPagAntecipado" => {
                    let refNFes: Vec<String> = parse_gPagAntecipado(reader);
                    ide.gPagAntecipado = Some(refNFes);
                }

                name => {
                    let txt: String = read_text_string(reader, &e);
                    match name {
                        b"cUF" => ide.cUF = txt.parse().unwrap(),
                        b"cNF" => ide.cNF = txt,
                        b"natOp" => ide.natOp = txt,
                        b"mod" => ide.r#mod = txt.parse::<u8>().unwrap(),
                        b"serie" => ide.serie = txt.parse::<u16>().unwrap(),
                        b"nNF" => ide.nNF = txt.parse::<u32>().unwrap(),
                        b"dhEmi" => ide.dhEmi = txt,
                        b"dhSaiEnt" => ide.dhSaiEnt = Some(txt),
                        b"tpNF" => ide.tpNF = txt == "1",
                        b"idDest" => ide.idDest = txt.parse::<u8>().unwrap(),
                        b"cMunFG" => ide.cMunFG = txt.parse::<u32>().unwrap(),
                        b"cMunFGIBS" => ide.cMunFGIBS = Some(txt.parse::<u32>().unwrap()),
                        b"tpImp" => ide.tpImp = txt.parse::<u8>().unwrap(),
                        b"tpEmis" => ide.tpEmis = txt.parse::<u8>().unwrap(),
                        b"cDV" => ide.cDV = txt.parse::<u8>().unwrap(),
                        b"tpAmb" => ide.tpAmb = txt.parse::<u8>().unwrap(),
                        b"finNFe" => ide.finNFe = txt.parse::<u8>().unwrap(),
                        b"tpNFDebito" => ide.tpNFDebito = Some(txt.parse::<u8>().unwrap()),
                        b"tpNFCredito" => ide.tpNFCredito = Some(txt.parse::<u8>().unwrap()),
                        b"indFinal" => ide.indFinal = txt == "1",
                        b"indPres" => ide.indPres = txt.parse::<u8>().unwrap(),
                        b"indIntermed" => ide.indIntermed = Some(txt == "1"),
                        b"procEmi" => ide.procEmi = txt.parse::<u8>().unwrap(),
                        b"verProc" => ide.verProc = txt,
                        b"dhCont" => ide.dhCont = Some(txt),
                        b"xJust" => ide.xJust = Some(txt),
                        _ => {log::warn!("Elemento ide não mapeado: {}", std::str::from_utf8(name.as_ref()).unwrap_or("<inválido>"))}
                    }
                }
            }

            Ok(Event::End(e)) if e.name().as_ref() == b"ide" => {
                return ide;
            }

            Ok(Event::Eof) => {
                log::error!("Unexpected Eof while parsing ide");
                break;
            }

            Err(e) => {
                log::error!("Error reading XML at Ide: {}", e);
                break;
            }
            _ => { }
        }
    }
    panic!("Unexpected error while parsing emit.")
}

fn parse_emit(reader: &mut XmlReader) -> Emit {
    let mut emit: Emit = Emit::default();

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => match e.name().as_ref() {
                b"enderEmit" => {
                    emit.enderEmit = parse_enderEmit(reader);
                }

                name => {
                    let txt: String = read_text_string(reader, &e);
                    match name {
                        b"CNPJ" => emit.EmitenteId = EmitenteId::CNPJ(txt),
                        b"CPF" => emit.EmitenteId = EmitenteId::CPF(txt),
                        b"xNome" => emit.xNome = txt,
                        b"xFant" => emit.xFant = Some(txt),
                        b"IE" => emit.IE = txt,
                        b"IEST" => emit.IEST = Some(txt),
                        b"IM" => emit.IM = Some(txt),
                        b"CNAE" => emit.CNAE = Some(txt),
                        b"CRT" => emit.CRT = txt.parse::<u8>().unwrap(),

                        _ => {}
                    }
                }
            },

            Ok(Event::End(e)) if e.name().as_ref() == b"emit" => return emit,

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

#[allow(non_snake_case)]
fn parse_enderEmit(reader: &mut XmlReader) -> EnderEmi {
    let mut enderEmi: EnderEmi = EnderEmi::default();
    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let name = e.name();
                let txt: String = read_text_string(reader, &e);

                match name.as_ref() {
                    b"xLgr" => enderEmi.xLgr = txt,
                    b"nro" => enderEmi.nro = txt,
                    b"xCpl" => enderEmi.xCpl = Some(txt),
                    b"xBairro" => enderEmi.xBairro = txt,
                    b"cMun" => enderEmi.cMun = txt.parse::<u32>().unwrap(),
                    b"xMun" => enderEmi.xMun = txt,
                    b"UF" => enderEmi.UF = UF::from(txt.as_str()),
                    b"CEP" => enderEmi.CEP = txt,
                    b"cPais" => enderEmi.cPais = Some(txt),
                    b"xPais" => enderEmi.xPais = Some(txt),
                    b"fone" => enderEmi.fone = Some(txt),
                    _ => {}
                }
            }

            Ok(Event::End(e)) if e.name().as_ref() == b"enderEmit" => return enderEmi,

            Ok(Event::Eof) => {
                log::error!("Unexpected Eof while parsing EnderEmi");
                break;
            }

            _ => {}
        }
    }
    panic!("Unexpected error while parsing EnderEmi.");
}

fn parse_nfref(reader: &mut XmlReader) -> NFRef {
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
                    let txt: String = read_text_string(reader, &e);
                    match name {
                        b"refNFe" => return NFRef::refNFe(txt),
                        b"refNFeSig" => return NFRef::refNFeSig(txt),
                        b"refCTe" => return NFRef::refCTe(txt),
                        _ => {break;} // Desconhecido. Forçar erro
                    }
                }
            }

            Ok(Event::Eof) => {
                log::error!("Unexpected Eof while parsing NFRef");
                break;
            }

            _ => {}
        }
    }
    panic!("Unexpected error while parsing NFRef.");
}

#[allow(non_snake_case)]
fn parse_refNF(reader: &mut XmlReader) -> NFRef {
    let mut refNF: RefNFData = RefNFData::default();
    
    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let txt = read_text_string(reader, &e);
                match e.name().as_ref() {
                    b"cUF" => refNF.cUF = txt.parse::<u8>().unwrap(),
                    b"AAMM" => refNF.AAMM = txt,
                    b"CNPJ" => refNF.CNPJ = txt,
                    b"mod" => refNF.r#mod = txt.parse::<u8>().unwrap(),
                    b"serie" => refNF.serie = txt.parse::<u16>().unwrap(),
                    b"nNF" => refNF.nNF = txt.parse::<u32>().unwrap(),
                    _ => {break;}
                }
            }

            Ok(Event::End(e)) if e.name().as_ref() == b"refNF" => {
                return NFRef::refNF(refNF);
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
fn parse_refNFP(reader: &mut XmlReader) -> NFRef {
        let mut refNFP: RefNFPData = RefNFPData::default();
    
    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let txt = read_text_string(reader, &e);
                match e.name().as_ref() {
                    b"cUF" => refNFP.cUF = txt.parse::<u8>().unwrap(),
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
                return NFRef::refNFP(refNFP);
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
fn parse_refECF(reader: &mut XmlReader) -> NFRef {
    let mut refECF: RefECFData = RefECFData::default();
    
    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let txt = read_text_string(reader, &e);
                match e.name().as_ref() {
                    b"mod" => refECF.r#mod = txt,
                    b"nECF" => refECF.nECF = txt,
                    b"nCOO" => refECF.nCOO = txt,
                    _ => {break;}
                }
            }

            Ok(Event::End(e)) if e.name().as_ref() == b"refECF" => {
                return NFRef::refECF(refECF);
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
pub fn parse_gCompraGov(reader: &mut XmlReader) -> CompraGov {
    let mut cg: CompraGov = CompraGov::default();

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let name = e.name();
                let txt: String = read_text_string(reader, &e);

                match name.as_ref() {
                    b"tpEnteGov" => cg.tpEnteGov = txt.parse().unwrap(),
                    b"pRedutor" => cg.pRedutor = txt.parse().unwrap(),
                    b"tpOperGov" => cg.tpOperGov = txt.parse().unwrap(),
                    _ => {
                        log::warn!("Elemento CompraGov não mapeado: {}", std::str::from_utf8(e.name().as_ref()).unwrap_or("<inválido>"));
                        break;
                    }
                }
            }

            Ok(Event::End(e)) if e.name().as_ref() == b"gCompraGov" => {
                return cg;
            }

            Ok(Event::Eof) => {
                log::error!("Unexpected Eof while parsing gCompraGov");
                break;
            }

            Err(e) => log::error!("Error reading gCompraGov: {}", e),
            _ => {}
        }
    }
    panic!("Unexpected error while parsing gCompraGov.");
}


#[allow(non_snake_case)]
pub fn parse_gPagAntecipado(reader: &mut XmlReader) -> Vec<String> {
    let mut refNfes: Vec<String> = Vec::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => if e.name().as_ref() == b"refNFe" {
                refNfes.push(read_text_string(reader, &e));
            }

            // Tag terminou
            Ok(Event::End(e)) => if e.name().as_ref() == b"gPagAntecipado" {
                return refNfes;
            }

            Ok(Event::Eof) => {
                log::error!("Unexpected Eof while parsing gPagAntecipado");
                break;
            }

            Err(e) => log::error!("Error reading gPagAntecipado: {}", e),
            _ => {}
        }
    }
    panic!("Unexpected error while parsing gPagAntecipado.");
}

pub fn get_mod_nfe(xml_bytes: &String) -> ModNfe {
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
                return ModNfe::from(txt.as_str());
            }

            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
    }

    return ModNfe::Desconhecido;
}

#[inline]
fn read_text_string(reader: &mut XmlReader, e: &BytesStart) -> String {
    let txt: String = reader.read_text(e.name()).unwrap().into_owned();

    //log::info!("Txt: {}", txt);
    return txt;
}
