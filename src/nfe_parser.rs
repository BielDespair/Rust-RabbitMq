use quick_xml::{events::{BytesStart, Event}, Reader};

use crate::nfes::{CompraGov, Ide, NFRef, NFe, NfeJson};




pub fn parse_nfe(xml_bytes: String) -> Option<String> {

    let modelo: ModNfe = get_mod_nfe(&xml_bytes);

    log::info!("Modelo: {:?}", modelo);
    
    
    match modelo {
        ModNfe::Mod55 => {return parse_nfe_mod_65(xml_bytes)}
        ModNfe::Mod65 => {return parse_nfe_mod_65(xml_bytes)}
        ModNfe::Mod57 => {return parse_nfe_mod_57(xml_bytes)}
        ModNfe::Desconhecido => todo!(),
    }
}



// Checar se é um envelope ou não.
pub fn parse_nfe_mod_65(xml: String) -> Option<String> {

    let mut nfe_json: NfeJson = NfeJson::default();

    let nfes: Vec<NFe> = Vec::new();
    let mut nfe: Option<NFe> = None;
    
    let mut reader: Reader<&[u8]> = Reader::from_str(&xml);
    reader.config_mut().trim_text(true);

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {

                match e.name().as_ref() {
                    b"NFe" => {
                        nfe = Some(NFe::default());
                    }
                    b"ide" => {
                        if let Some(ref mut n) = nfe {
                            n.ide = parse_ide(&mut reader);
                        }
                    }

                    b"emit" => {
                        println!("Achei emit");
                    }

                    _ => {}
                    
                }
                // match ... ide, emit...
                // 
            }

            Ok(Event::Text(e)) => {
                let bytes: &[u8] = e.as_ref();
                let txt: &str = str::from_utf8(bytes).unwrap();

            }

            Ok(Event::End(e)) if e.name().as_ref() == b"NFe" => {
                if let Some(n) = nfe.take() {
                    nfe_json.nfes.push(n);
                }
            }
            Ok(Event::Eof) => {
                break;
            }

            Err(e) => { break; }

            _ => {}

        }
    }
    return serde_json::to_string(&nfe_json).ok();
}

pub fn parse_nfe_mod_57(xml: String) -> Option<String> {
    return Some("MOD56-PARSED".to_string());
}


pub fn parse_ide(reader: &mut Reader<&[u8]>) -> Ide {
    // Começa com uma struct com valores padrão
    let mut ide = Ide::default();

    // Transformar NFRef em Option<Vec<NFRef>>
    // Ver as nuâncias de como lidar com isso
    let mut nfref: Option<NFRef> = None;

    
    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let name = e.name();
                let txt: String = read_text_string(reader, &e);

                match name.as_ref() {                
                    b"cUF" => ide.cUF = txt.parse().unwrap(),
                    b"cNF" => ide.cNF = txt,
                    b"natOp" => ide.natOp = txt,
                    b"mod" => ide.r#mod = txt.parse::<u8>().unwrap(),
                    b"serie" => ide.serie = txt.parse::<u16>().unwrap(),
                    b"nNF" => ide.nNF = txt.parse::<u32>().unwrap(),
                    b"dhEmi" => ide.dhEmi = txt,
                    b"dhSaiEnt" => ide.dhSaiEnt = Some(txt),
                    b"tpNF" => ide.tpNF = txt == "1" || txt.to_lowercase() == "true",
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
                    b"indFinal" => ide.indFinal = read_text_bool(reader, &e),
                    b"indPres" => ide.indPres = txt.parse::<u8>().unwrap(),
                    b"indIntermed" => ide.indIntermed = Some(read_text_bool(reader, &e)),
                    b"procEmi" => ide.procEmi = txt.parse::<u8>().unwrap(),
                    b"verProc" => ide.verProc = txt,
                    b"dhCont" => ide.dhCont = Some(txt),
                    b"xJust" => ide.xJust = Some(txt),
                    b"refNFe" => {

                    }
                    _ => {
                        log::warn!("Elemento IDE não mapeado: {}",
                        std::str::from_utf8(e.name().as_ref()).unwrap_or("<inválido>"));
                    }
                }
            },

            Ok(Event::End(e)) if e.name().as_ref() == b"ide" => {
                //log::info!("End of Tag: {}",
                 //std::str::from_utf8(e.name().as_ref()).unwrap_or("<inválido>"));
                 return ide;
            }

            Ok(Event::End(e)) if e.name().as_ref() == b"NFref" => {
                if let Some(n) = nfref.take() {
                    ide.NFref.push(n);
                }
            }

            Err(e) => log::error!("Error reading XML: {}", e),
            _ => {return ide;}
        }
    }
}

pub fn parse_compra_gov(reader: &mut Reader<&[u8]>) -> Option<CompraGov> {
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
                        log::warn!("Elemento CompraGov não mapeado: {}",
                        std::str::from_utf8(e.name().as_ref()).unwrap_or("<inválido>"));
                    }
                }
            },

            Ok(Event::End(e)) if e.name().as_ref() == b"gCompraGov" => {
                 return Some(cg);
            }

            Err(e) => log::error!("Error reading gCompraGov: {}", e),
            _ => {return None;}
        }
    }
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
fn read_text_string(reader: &mut Reader<&[u8]>, e: &BytesStart) -> String {
    let txt: String = reader.read_text(e.name()).unwrap().into_owned();

    //log::info!("Txt: {}", txt);
    return txt;
}

fn read_text_bool(reader: &mut Reader<&[u8]>, e: &BytesStart) -> bool {
    let txt: String = reader.read_text(e.name()).unwrap().into_owned();
    return txt == "1" || txt.to_lowercase() == "true";
}

#[derive(Debug)]
pub enum ModNfe {
    Mod55,
    Mod57,
    Mod65,
    Desconhecido
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