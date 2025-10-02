#![allow(non_snake_case, non_camel_case_types)]

use std::error::Error;

use quick_xml::events::Event;
use rust_decimal::Decimal;
use serde::Serialize;

use crate::nfe::common::{ParseError, XmlReader, read_text_string};

#[derive(Debug, Default, Serialize)]
pub struct Total {
    pub ICMSTot: ICMSTot,
    pub ISSQNtot: Option<ISSQNtot>,
    pub retTrib: Option<RetTrib>,
    pub ISTot: Option<ISTot>,
    pub IBSCBSTot: Option<IBSCBSTot>,
    pub vNFTot: Option<Decimal>,
}

#[derive(Debug, Default, Serialize)]
pub struct ICMSTot {
    pub vBC: Decimal,
    pub vICMS: Decimal,
    pub vICMSDeson: Decimal,
    pub vFCPUFDest: Option<Decimal>,
    pub vICMSUFDest: Option<Decimal>,
    pub vICMSUFRemet: Option<Decimal>,
    pub vFCP: Decimal,
    pub vBCST: Decimal,
    pub vST: Decimal,
    pub vFCPST: Decimal,
    pub vFCPSTRet: Decimal,
    pub qBCMono: Option<Decimal>,
    pub vICMSMono: Option<Decimal>,
    pub qBCMonoReten: Option<Decimal>,
    pub vICMSMonoReten: Option<Decimal>,
    pub qBCMonoRet: Option<Decimal>,
    pub vICMSMonoRet: Option<Decimal>,
    pub vProd: Decimal,
    pub vFrete: Decimal,
    pub vSeg: Decimal,
    pub vDesc: Decimal,
    pub vII: Decimal,
    pub vIPI: Decimal,
    pub vIPIDevol: Decimal,
    pub vPIS: Decimal,
    pub vCOFINS: Decimal,
    pub vOutro: Decimal,
    pub vNF: Decimal,
    pub vTotTrib: Option<Decimal>,
}

#[derive(Debug, Default, Serialize)]
pub struct ISSQNtot {
    pub vServ: Option<Decimal>,
    pub vBC: Option<Decimal>,
    pub vISS: Option<Decimal>,
    pub vPIS: Option<Decimal>,
    pub vCOFINS: Option<Decimal>,
    pub dCompet: String,
    pub vDeducao: Option<Decimal>,
    pub vOutro: Option<Decimal>,
    pub vDescIncond: Option<Decimal>,
    pub vDescCond: Option<Decimal>,
    pub vISSRet: Option<Decimal>,
    pub cRegTrib: Option<u8>,
}

#[derive(Debug, Default, Serialize)]
pub struct RetTrib {
    pub vRetPIS: Option<Decimal>,
    pub vRetCOFINS: Option<Decimal>,
    pub vRetCSLL: Option<Decimal>,
    pub vBCIRRF: Option<Decimal>,
    pub vIRRF: Option<Decimal>,
    pub vBCRetPrev: Option<Decimal>,
    pub vRetPrev: Option<Decimal>,
}

#[derive(Debug, Default, Serialize)]
pub struct ISTot {
    pub vIS: Decimal,
}

#[derive(Debug, Default, Serialize)]
pub struct GIBSTotUF {
    pub vDif: Decimal,
    pub vDevTrib: Decimal,
    pub vIBSUF: Decimal,
}

#[derive(Debug, Default, Serialize)]
pub struct GIBSTotMun {
    pub vDif: Decimal,
    pub vDevTrib: Decimal,
    pub vIBSMun: Decimal,
}

#[derive(Debug, Default, Serialize)]
pub struct GIBSTot {
    pub gIBSUF: GIBSTotUF,
    pub gIBSMun: GIBSTotMun,
    pub vIBS: Decimal,
    pub vCredPres: Decimal,
    pub vCredPresCondSus: Decimal,
}

#[derive(Debug, Default, Serialize)]
pub struct GCBSTot {
    pub vDif: Decimal,
    pub vDevTrib: Decimal,
    pub vCBS: Decimal,
    pub vCredPres: Decimal,
    pub vCredPresCondSus: Decimal,
}

#[derive(Debug, Default, Serialize)]
pub struct GMonoTot {
    pub vIBSMono: Decimal,
    pub vCBSMono: Decimal,
    pub vIBSMonoReten: Decimal,
    pub vCBSMonoReten: Decimal,
    pub vIBSMonoRet: Decimal,
    pub vCBSMonoRet: Decimal,
}

/// Valores totais da NF com IBS / CBS.
#[derive(Debug, Default, Serialize)]
pub struct IBSCBSTot {
    pub vBCIBSCBS: Decimal,
    pub gIBS: Option<GIBSTot>,
    pub gCBS: Option<GCBSTot>,
    pub gMono: Option<GMonoTot>,
}

pub fn parse_total(reader: &mut XmlReader) -> Result<Total, Box<dyn Error>> {
    let mut total: Total = Total::default();
    loop {
        match reader.read_event()? {
            Event::Start(e) => match e.name().as_ref() {
                b"ICMSTot" => total.ICMSTot = parse_ICMSTot(reader)?,
                b"ISSQNtot" => total.ISSQNtot = Some(parse_ISSQNtot(reader)?),
                b"retTrib" => total.retTrib = Some(parse_retTrib(reader)?),
                b"ISTot" => total.ISTot = Some(parse_ISTot(reader)?),
                b"IBSCBSTot" => total.IBSCBSTot = Some(parse_IBSCBSTot(reader)?),
                b"vNFTot" => total.vNFTot = Some(read_text_string(reader, &e)?.parse::<Decimal>()?),
                _ => (),
            },
            Event::End(e) if e.name().as_ref() == b"total" => return Ok(total),
            Event::Eof => return Err(Box::new(ParseError::UnexpectedEof("total".to_string()))),
            _ => (),
        }
    }
}

fn parse_ICMSTot(reader: &mut XmlReader) -> Result<ICMSTot, Box<dyn Error>> {
    let mut g: ICMSTot = ICMSTot::default();
    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                let txt = read_text_string(reader, &e)?;
                match e.name().as_ref() {
                    b"vBC" => g.vBC = txt.parse::<Decimal>()?,
                    b"vICMS" => g.vICMS = txt.parse::<Decimal>()?,
                    b"vICMSDeson" => g.vICMSDeson = txt.parse::<Decimal>()?,
                    b"vFCPUFDest" => g.vFCPUFDest = Some(txt.parse::<Decimal>()?),
                    b"vICMSUFDest" => g.vICMSUFDest = Some(txt.parse::<Decimal>()?),
                    b"vICMSUFRemet" => g.vICMSUFRemet = Some(txt.parse::<Decimal>()?),
                    b"vFCP" => g.vFCP = txt.parse::<Decimal>()?,
                    b"vBCST" => g.vBCST = txt.parse::<Decimal>()?,
                    b"vST" => g.vST = txt.parse::<Decimal>()?,
                    b"vFCPST" => g.vFCPST = txt.parse::<Decimal>()?,
                    b"vFCPSTRet" => g.vFCPSTRet = txt.parse::<Decimal>()?,
                    b"qBCMono" => g.qBCMono = Some(txt.parse::<Decimal>()?),
                    b"vICMSMono" => g.vICMSMono = Some(txt.parse::<Decimal>()?),
                    b"qBCMonoReten" => g.qBCMonoReten = Some(txt.parse::<Decimal>()?),
                    b"vICMSMonoReten" => g.vICMSMonoReten = Some(txt.parse::<Decimal>()?),
                    b"qBCMonoRet" => g.qBCMonoRet = Some(txt.parse::<Decimal>()?),
                    b"vICMSMonoRet" => g.vICMSMonoRet = Some(txt.parse::<Decimal>()?),
                    b"vProd" => g.vProd = txt.parse::<Decimal>()?,
                    b"vFrete" => g.vFrete = txt.parse::<Decimal>()?,
                    b"vSeg" => g.vSeg = txt.parse::<Decimal>()?,
                    b"vDesc" => g.vDesc = txt.parse::<Decimal>()?,
                    b"vII" => g.vII = txt.parse::<Decimal>()?,
                    b"vIPI" => g.vIPI = txt.parse::<Decimal>()?,
                    b"vIPIDevol" => g.vIPIDevol = txt.parse::<Decimal>()?,
                    b"vPIS" => g.vPIS = txt.parse::<Decimal>()?,
                    b"vCOFINS" => g.vCOFINS = txt.parse::<Decimal>()?,
                    b"vOutro" => g.vOutro = txt.parse::<Decimal>()?,
                    b"vNF" => g.vNF = txt.parse::<Decimal>()?,
                    b"vTotTrib" => g.vTotTrib = Some(txt.parse::<Decimal>()?),
                    _ => (),
                }
            }
            Event::End(e) if e.name().as_ref() == b"ICMSTot" => return Ok(g),
            Event::Eof => {
                return Err(Box::new(ParseError::UnexpectedEof("ICMSTot".to_string())));
            }
            _ => (),
        }
    }
}

fn parse_ISSQNtot(reader: &mut XmlReader) -> Result<ISSQNtot, Box<dyn Error>> {
    let mut g: ISSQNtot = ISSQNtot::default();
    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                let txt = read_text_string(reader, &e)?;
                match e.name().as_ref() {
                    b"vServ" => g.vServ = Some(txt.parse::<Decimal>()?),
                    b"vBC" => g.vBC = Some(txt.parse::<Decimal>()?),
                    b"vISS" => g.vISS = Some(txt.parse::<Decimal>()?),
                    b"vPIS" => g.vPIS = Some(txt.parse::<Decimal>()?),
                    b"vCOFINS" => g.vCOFINS = Some(txt.parse::<Decimal>()?),
                    b"dCompet" => g.dCompet = txt,
                    b"vDeducao" => g.vDeducao = Some(txt.parse::<Decimal>()?),
                    b"vOutro" => g.vOutro = Some(txt.parse::<Decimal>()?),
                    b"vDescIncond" => g.vDescIncond = Some(txt.parse::<Decimal>()?),
                    b"vDescCond" => g.vDescCond = Some(txt.parse::<Decimal>()?),
                    b"vISSRet" => g.vISSRet = Some(txt.parse::<Decimal>()?),
                    b"cRegTrib" => g.cRegTrib = Some(txt.parse::<u8>()?),
                    _ => (),
                }
            }
            Event::End(e) if e.name().as_ref() == b"ISSQNtot" => return Ok(g),
            Event::Eof => {
                return Err(Box::new(ParseError::UnexpectedEof("ISSQNtot".to_string())));
            }
            _ => (),
        }
    }
}

fn parse_retTrib(reader: &mut XmlReader) -> Result<RetTrib, Box<dyn Error>> {
    let mut g: RetTrib = RetTrib::default();
    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                let txt = read_text_string(reader, &e)?;
                match e.name().as_ref() {
                    b"vRetPIS" => g.vRetPIS = Some(txt.parse::<Decimal>()?),
                    b"vRetCOFINS" => g.vRetCOFINS = Some(txt.parse::<Decimal>()?),
                    b"vRetCSLL" => g.vRetCSLL = Some(txt.parse::<Decimal>()?),
                    b"vBCIRRF" => g.vBCIRRF = Some(txt.parse::<Decimal>()?),
                    b"vIRRF" => g.vIRRF = Some(txt.parse::<Decimal>()?),
                    b"vBCRetPrev" => g.vBCRetPrev = Some(txt.parse::<Decimal>()?),
                    b"vRetPrev" => g.vRetPrev = Some(txt.parse::<Decimal>()?),
                    _ => (),
                }
            }
            Event::End(e) if e.name().as_ref() == b"retTrib" => return Ok(g),
            Event::Eof => {
                return Err(Box::new(ParseError::UnexpectedEof("retTrib".to_string())));
            }
            _ => (),
        }
    }
}

fn parse_ISTot(reader: &mut XmlReader) -> Result<ISTot, Box<dyn Error>> {
    let mut g: ISTot = ISTot::default();
    loop {
        match reader.read_event()? {
            Event::Start(e) if e.name().as_ref() == b"vIS" => {
                g.vIS = read_text_string(reader, &e)?.parse::<Decimal>()?;
            }
            Event::End(e) if e.name().as_ref() == b"ISTot" => return Ok(g),
            Event::Eof => return Err(Box::new(ParseError::UnexpectedEof("ISTot".to_string()))),
            _ => (),
        }
    }
}


fn parse_IBSCBSTot(reader: &mut XmlReader) -> Result<IBSCBSTot, Box<dyn Error>> {
    let mut g: IBSCBSTot = IBSCBSTot::default();
    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                match e.name().as_ref() {
                    b"gIBS" => g.gIBS = Some(parse_GIBSTot(reader)?),
                    b"gCBS" => g.gCBS = Some(parse_GCBSTot(reader)?),
                    b"gMono" => g.gMono = Some(parse_GMonoTot(reader)?),
                    b"vBCIBSCBS" => g.vBCIBSCBS = read_text_string(reader, &e)?.parse::<Decimal>()?,
                    _ => (),
                }
            }
            Event::End(e) if e.name().as_ref() == b"IBSCBSTot" => return Ok(g),
            Event::Eof => return Err(Box::new(ParseError::UnexpectedEof("IBSCBSTot".to_string()))),
            _ => (),
        }
    }
}



fn parse_GIBSTot(reader: &mut XmlReader) -> Result<GIBSTot, Box<dyn Error>> {
    let mut g: GIBSTot = GIBSTot::default();
    loop {
        match reader.read_event()? {
            Event::Start(e) => match e.name().as_ref() {
                b"gIBSUF" => g.gIBSUF = parse_GIBSTotUF(reader)?,
                b"gIBSMun" => g.gIBSMun = parse_GIBSTotMun(reader)?,
                name => {
                    let txt: String = read_text_string(reader, &e)?;
                    match name {
                        b"vIBS" => g.vIBS = txt.parse::<Decimal>()?,
                        b"vCredPres" => g.vCredPres = txt.parse::<Decimal>()?,
                        b"vCredPresCondSus" => g.vCredPresCondSus = txt.parse::<Decimal>()?,
                        _ => (),
                    }
                }
            },
            Event::End(e) if e.name().as_ref() == b"gIBS" => return Ok(g),
            Event::Eof => return Err(Box::new(ParseError::UnexpectedEof("gIBS".to_string()))),
            _ => (),
        }
    }
}

fn parse_GCBSTot(reader: &mut XmlReader) -> Result<GCBSTot, Box<dyn Error>> {
    let mut g: GCBSTot = GCBSTot::default();
    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                let txt = read_text_string(reader, &e)?;
                match e.name().as_ref() {
                    b"vDif" => g.vDif = txt.parse()?,
                    b"vDevTrib" => g.vDevTrib = txt.parse()?,
                    b"vCBS" => g.vCBS = txt.parse()?,
                    b"vCredPres" => g.vCredPres = txt.parse()?,
                    b"vCredPresCondSus" => g.vCredPresCondSus = txt.parse()?,
                    _ => (),
                }
            }
            Event::End(e) if e.name().as_ref() == b"gCBS" => return Ok(g),
            Event::Eof => return Err(Box::new(ParseError::UnexpectedEof("gCBS".to_string()))),
            _ => (),
        }
    }
}

fn parse_GMonoTot(reader: &mut XmlReader) -> Result<GMonoTot, Box<dyn Error>> {
    let mut g = GMonoTot::default();
    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                let txt = read_text_string(reader, &e)?;
                match e.name().as_ref() {
                    b"vIBSMono" => g.vIBSMono = txt.parse()?,
                    b"vCBSMono" => g.vCBSMono = txt.parse()?,
                    b"vIBSMonoReten" => g.vIBSMonoReten = txt.parse()?,
                    b"vCBSMonoReten" => g.vCBSMonoReten = txt.parse()?,
                    b"vIBSMonoRet" => g.vIBSMonoRet = txt.parse()?,
                    b"vCBSMonoRet" => g.vCBSMonoRet = txt.parse()?,
                    _ => (),
                }
            }
            Event::End(e) if e.name().as_ref() == b"gMono" => return Ok(g),
            Event::Eof => return Err(Box::new(ParseError::UnexpectedEof("gMono".to_string()))),
            _ => (),
        }
    }
}

fn parse_GIBSTotUF(reader: &mut XmlReader) -> Result<GIBSTotUF, Box<dyn Error>> {
    let mut g: GIBSTotUF = GIBSTotUF::default();
    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                let txt = read_text_string(reader, &e)?;
                match e.name().as_ref() {
                    b"vDif" => g.vDif = txt.parse::<Decimal>()?,
                    b"vDevTrib" => g.vDevTrib = txt.parse::<Decimal>()?,
                    b"vIBSUF" => g.vIBSUF = txt.parse::<Decimal>()?,
                    _ => (),
                }
            }
            Event::End(e) if e.name().as_ref() == b"gIBSUF" => return Ok(g),
            Event::Eof => return Err(Box::new(ParseError::UnexpectedEof("gIBSUF".to_string()))),
            _ => (),
        }
    }
}

fn parse_GIBSTotMun(reader: &mut XmlReader) -> Result<GIBSTotMun, Box<dyn Error>> {
    let mut g: GIBSTotMun = GIBSTotMun::default();
    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                let txt = read_text_string(reader, &e)?;
                match e.name().as_ref() {
                    b"vDif" => g.vDif = txt.parse::<Decimal>()?,
                    b"vDevTrib" => g.vDevTrib = txt.parse::<Decimal>()?,
                    b"vIBSMun" => g.vIBSMun = txt.parse::<Decimal>()?,
                    _ => (),
                }
            }
            Event::End(e) if e.name().as_ref() == b"gIBSMun" => return Ok(g),
            Event::Eof => return Err(Box::new(ParseError::UnexpectedEof("gIBSMun".to_string()))),
            _ => (),
        }
    }
}