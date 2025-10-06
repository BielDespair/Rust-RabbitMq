#![allow(non_snake_case)]
use core::panic;
use std::{error::Error};

use bytes::Bytes;
use quick_xml::{
    Reader,
    events::{BytesStart, Event},
};
use rust_decimal::Decimal;


use crate::{
    nfe::{agropecuario::parse_agropecuario, cana::parse_cana, cobr::{Cobr, Dup, Fat}, common::{get_tag_attribute, read_text, ParseError, XmlReader}, compra::parse_compra, det::imposto_devol::{ImpostoDevol, IpiDevol}, eventos::evento::{parse_evento_nfe, EventoJson}, exporta::parse_exporta, impostos::{
        cibs::{
            GIBSMun, TCredPres, TDevTrib, TDif, TRed, TTribCompraGov, TTribRegular, ValorCredPres, GCBS, GIBSUF, TCIBS
        },
        cofins::{self, COFINSAliq, COFINSOutr, COFINSQtde, CalculoCOFINSOutr, TipoCofins, COFINS},
        cofins_st::{CalculoCofinsSt, COFINSST},
        ibs_cbs::{TCredPresIBSZFM, TTransfCred, TributacaoIBS, IBSCBS},
        icms::{Icms, TipoICMS},
        icms_uf_dest::ICMSUFDest,
        ii::Ii,
        ipi::{self, CalculoIpi, IPITrib, Ipi},
        is::{CalculoIS, UnidadeTributavel, IS},
        issqn::ISSQN,
        monofasia::{GMonoDif, GMonoPadrao, GMonoRet, GMonoReten, TMonofasia},
        pis::{self, CalculoPISOutr, PISAliq, PISOutr, PISQtde, TipoPis, PIS},
        pis_st::{CalculoPisSt, PISST},
    }, infAdic::parse_infAdic, inf_intermed::parse_infIntermed, inf_resp_tec::parse_infRespTec, pag::parse_pag, total::parse_total, transp::{Lacre, RetTransp, TVeiculo, Transp, Transporta, TransporteRodoviario, VeiculoTransporte, Vol}}, nfes::{
        Adi, Arma, Avulsa, Cide, Combustivel, CompraGov, Dest, Det, DetExport, Emit, EmitenteId, Encerrante, EnderEmi, ExportInd, GCred, Ide, Imposto, InfProdEmb, InfProdNFF, Local, Medicamento, NFRef, NFe, NfeJson, OrigComb, Prod, ProdutoEspecifico, RefECFData, RefNFData, RefNFPData, Tributacao, Veiculo, DI, UF
    }
};

#[derive(Debug)]
enum TipoXml {
    NFe(Modelo),       // <nfeProc> ou <NFe>
    LoteNFe,        // <enviNFe>
    CTe(Modelo),
    LoteCTe, // 
    Evento,    // <procEventoNFe> ou <evento>
    LoteEvento,     // <envEvento>
    Desconhecido,
}

#[derive(Debug)]
enum Modelo {
    Mod55,
    Mod65,
    Mod57,
    Desconhecido
}



pub fn parse_xml(xml: Bytes, company_id: i64, org_id: i64) -> Result<Vec<u8>, Box<dyn Error>> {
    let tipo_xml: TipoXml = get_tipo_xml(&xml)?;

    log::debug!("Tipo XML: {:?}", tipo_xml);
    match tipo_xml {
        TipoXml::NFe(modelo) => {
            let mut nfe_json: NfeJson = parse_nfe(xml, modelo)?;
            nfe_json.company_id = company_id;
            nfe_json.org_id = org_id;
            return Ok(serde_json::to_vec(&nfe_json)?);
        }

        TipoXml::CTe(modelo) => todo!(),

        TipoXml::LoteNFe => todo!(),
        TipoXml::LoteCTe => todo!(),

        TipoXml::Evento => {
            let mut evento: EventoJson = parse_evento_nfe(xml)?;
            evento.company_id = company_id;
            evento.org_id = org_id;
            return Ok(serde_json::to_vec(&evento)?);
        }
        TipoXml::LoteEvento => todo!(),
        TipoXml::Desconhecido => return Err(ParseError::ModeloDesconhecido.into()),
    };
}

fn get_tipo_xml(xml: &Bytes) -> Result<TipoXml, Box<dyn Error>> {
    let mut reader: Reader<&[u8]> = Reader::from_reader(xml);
    reader.config_mut().trim_text(true);

    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                match e.name().as_ref() {
                    
                    // Eventos
                    b"evento" | b"procEventoNFe" => return Ok(TipoXml::Evento),

                    // Lotes de Eventos
                    b"envEvento" | b"retEnvEvento" => return Ok(TipoXml::LoteEvento),
                    
                    // Lote NFe ou CTe 
                    b"enviNFe" => return Ok(TipoXml::LoteNFe),
                    b"enviCTe" => return Ok(TipoXml::LoteCTe),
                    
                    b"NFe" | b"nfeProc" => return Ok(TipoXml::NFe(get_mod_nfe(&mut reader)?)),
                    b"CTe" | b"cteProc" => return Ok(TipoXml::CTe(get_mod_nfe(&mut reader)?)),
                    
                    // Qualquer outra tag raiz é desconhecida
                    _ => return Ok(TipoXml::Desconhecido)
                }
            }

            Event::Eof => return Err(Box::new(ParseError::ModeloDesconhecido)),
            _ => ()
        }
    }
}

fn get_mod_nfe(reader: &mut XmlReader) -> Result<Modelo, Box<dyn Error>> {

    loop {
        match reader.read_event()? {
            Event::Start(e)  if e.name().as_ref() == b"mod" => {
                let txt: String = read_text(reader, &e)?;
                return match txt.as_str() {
                    "55" => Ok(Modelo::Mod55),
                    "65" => Ok(Modelo::Mod65),
                    "57" => Ok(Modelo::Mod57),
                    _ => Ok(Modelo::Desconhecido),
                };

            }
            Event::Eof => return Ok(Modelo::Desconhecido),
            

            _ => (),
        }
    }
}

fn parse_nfe(xml: Bytes, modelo: Modelo) -> Result<NfeJson, Box<dyn Error>> {
    let mut nfe_json: NfeJson = NfeJson::default();
    let mut reader: Reader<&[u8]> = Reader::from_reader(&xml);

    match modelo {
        Modelo::Mod55 => {
            let nfe: NFe = parse_NFe(&mut reader)?;
            nfe_json.nfes.push(nfe);
            return Ok(nfe_json);
        }
        Modelo::Mod65 => {
            let nfe: NFe = parse_NFe(&mut reader)?;
            nfe_json.nfes.push(nfe);
            return Ok(nfe_json);
        }
        Modelo::Mod57 => {
            return Err(Box::new(ParseError::ModeloDesconhecido));
        }
        Modelo::Desconhecido => Err(Box::new(ParseError::ModeloDesconhecido))
    }
}


fn parse_NFe(reader: &mut XmlReader) -> Result<NFe, Box<dyn Error>> {
    let mut nfe: NFe = NFe::default();

    loop {
        match reader.read_event()? {
            Event::Start(e) => match e.name().as_ref() {
                b"infNFe" => nfe.Id = get_tag_attribute(&e, b"Id")?,
                b"ide" => nfe.ide = parse_ide(reader)?,
                b"emit" => nfe.emit = parse_emit(reader)?,
                b"avulsa" => nfe.avulsa = Some(parse_avulsa(reader)?),
                b"dest" => nfe.dest = Some(parse_dest(reader)?),
                b"retirada" => nfe.retirada = Some(parse_TLocal(reader, b"retirada")?),
                b"entrega" => nfe.entrega = Some(parse_TLocal(reader, b"entrega")?),
                //b"autXML" => (),
                b"det" => nfe.produtos.push(parse_det(reader)?),
                b"total" => nfe.total = parse_total(reader)?,
                b"transp" => nfe.transp = parse_transp(reader)?,
                b"cobr" => nfe.cobr = Some(parse_cobr(reader)?),
                b"pag" => nfe.pag = parse_pag(reader)?,
                b"infIntermed" => nfe.infIntermed = Some(parse_infIntermed(reader)?),
                b"infAdic" => nfe.infAdic = Some(parse_infAdic(reader)?),
                b"exporta" => nfe.exporta = Some(parse_exporta(reader)?),
                b"compra" => nfe.compra = Some(parse_compra(reader)?),
                b"cana" => nfe.cana = Some(parse_cana(reader)?),
                b"infRespTec" => nfe.infRespTec = Some(parse_infRespTec(reader)?),

                // infSolicNFF, simplificando leitura (é o campo seguinte e único)
                b"xSolic" => nfe.infSolicNFF = Some(read_text(reader, &e)?),

                b"agropecuario" => nfe.agropecuario = Some(parse_agropecuario(reader)?),
                _ => {}
            },

            Event::Eof => {
                return Ok(nfe);
            }

            _ => {}
        }
    }

}

fn parse_ide(reader: &mut XmlReader) -> Result<Ide, Box<dyn Error>> {
    // Começa com uma struct com valores padrão
    let mut ide: Ide = Ide::default();

    // Transformar NFRef em Option<Vec<NFRef>>
    // Ver as nuâncias de como lidar com isso

    loop {
        match reader.read_event()? {
            Event::Start(e) => match e.name().as_ref() {
                b"NFref" => ide
                    .NFref
                    .get_or_insert_with(Vec::new)
                    .push(parse_nfref(reader)?),
                b"gCompraGov" => ide.gCompraGov = Some(parse_gCompraGov(reader)?),
                b"gPagAntecipado" => ide.gPagAntecipado = Some(parse_gPagAntecipado(reader)?),

                name => {
                    let txt: String = read_text(reader, &e)?;
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
                        _ => {
                            log::warn!(
                                "Elemento ide não mapeado: {}",
                                std::str::from_utf8(name.as_ref())?
                            )
                        }
                    }
                }
            },

            Event::End(e) if e.name().as_ref() == b"ide" => {
                return Ok(ide);
            }

            Event::Eof => return Err(Box::new(ParseError::UnexpectedEof("ide".to_string()))),
            

            _ => {}
        }
    }
}

fn parse_emit(reader: &mut XmlReader) -> Result<Emit, Box<dyn Error>> {
    let mut emit: Emit = Emit::default();

    loop {
        match reader.read_event()? {
            Event::Start(e) => match e.name().as_ref() {
                b"enderEmit" => emit.enderEmit = parse_enderEmit(reader, b"enderEmit")?,

                name => {
                    let txt: String = read_text(reader, &e)?;
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

            Event::End(e) if e.name().as_ref() == b"emit" => return Ok(emit),

            Event::Eof => {
                log::error!("Unexpected Eof while parsing emit")
            }

            _ => {}
        }
    }
}

fn parse_avulsa(reader: &mut XmlReader) -> Result<Avulsa, Box<dyn Error>> {
    let mut avulsa = Avulsa::default();
    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                let txt = read_text(reader, &e)?;
                match e.name().as_ref() {
                    b"CNPJ" => avulsa.CNPJ = txt,
                    b"xOrgao" => avulsa.xOrgao = txt,
                    b"matr" => avulsa.matr = txt,
                    b"xAgente" => avulsa.xAgente = txt,
                    b"fone" => avulsa.fone = Some(txt),
                    b"UF" => avulsa.UF = UF::from(txt.as_str()),
                    b"nDAR" => avulsa.nDAR = Some(txt),
                    b"dEmi" => avulsa.dEmi = Some(txt),
                    b"vDAR" => avulsa.vDAR = Some(txt.parse()?),
                    b"repEmi" => avulsa.repEmi = txt,
                    b"dPag" => avulsa.dPag = Some(txt),
                    tag => {
                        log::warn!("Elemento <avulsa> não mapeado: {}", String::from_utf8_lossy(tag));
                    }
                }
            },
            Event::End(e) if e.name().as_ref() == b"avulsa" => {
                return Ok(avulsa);
            },
            Event::Eof => return Err(Box::new(ParseError::UnexpectedEof("avulsa".to_string()))),
            _ => {},
        }
    }
}

fn parse_dest(reader: &mut XmlReader) -> Result<Dest, Box<dyn Error>> {
    let mut dest: Dest = Dest::default();
    loop {
        match reader.read_event()? {
            Event::Start(e) => match e.name().as_ref() {
                // Delegação para o sub-parser de endereço
                b"enderDest" => dest.enderDest = Some(parse_enderEmit(reader, b"enderDest")?),

                // Tratamento dos campos finais
                name => {
                    let txt = read_text(reader, &e)?;
                    match name {
                        // Tratamento da <choice> de identificação
                        b"CNPJ" => dest.EmitenteId = EmitenteId::CNPJ(txt),
                        b"CPF" => dest.EmitenteId = EmitenteId::CPF(txt),
                        b"idEstrangeiro" => dest.EmitenteId = EmitenteId::idEstrangeiro(txt),

                        // Outros campos
                        b"xNome" => dest.xNome = Some(txt),
                        b"indIEDest" => dest.indIEDest = txt.parse::<u8>()?,
                        b"IE" => dest.IE = Some(txt),
                        b"ISUF" => dest.ISUF = Some(txt),
                        b"IM" => dest.IM = Some(txt),
                        b"email" => dest.email = Some(txt),
                        _ => (),
                    }
                }
            },
            Event::End(e) if e.name().as_ref() == b"dest" => return Ok(dest),
            Event::Eof => return Err(Box::new(ParseError::UnexpectedEof("dest".to_string()))),
            _ => (),
        }
    }
}

fn parse_det(reader: &mut XmlReader) -> Result<Det, Box<dyn Error>> {
    let mut det: Det = Det::default();

    loop {
        match reader.read_event()? {
            Event::Start(e) => match e.name().as_ref() {
                b"prod" => det.produto = parse_prod(reader)?,

                b"imposto" => det.imposto = parse_imposto(reader)?,
                b"impostoDevol" => det.impostoDevol = Some(parse_impostoDevol(reader)?),
                name => {
                    let txt: String = read_text(reader, &e)?;
                    match name {
                        b"infAdProd" => det.infAdProd = Some(txt),
                        b"vItem" => det.vItem = Some(txt.parse::<Decimal>()?),
                        tag => {
                            log::warn!("Elemento PIS não mapeado: {}", String::from_utf8_lossy(tag));
                        }
                    }
                }
            },

            Event::End(e) if e.name().as_ref() == b"det" => return Ok(det),

            Event::Eof => return Err(Box::new(ParseError::UnexpectedEof("det".to_string()))),
            _ => {}
        }
    }
}

fn parse_transp(reader: &mut XmlReader) -> Result<Transp, Box<dyn Error>> {
    let mut transp = Transp::default();
    let mut veicTransp: Option<TVeiculo> = None;
    let mut reboque: Option<Vec<TVeiculo>> = None;

    loop {
        match reader.read_event()? {
            Event::Start(e) => match e.name().as_ref() {
                b"transporta" => transp.transporta = Some(parse_transporta(reader)?),
                b"retTransp" => transp.retTransp = Some(parse_retTransp(reader)?),
                b"vol" => transp.vol.get_or_insert_with(Vec::new).push(parse_vol(reader)?),
            
                b"veicTransp" => veicTransp = Some(parse_TVeiculo(reader, b"veicTransp")?),
                b"reboque" => reboque.get_or_insert_with(Vec::new).push(parse_TVeiculo(reader, b"reboque")?),
                
                name => {
                    let txt = read_text(reader, &e)?;
                    match name {
                        b"modFrete" => transp.modFrete = txt.parse::<Decimal>()?,
                        b"vagao" => transp.veiculo = Some(VeiculoTransporte::Vagao{vagao: txt}),
                        b"balsa" => transp.veiculo = Some(VeiculoTransporte::Balsa{balsa: txt}),
                        _ => (),
                    }
                }
            },
            Event::End(e) if e.name().as_ref() == b"transp" => {
                if veicTransp.is_some() || reboque.is_some() {
                    transp.veiculo = Some(VeiculoTransporte::Rodoviario(TransporteRodoviario {
                        veicTransp,
                        reboque,
                    }));
                }
                return Ok(transp);
            }
            Event::Eof => return Err(Box::new(ParseError::UnexpectedEof("transp".to_string()))),
            _ => (),
        }
    }
}

fn parse_lacres(reader: &mut XmlReader) -> Result<Lacre, Box<dyn Error>> {
    let mut lacre: Lacre = Lacre::default();
    loop {
        match reader.read_event()? {
            Event::Start(e) if e.name().as_ref() == b"nLacre" => {
                lacre.nLacre = read_text(reader, &e)?;
            }
            Event::End(e) if e.name().as_ref() == b"lacres" => return Ok(lacre),

            Event::Eof => return Err(Box::new(ParseError::UnexpectedEof("lacres".to_string()))),
            _ => (),
        }
    }
}

fn parse_TVeiculo(reader: &mut XmlReader, end_tag: &[u8]) -> Result<TVeiculo, Box<dyn Error>> {
    let mut veiculo: TVeiculo = TVeiculo::default();
    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                let txt = read_text(reader, &e)?;
                match e.name().as_ref() {
                    b"placa" => veiculo.placa = txt,
                    b"UF" => veiculo.UF = Some(UF::from(txt.as_str())),
                    b"RNTC" => veiculo.RNTC = Some(txt),
                    _ => (),
                }
            }
            Event::End(e) if e.name().as_ref() == end_tag => return Ok(veiculo),
            Event::Eof => return Err(Box::new(ParseError::UnexpectedEof(String::from_utf8_lossy(end_tag).to_string()))),
            _ => (),
        }
    }
}

fn parse_vol(reader: &mut XmlReader) -> Result<Vol, Box<dyn Error>> {
    let mut vol = Vol::default();
    loop {
        match reader.read_event()? {
            Event::Start(e) => match e.name().as_ref() {
                b"lacres" => {
                    vol.lacres.get_or_insert_with(Vec::new).push(parse_lacres(reader)?);
                }
                name => {
                    let txt = read_text(reader, &e)?;
                    match name {
                        b"qVol" => vol.qVol = Some(txt),
                        b"esp" => vol.esp = Some(txt),
                        b"marca" => vol.marca = Some(txt),
                        b"nVol" => vol.nVol = Some(txt),
                        b"pesoL" => vol.pesoL = Some(txt.parse()?),
                        b"pesoB" => vol.pesoB = Some(txt.parse()?),
                        _ => (),
                    }
                }
            },
            Event::End(e) if e.name().as_ref() == b"vol" => return Ok(vol),

            Event::Eof => return Err(Box::new(ParseError::UnexpectedEof("vol".to_string()))),
            
            _ => (),
        }
    }
}

fn parse_transporta(reader: &mut XmlReader) -> Result<Transporta, Box<dyn Error>> {
    let mut t: Transporta = Transporta::default();
    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                let txt = read_text(reader, &e)?;
                match e.name().as_ref() {
                    b"CNPJ" => t.identificacao = Some(EmitenteId::CNPJ(txt)),
                    b"CPF" => t.identificacao = Some(EmitenteId::CPF(txt)),
                    b"xNome" => t.xNome = Some(txt),
                    b"IE" => t.IE = Some(txt),
                    b"xEnder" => t.xEnder = Some(txt),
                    b"xMun" => t.xMun = Some(txt),
                    b"UF" => t.UF = Some(UF::from(txt.as_str())),
                    _ => (),
                }
            }
            Event::End(e) if e.name().as_ref() == b"transporta" => return Ok(t),
            Event::Eof => return Err(Box::new(ParseError::UnexpectedEof("transporta".to_string()))),
            _ => (),
        }
    }
}

fn parse_retTransp(reader: &mut XmlReader) -> Result<RetTransp, Box<dyn Error>> {
    let mut rt: RetTransp = RetTransp::default();
    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                let txt = read_text(reader, &e)?;
                match e.name().as_ref() {
                    b"vServ" => rt.vServ = txt.parse()?,
                    b"vBCRet" => rt.vBCRet = txt.parse()?,
                    b"pICMSRet" => rt.pICMSRet = txt.parse()?,
                    b"vICMSRet" => rt.vICMSRet = txt.parse()?,
                    b"CFOP" => rt.CFOP = txt,
                    b"cMunFG" => rt.cMunFG = txt.parse()?,
                    _ => (),
                }
            }
            Event::End(e) if e.name().as_ref() == b"retTransp" => return Ok(rt),
            Event::Eof => return Err(Box::new(ParseError::UnexpectedEof("retTransp".to_string()))),
            _ => (),
        }
    }
}

fn parse_cobr(reader: &mut XmlReader) -> Result<Cobr, Box<dyn Error>> {
    let mut cobr: Cobr = Cobr::default();
    loop {
        match reader.read_event()? {
            Event::Start(e) => match e.name().as_ref() {
                b"fat" => cobr.fat = Some(parse_fat(reader)?),
                b"dup" => cobr.dup.get_or_insert_with(Vec::new).push(parse_dup(reader)?),
                
                _ => (),
            },
            Event::End(e) if e.name().as_ref() == b"cobr" => return Ok(cobr),
            Event::Eof => return Err(Box::new(ParseError::UnexpectedEof("cobr".to_string()))),
            _ => (),
        }
    }
}

fn parse_fat(reader: &mut XmlReader) -> Result<Fat, Box<dyn Error>> {
    let mut fat = Fat::default();
    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                let txt = read_text(reader, &e)?;
                match e.name().as_ref() {
                    b"nFat" => fat.nFat = Some(txt),
                    b"vOrig" => fat.vOrig = Some(txt.parse()?),
                    b"vDesc" => fat.vDesc = Some(txt.parse()?),
                    b"vLiq" => fat.vLiq = Some(txt.parse()?),
                    _ => (),
                }
            }
            Event::End(e) if e.name().as_ref() == b"fat" => return Ok(fat),
            Event::Eof => return Err(Box::new(ParseError::UnexpectedEof("fat".to_string()))),
            _ => (),
        }
    }
}

fn parse_dup(reader: &mut XmlReader) -> Result<Dup, Box<dyn Error>> {
    let mut dup: Dup = Dup::default();
    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                let txt: String = read_text(reader, &e)?;
                match e.name().as_ref() {
                    b"nDup" => dup.nDup = Some(txt),
                    b"dVenc" => dup.dVenc = Some(txt),
                    b"vDup" => dup.vDup = txt.parse()?,
                    _ => (),
                }
            }
            Event::End(e) if e.name().as_ref() == b"dup" => return Ok(dup),
            Event::Eof => return Err(Box::new(ParseError::UnexpectedEof("dup".to_string()))),
            _ => (),
        }
    }
}


fn parse_prod(reader: &mut XmlReader) -> Result<Prod, Box<dyn Error>> {
    let mut prod: Prod = Prod::default();

    loop {
        match reader.read_event()? {
            Event::Start(e) => match e.name().as_ref() {
                b"gCred" => prod
                    .gCred
                    .get_or_insert_default()
                    .push(parse_gCred(reader)?),
                b"DI" => prod.DI.get_or_insert_default().push(parse_DI(reader)?),
                b"detExport" => prod
                    .detExport
                    .get_or_insert_default()
                    .push(parse_detExport(reader)?),
                b"infProdNFF" => prod.infProdNFF = Some(parse_infProdNFF(reader)?),
                b"infProdEmb" => prod.infProdEmb = Some(parse_infProdEmb(reader)?),
                b"veicProd" => {
                    prod.especifico = Some(ProdutoEspecifico::veicProd(parse_veicProd(reader)?))
                }
                b"med" => prod.especifico = Some(ProdutoEspecifico::med(parse_med(reader)?)),
                b"arma" => {
                    let arma_parseada: Arma = parse_arma(reader)?;
                    if let Some(ProdutoEspecifico::arma(ref mut vec)) = prod.especifico {
                        vec.push(arma_parseada);
                    } else {
                        prod.especifico = Some(ProdutoEspecifico::arma(vec![arma_parseada]));
                    }
                }
                b"comb" => prod.especifico = Some(ProdutoEspecifico::comb(parse_comb(reader)?)),

                name => {
                    let txt: String = read_text(reader, &e)?;
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
            },

            Event::End(e) if e.name().as_ref() == b"prod" => return Ok(prod),

            Event::Eof => return Err(Box::new(ParseError::UnexpectedEof("prod".to_string()))),

            _ => {}
        }
    }
}

fn parse_gCred(reader: &mut XmlReader) -> Result<GCred, Box<dyn Error>> {
    let mut gCred: GCred = GCred::default();

    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                let txt = read_text(reader, &e)?;

                match e.name().as_ref() {
                    b"cCredPresumido" => gCred.cCredPresumido = txt,
                    b"pCredPresumido" => gCred.pCredPresumido = txt.parse::<Decimal>()?,
                    b"vCredPresumido" => gCred.vCredPresumido = txt.parse::<Decimal>()?,

                    _ => {}
                }
            }

            Event::End(e) if e.name().as_ref() == b"gCred" => return Ok(gCred),

            Event::Eof => return Err(Box::new(ParseError::UnexpectedEof("gCred".to_string()))),

            _ => {}
        }
    }
}

fn parse_DI(reader: &mut XmlReader) -> Result<DI, Box<dyn Error>> {
    let mut DI: DI = DI::default();

    loop {
        match reader.read_event()? {
            Event::Start(e) => match e.name().as_ref() {
                b"adi" => DI.adi.push(parse_adi(reader)?),

                name => {
                    let txt: String = read_text(reader, &e)?;
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
            },

            Event::End(e) if e.name().as_ref() == b"DI" => return Ok(DI),

            Event::Eof => return Err(Box::new(ParseError::UnexpectedEof("DI".to_string()))),
            _ => {}
        }
    }
}

fn parse_detExport(reader: &mut XmlReader) -> Result<DetExport, Box<dyn Error>> {
    let mut detExport: DetExport = DetExport::default();

    loop {
        match reader.read_event()? {
            Event::Start(e) => match e.name().as_ref() {
                b"exportInd" => {
                    let mut exportInd: ExportInd = ExportInd::default();
                    loop {
                        match reader.read_event()? {
                            Event::Start(e) => {
                                let txt: String = read_text(reader, &e)?;
                                match e.name().as_ref() {
                                    b"nRE" => exportInd.nRE = txt,
                                    b"chNFe" => exportInd.chNFe = txt,
                                    b"qExport" => exportInd.qExport = txt.parse::<Decimal>()?,
                                    _ => break,
                                }
                            }
                            Event::End(e) if e.name().as_ref() == b"exportInd" => {
                                detExport.exportInd = Some(exportInd);
                                break;
                            }
                            Event::Eof => {
                                return Err(Box::new(ParseError::UnexpectedEof(
                                    "exportInd".to_string(),
                                )));
                            }
                            _ => {}
                        }
                    }
                }
                name => {
                    let txt: String = read_text(reader, &e)?;
                    match name {
                        b"nDraw" => detExport.nDraw = Some(txt),
                        _ => (),
                    }
                }
            },

            Event::End(e) if e.name().as_ref() == b"detExport" => return Ok(detExport),

            Event::Eof => {
                return Err(Box::new(ParseError::UnexpectedEof("detExport".to_string())));
            }
            _ => {}
        }
    }
}

fn parse_infProdNFF(reader: &mut XmlReader) -> Result<InfProdNFF, Box<dyn Error>> {
    let mut infProdNFF: InfProdNFF = InfProdNFF::default();

    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                let txt = read_text(reader, &e)?;

                match e.name().as_ref() {
                    b"cProdFisco" => infProdNFF.cProdFisco = txt,
                    b"cOperNFF" => infProdNFF.cOperNFF = txt,
                    _ => {}
                }
            }

            Event::End(e) if e.name().as_ref() == b"infProdNFF" => return Ok(infProdNFF),

            Event::Eof => {
                return Err(Box::new(ParseError::UnexpectedEof(
                    "infProdNFF".to_string(),
                )));
            }

            _ => {}
        }
    }
}

fn parse_infProdEmb(reader: &mut XmlReader) -> Result<InfProdEmb, Box<dyn Error>> {
    let mut infProdEmb: InfProdEmb = InfProdEmb::default();

    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                let txt = read_text(reader, &e)?;

                match e.name().as_ref() {
                    b"xEmb" => infProdEmb.xEmb = txt,
                    b"qVolEmb" => infProdEmb.qVolEmb = txt.parse::<Decimal>()?,
                    b"uEmb" => infProdEmb.uEmb = txt,
                    _ => {}
                }
            }

            Event::End(e) if e.name().as_ref() == b"infProdEmb" => return Ok(infProdEmb),

            Event::Eof => {
                return Err(Box::new(ParseError::UnexpectedEof(
                    "infProdEmb".to_string(),
                )));
            }

            _ => {}
        }
    }
}

fn parse_veicProd(reader: &mut XmlReader) -> Result<Veiculo, Box<dyn Error>> {
    let mut veicProd: Veiculo = Veiculo::default();

    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                let txt = read_text(reader, &e)?;

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

            Event::End(e) if e.name().as_ref() == b"veicProd" => return Ok(veicProd),

            Event::Eof => {
                return Err(Box::new(ParseError::UnexpectedEof("veicProd".to_string())));
            }

            _ => {}
        }
    }
}

fn parse_med(reader: &mut XmlReader) -> Result<Medicamento, Box<dyn Error>> {
    let mut med = Medicamento::default();

    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                let txt = read_text(reader, &e)?;

                match e.name().as_ref() {
                    b"cProdANVISA" => med.cProdANVISA = txt,
                    b"xMotivoIsencao" => med.xMotivoIsencao = Some(txt),
                    b"vPMC" => med.vPMC = txt.parse::<Decimal>()?,
                    _ => (),
                }
            }
            Event::End(e) if e.name().as_ref() == b"med" => return Ok(med),
            Event::Eof => return Err(Box::new(ParseError::UnexpectedEof("med".to_string()))),
            _ => (),
        }
    }
}

fn parse_arma(reader: &mut XmlReader) -> Result<Arma, Box<dyn Error>> {
    let mut arma = Arma::default();

    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                let txt = read_text(reader, &e)?;
                match e.name().as_ref() {
                    b"tpArma" => arma.tpArma = txt.parse()?,
                    b"nSerie" => arma.nSerie = txt,
                    b"nCano" => arma.nCano = txt,
                    b"descr" => arma.descr = txt,
                    _ => (),
                }
            }

            Event::End(e) if e.name().as_ref() == b"arma" => return Ok(arma),

            Event::Eof => return Err(Box::new(ParseError::UnexpectedEof("arma".to_string()))),

            _ => (),
        }
    }
}

fn parse_comb(reader: &mut XmlReader) -> Result<Combustivel, Box<dyn Error>> {
    let mut combustivel = Combustivel::default();

    loop {
        match reader.read_event()? {
            Event::Start(e) => match e.name().as_ref() {
                // Tags que são grupos aninhados
                b"CIDE" => combustivel.CIDE = Some(parse_cide(reader)?),
                b"encerrante" => combustivel.encerrante = Some(parse_encerrante(reader)?),
                b"origComb" => {
                    let orig = parse_orig_comb(reader)?;
                    combustivel.origComb.get_or_insert_default().push(orig);
                }

                // Tags com valores simples
                name => {
                    let txt = read_text(reader, &e)?;
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
            Event::End(e) if e.name().as_ref() == b"comb" => return Ok(combustivel),
            Event::Eof => return Err(Box::new(ParseError::UnexpectedEof("comb".to_string()))),
            _ => (),
        }
    }
}

// --- Funções Auxiliares ---

fn parse_cide(reader: &mut XmlReader) -> Result<Cide, Box<dyn Error>> {
    let mut cide: Cide = Cide::default();
    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                let txt = read_text(reader, &e)?;
                match e.name().as_ref() {
                    b"qBCProd" => cide.qBCProd = txt.parse()?,
                    b"vAliqProd" => cide.vAliqProd = txt.parse()?,
                    b"vCIDE" => cide.vCIDE = txt.parse()?,
                    _ => (),
                }
            }
            Event::End(e) if e.name().as_ref() == b"CIDE" => return Ok(cide),
            Event::Eof => return Err(Box::new(ParseError::UnexpectedEof("CIDE".to_string()))),
            _ => (),
        }
    }
}

fn parse_encerrante(reader: &mut XmlReader) -> Result<Encerrante, Box<dyn Error>> {
    let mut encerrante: Encerrante = Encerrante::default();
    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                let txt = read_text(reader, &e)?;
                match e.name().as_ref() {
                    b"nBico" => encerrante.nBico = txt.parse()?,
                    b"nBomba" => encerrante.nBomba = Some(txt.parse()?),
                    b"nTanque" => encerrante.nTanque = txt.parse()?,
                    b"vEncIni" => encerrante.vEncIni = txt.parse()?,
                    b"vEncFin" => encerrante.vEncFin = txt.parse()?,
                    _ => (),
                }
            }
            Event::End(e) if e.name().as_ref() == b"encerrante" => return Ok(encerrante),
            Event::Eof => {
                return Err(Box::new(ParseError::UnexpectedEof(
                    "encerrante".to_string(),
                )));
            }
            _ => (),
        }
    }
}

fn parse_orig_comb(reader: &mut XmlReader) -> Result<OrigComb, Box<dyn Error>> {
    let mut orig: OrigComb = OrigComb::default();
    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                let txt = read_text(reader, &e)?;
                match e.name().as_ref() {
                    b"indImport" => orig.indImport = txt.parse()?,
                    b"cUFOrig" => orig.cUFOrig = txt.parse::<u8>()?,
                    b"pOrig" => orig.pOrig = txt.parse()?,
                    _ => (),
                }
            }
            Event::End(e) if e.name().as_ref() == b"origComb" => return Ok(orig),
            Event::Eof => {
                return Err(Box::new(ParseError::UnexpectedEof("origComb".to_string())));
            }
            _ => (),
        }
    }
}
fn parse_adi(reader: &mut XmlReader) -> Result<Adi, Box<dyn Error>> {
    let mut adi: Adi = Adi::default();
    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                let txt = read_text(reader, &e)?;

                match e.name().as_ref() {
                    b"nAdicao" => adi.nAdicao = Some(txt),
                    b"nSeqAdic" => adi.nSeqAdic = Some(txt),
                    b"cFabricante" => adi.cFabricante = txt,
                    b"vDescDI" => adi.vDescDI = Some(txt.parse::<Decimal>()?),
                    b"nDraw" => adi.nDraw = Some(txt),
                    _ => {}
                }
            }

            Event::End(e) if e.name().as_ref() == b"adi" => return Ok(adi),

            Event::Eof => return Err(Box::new(ParseError::UnexpectedEof("adi".to_string()))),
            _ => {}
        }
    }
}
fn parse_imposto(reader: &mut XmlReader) -> Result<Imposto, Box<dyn Error>> {
    let mut imposto: Imposto = Imposto::default();

    // Sequencia do tipo Mercadoria
    let mut icms: Option<Icms> = None;
    let mut ipi: Option<Ipi> = None;
    let mut ii: Option<Ii> = None;

    // Sequencia do tipo Servico
    let mut issqn: Option<ISSQN> = None;

    loop {
        match reader.read_event()? {
            Event::Start(e) => match e.name().as_ref() {
                b"ICMS" => icms = Some(parse_ICMS(reader)?),
                b"II" => ii = Some(parse_II(reader)?),

                b"IPI" => ipi = Some(parse_IPI(reader)?),
                b"ISSQN" => issqn = Some(parse_ISSQN(reader)?),
                b"PIS" => imposto.PIS = Some(parse_PIS(reader)?),
                b"PISST" => imposto.PISST = Some(parse_PISST(reader)?),
                b"COFINS" => imposto.COFINS = Some(parse_COFINS(reader)?),
                b"COFINSST" => imposto.COFINSST = Some(parse_COFINSST(reader)?),
                b"ICMSUFDest" => imposto.ICMSUFDest = Some(parse_ICMSUFDest(reader)?),
                b"IS" => imposto.IS = Some(parse_IS(reader)?),
                b"IBSCBS" => imposto.IBSCBS = Some(parse_IBSCBS(reader)?),

                name => {
                    let txt: String = read_text(reader, &e)?;
                    match name {
                        b"vTotTrib" => imposto.vTotTrib = Some(txt.parse::<Decimal>()?),
                        _ => {}
                    }
                }
            },

            Event::End(e) if e.name().as_ref() == b"imposto" => {
                if let Some(icms) = icms {
                    imposto.tributacao = Some(Tributacao::Mercadoria {
                        ICMS: icms,
                        IPI: ipi,
                        II: ii,
                    })
                } else if let Some(issqn) = issqn {
                    imposto.tributacao = Some(Tributacao::Servico {
                        IPI: ipi,
                        ISSQN: issqn,
                    })
                }
                return Ok(imposto);
            }

            Event::Eof => {
                return Err(Box::new(ParseError::UnexpectedEof("imposto".to_string())));
            }

            _ => {}
        }
    }
}

fn parse_impostoDevol(reader: &mut XmlReader) -> Result<ImpostoDevol, Box<dyn Error>> {
    let mut imposto_devol = ImpostoDevol::default();

    loop {
        match reader.read_event()? {
            Event::Start(e) => match e.name().as_ref() {
                b"IPI" => imposto_devol.IPI = parse_IpiDevol(reader)?,
                // Trata os campos que são filhos diretos
                name => {
                    let txt = read_text(reader, &e)?;
                    match name {
                        b"pDevol" => imposto_devol.pDevol = txt.parse()?,
                        _ => (),
                    }
                }
            },

            Event::End(e) if e.name().as_ref() == b"impostoDevol" => return Ok(imposto_devol),
            Event::Eof => return Err(Box::new(ParseError::UnexpectedEof("impostoDevol".to_string()))),
            _ => (),
        }
    }
}

fn parse_IpiDevol(reader: &mut XmlReader) -> Result<IpiDevol, Box<dyn Error>> {
    let mut ipi_devol: IpiDevol = IpiDevol::default();
    loop {
        match reader.read_event()? {
            Event::Start(e) if e.name().as_ref() == b"vIPIDevol" => ipi_devol.vIPIDevol = read_text(reader, &e)?.parse()?,
            
            Event::End(e) if e.name().as_ref() == b"IPI" => return Ok(ipi_devol),
            
            Event::Eof => return Err(Box::new(ParseError::UnexpectedEof("IPI em impostoDevol".to_string()))),
            _ => (),
        }
    }
}

fn parse_ICMS(reader: &mut XmlReader) -> Result<Icms, Box<dyn Error>> {
    let mut ICMS: Icms = Icms::default();
    loop {
        match reader.read_event()? {
            Event::Start(e) => match e.name().as_ref() {
                b"ICMS00" => ICMS.tipo = TipoICMS::ICMS00,
                b"ICMS02" => ICMS.tipo = TipoICMS::ICMS02,
                b"ICMS10" => ICMS.tipo = TipoICMS::ICMS10,
                b"ICMS15" => ICMS.tipo = TipoICMS::ICMS15,
                b"ICMS20" => ICMS.tipo = TipoICMS::ICMS20,
                b"ICMS30" => ICMS.tipo = TipoICMS::ICMS30,
                b"ICMS40" => ICMS.tipo = TipoICMS::ICMS40,
                b"ICMS51" => ICMS.tipo = TipoICMS::ICMS51,
                b"ICMS53" => ICMS.tipo = TipoICMS::ICMS53,
                b"ICMS60" => ICMS.tipo = TipoICMS::ICMS60,
                b"ICMS61" => ICMS.tipo = TipoICMS::ICMS61,
                b"ICMS70" => ICMS.tipo = TipoICMS::ICMS70,
                b"ICMS90" => ICMS.tipo = TipoICMS::ICMS90,
                b"ICMSPart" => ICMS.tipo = TipoICMS::ICMSPART,
                b"ICMSST" => ICMS.tipo = TipoICMS::ICMSST,
                b"ICMSSN101" => ICMS.tipo = TipoICMS::ICMSSN101,
                b"ICMSSN102" => ICMS.tipo = TipoICMS::ICMSSN102,
                b"ICMSSN201" => ICMS.tipo = TipoICMS::ICMSSN201,
                b"ICMSSN202" => ICMS.tipo = TipoICMS::ICMSSN202,
                b"ICMSSN500" => ICMS.tipo = TipoICMS::ICMSSN500,
                b"ICMSSN900" => ICMS.tipo = TipoICMS::ICMSSN900,
                name => {
                    let txt: String = read_text(reader, &e)?;
                    match name {
                        b"orig" => ICMS.orig = Some(txt),
                        b"CST" => ICMS.CST = Some(txt),
                        b"CSOSN" => ICMS.CSOSN = Some(txt),

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
                        b"UFST" => ICMS.UFST = Some(UF::from(txt.as_str())),

                        // --- ICMS ST (REPASSE) ---
                        b"vBCSTDest" => ICMS.vBCSTDest = Some(txt.parse()?),
                        b"vICMSSTDest" => ICMS.vICMSSTDest = Some(txt.parse()?),

                        // --- SIMPLES NACIONAL (CRÉDITO) ---
                        b"pCredSN" => ICMS.pCredSN = Some(txt.parse()?),
                        b"vCredICMSSN" => ICMS.vCredICMSSN = Some(txt.parse()?),

                        _ => {}
                    }
                }
            },

            Event::End(e) if e.name().as_ref() == b"ICMS" => return Ok(ICMS),

            Event::Eof => return Err(Box::new(ParseError::UnexpectedEof("ICMS".to_string()))),

            _ => {}
        }
    }
}

fn parse_IPI(reader: &mut XmlReader) -> Result<Ipi, Box<dyn Error>> {
    let mut ipi: Ipi = Ipi::default();

    loop {
        match reader.read_event()? {
            Event::Start(e) => match e.name().as_ref() {
                b"IPITrib" => ipi.Tributacao = ipi::Tributacao::IPITrib(parse_IPITrib(reader)?),
                b"IPINT" => {} // É ignorado, pois será lido no match abaixo.

                name => {
                    let txt: String = read_text(reader, &e)?;
                    match name {
                        b"CNPJProd" => ipi.CNPJProd = Some(txt),
                        b"cSelo" => ipi.cSelo = Some(txt),
                        b"qSelo" => ipi.qSelo = Some(txt),
                        b"cEnq" => ipi.cEnq = txt,

                        // Filha de IPINT. Lido pois read_event só para quando encontrar 'IPI' novamente.
                        b"CST" => ipi.Tributacao = ipi::Tributacao::IPINT { CST: (txt) },
                        _ => {}
                    }
                }
            },

            Event::End(e) if e.name().as_ref() == b"IPI" => return Ok(ipi),

            Event::Eof => return Err(Box::new(ParseError::UnexpectedEof("IPI".to_string()))),

            _ => {}
        }
    }
}

fn parse_II(reader: &mut XmlReader) -> Result<Ii, Box<dyn Error>> {
    let mut ii: Ii = Ii::default();
    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                let name = e.name();
                let txt = read_text(reader, &e)?;

                match name.as_ref() {
                    b"vBC" => ii.vBC = txt.parse::<Decimal>()?,
                    b"vDespAdu" => ii.vDespAdu = txt.parse::<Decimal>()?,
                    b"vII" => ii.vII = txt.parse::<Decimal>()?,
                    b"vIOF" => ii.vIOF = txt.parse::<Decimal>()?,
                    _ => {}
                }
            }

            Event::End(e) if e.name().as_ref() == b"II" => return Ok(ii),
            Event::Eof => return Err(Box::new(ParseError::UnexpectedEof("II".to_string()))),

            _ => {}
        }
    }
}
fn parse_ISSQN(reader: &mut XmlReader) -> Result<ISSQN, Box<dyn Error>> {
    let mut ISSQN: ISSQN = ISSQN::default();
    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                let name = e.name();
                let txt = read_text(reader, &e)?;

                match name.as_ref() {
                    b"vBC" => ISSQN.vBC = txt.parse::<Decimal>()?,
                    b"vAliq" => ISSQN.vAliq = txt.parse::<Decimal>()?,
                    b"vISSQN" => ISSQN.vISSQN = txt.parse::<Decimal>()?,
                    b"cMunFG" => ISSQN.cMunFG = txt.parse::<u32>()?,
                    b"cListServ" => ISSQN.cListServ = txt,
                    b"vDeducao" => ISSQN.vDeducao = Some(txt.parse::<Decimal>()?),
                    b"vOutro" => ISSQN.vOutro = Some(txt.parse::<Decimal>()?),
                    b"vDescIncond" => ISSQN.vDescIncond = Some(txt.parse::<Decimal>()?),
                    b"vDescCond" => ISSQN.vDescCond = Some(txt.parse::<Decimal>()?),
                    b"vISSRet" => ISSQN.vISSRet = Some(txt.parse::<Decimal>()?),
                    b"indISS" => ISSQN.indISS = txt.parse::<u8>()?,
                    b"cServico" => ISSQN.cServico = Some(txt),
                    b"cMun" => ISSQN.cMun = Some(txt.parse::<u32>()?),
                    b"cPais" => ISSQN.cPais = Some(txt),
                    b"nProcesso" => ISSQN.nProcesso = Some(txt),
                    b"indIncentivo" => ISSQN.indIncentivo = txt.parse::<u8>()?,

                    tag => {
                        let tag_name: String = String::from_utf8_lossy(tag).to_string();
                        return Err(Box::new(ParseError::CampoDesconhecido(tag_name)));
                    }
                }
            }

            Event::End(e) if e.name().as_ref() == b"ISSQN" => return Ok(ISSQN),
            Event::Eof => return Err(Box::new(ParseError::UnexpectedEof("ISSQN".to_string()))),

            _ => {}
        }
    }
}

fn parse_PIS(reader: &mut XmlReader) -> Result<PIS, Box<dyn Error>> {
    let mut pis: PIS = PIS::default();

    loop {
        match reader.read_event()? {
            Event::Start(e) => match e.name().as_ref() {
                b"PISAliq" => {
                    pis.tipo = TipoPis::PISAliq;
                    pis.tributacao = pis::Tributacao::PISAliq(parse_PISAliq(reader)?);
                }
                b"PISQtde" => {
                    pis.tipo = TipoPis::PISQtde;
                    pis.tributacao = pis::Tributacao::PISQtde(parse_PISQtde(reader)?);
                }
                b"PISOutr" => {
                    pis.tipo = TipoPis::PISOutr;
                    pis.tributacao = pis::Tributacao::PISOutr(parse_PISOutr(reader)?);
                }

                b"PISNT" => {}
                b"CST" => {
                    let txt: String = read_text(reader, &e)?;
                    pis.tipo = TipoPis::PISNT;
                    pis.tributacao = pis::Tributacao::PISNT { CST: txt };
                }

                tag => {
                    let tag = String::from_utf8_lossy(tag);
                    log::warn!("Elemento PIS não mapeado: {}", tag);
                }
            },

            Event::End(e) if e.name().as_ref() == b"PIS" => return Ok(pis),

            Event::Eof => return Err(Box::new(ParseError::UnexpectedEof("PIS".to_string()))),

            _ => {}
        }
    }
}

fn parse_PISST(reader: &mut XmlReader) -> Result<PISST, Box<dyn Error>> {
    let mut pis_st = PISST::default();

    // Variáveis temporárias para os campos que definem o enum de cálculo
    let mut vBC: Option<Decimal> = None;
    let mut pPIS: Option<Decimal> = None;
    let mut qBCProd: Option<Decimal> = None;
    let mut vAliqProd: Option<Decimal> = None;

    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                let txt = read_text(reader, &e)?;
                match e.name().as_ref() {
                    // Campos diretos da struct PISST
                    b"vPIS" => pis_st.vPIS = txt.parse()?,
                    b"indSomaPISST" => pis_st.indSomaPISST = Some(txt == "1"),

                    // Campos que definem o enum, armazenados temporariamente
                    b"vBC" => vBC = Some(txt.parse()?),
                    b"pPIS" => pPIS = Some(txt.parse()?),
                    b"qBCProd" => qBCProd = Some(txt.parse()?),
                    b"vAliqProd" => vAliqProd = Some(txt.parse()?),

                    tag => {
                        let tag_name = String::from_utf8_lossy(tag).to_string();
                        log::warn!("Elemento PISST não mapeado: {}", tag_name);
                    }
                }
            }

            Event::End(e) if e.name().as_ref() == b"PISST" => {
                // Ao final do bloco, decide qual variante do enum construir
                if let (Some(vbc_val), Some(ppis_val)) = (vBC, pPIS) {
                    pis_st.calculo = CalculoPisSt::Aliquota {
                        vBC: vbc_val,
                        pPIS: ppis_val,
                    };
                } else if let (Some(qbc_val), Some(valiq_val)) = (qBCProd, vAliqProd) {
                    pis_st.calculo = CalculoPisSt::Unidade {
                        qBCProd: qbc_val,
                        vAliqProd: valiq_val,
                    };
                } else {
                    // Se nenhum dos pares obrigatórios foi encontrado, o XML é inválido
                    return Err("Estrutura de cálculo do PISST inválida ou incompleta".into());
                }

                return Ok(pis_st);
            }

            Event::Eof => return Err(Box::new(ParseError::UnexpectedEof("PISST".to_string()))),
            _ => {}
        }
    }
}

fn parse_COFINS(reader: &mut XmlReader) -> Result<COFINS, Box<dyn Error>> {
    let mut COFINS: COFINS = COFINS::default();
    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                match e.name().as_ref() {
                    b"COFINSAliq" => {
                        COFINS.tipo = TipoCofins::COFINSAliq;
                        COFINS.tributacao =
                            cofins::Tributacao::COFINSAliq(parse_COFINSAliq(reader)?);
                    }
                    b"COFINSQtde" => {
                        COFINS.tipo = TipoCofins::COFINSQtde;
                        COFINS.tributacao =
                            cofins::Tributacao::COFINSQtde(parse_COFINSQtde(reader)?)
                    }
                    b"COFINSOutr" => {
                        COFINS.tipo = TipoCofins::COFINSOutr;
                        COFINS.tributacao =
                            cofins::Tributacao::COFINSOutr(parse_COFINSOutr(reader)?)
                    }
                    b"COFINSNT" => {
                        COFINS.tipo = TipoCofins::COFINSNT;
                        COFINS.tributacao = cofins::Tributacao::COFINSNT {
                            CST: parse_COFINSNT(reader)?,
                        }
                    }

                    tag => {
                        let tag_name = String::from_utf8_lossy(tag).to_string();
                        return Err(Box::new(ParseError::CampoDesconhecido(tag_name)));
                    }
                };
            }
            Event::End(e) if e.name().as_ref() == b"COFINS" => {
                return Ok(COFINS);
            }
            Event::Eof => {
                return Err(Box::new(ParseError::UnexpectedEof("COFINS".to_string())));
            }
            _ => (),
        }
    }
}

fn parse_COFINSST(reader: &mut XmlReader) -> Result<COFINSST, Box<dyn Error>> {
    let mut cofins_st = COFINSST::default();

    // Variáveis temporárias para os campos do enum
    let mut vBC: Option<Decimal> = None;
    let mut pCOFINS: Option<Decimal> = None;
    let mut qBCProd: Option<Decimal> = None;
    let mut vAliqProd: Option<Decimal> = None;

    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                let txt = read_text(reader, &e)?;
                match e.name().as_ref() {
                    // Campos diretos da struct
                    b"vCOFINS" => cofins_st.vCOFINS = txt.parse()?,
                    b"indSomaCOFINSST" => cofins_st.indSomaCOFINSST = Some(txt.parse::<u8>()?),

                    // Campos do enum armazenados temporariamente
                    b"vBC" => vBC = Some(txt.parse()?),
                    b"pCOFINS" => pCOFINS = Some(txt.parse()?),
                    b"qBCProd" => qBCProd = Some(txt.parse()?),
                    b"vAliqProd" => vAliqProd = Some(txt.parse()?),

                    tag => {
                        let tag_name = String::from_utf8_lossy(tag).to_string();
                        log::warn!("Elemento COFINSST não mapeado: {}", tag_name);
                    }
                }
            }

            Event::End(e) if e.name().as_ref() == b"COFINSST" => {
                // Decide e constrói o enum com base nos campos coletados
                if let (Some(vbc_val), Some(pcofins_val)) = (vBC, pCOFINS) {
                    cofins_st.calculo = CalculoCofinsSt::Aliquota {
                        vBC: vbc_val,
                        pCOFINS: pcofins_val,
                    };
                } else if let (Some(qbc_val), Some(valiq_val)) = (qBCProd, vAliqProd) {
                    cofins_st.calculo = CalculoCofinsSt::Unidade {
                        qBCProd: qbc_val,
                        vAliqProd: valiq_val,
                    };
                } else {
                    return Err("Estrutura de cálculo do COFINSST inválida.".into());
                }

                return Ok(cofins_st);
            }

            Event::Eof => {
                return Err(Box::new(ParseError::UnexpectedEof("COFINSST".to_string())));
            }
            _ => {}
        }
    }
}

fn parse_ICMSUFDest(reader: &mut XmlReader) -> Result<ICMSUFDest, Box<dyn Error>> {
    let mut icms_uf_dest: ICMSUFDest = ICMSUFDest::default();

    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                let txt = read_text(reader, &e)?;
                match e.name().as_ref() {
                    b"vBCUFDest" => icms_uf_dest.vBCUFDest = txt.parse()?,
                    b"vBCFCPUFDest" => icms_uf_dest.vBCFCPUFDest = Some(txt.parse()?),
                    b"pFCPUFDest" => icms_uf_dest.pFCPUFDest = Some(txt.parse()?),
                    b"pICMSUFDest" => icms_uf_dest.pICMSUFDest = txt.parse()?,
                    b"pICMSInter" => icms_uf_dest.pICMSInter = txt,
                    b"pICMSInterPart" => icms_uf_dest.pICMSInterPart = txt.parse()?,
                    b"vFCPUFDest" => icms_uf_dest.vFCPUFDest = Some(txt.parse()?),
                    b"vICMSUFDest" => icms_uf_dest.vICMSUFDest = txt.parse()?,
                    b"vICMSUFRemet" => icms_uf_dest.vICMSUFRemet = txt.parse()?,
                    tag => {
                        let tag_name = String::from_utf8_lossy(tag).to_string();
                        log::warn!("Elemento ICMSUFDest não mapeado: {}", tag_name);
                    }
                }
            }
            Event::End(e) if e.name().as_ref() == b"ICMSUFDest" => {
                return Ok(icms_uf_dest);
            }
            Event::Eof => {
                return Err(Box::new(ParseError::UnexpectedEof(
                    "ICMSUFDest".to_string(),
                )));
            }
            _ => {}
        }
    }
}

fn parse_IS(reader: &mut XmlReader) -> Result<IS, Box<dyn Error>> {
    let mut is = IS::default();

    // Variáveis temporárias para todos os campos dos blocos opcionais
    let mut vBCIS: Option<Decimal> = None;
    let mut pIS: Option<Decimal> = None;
    let mut pISEspec: Option<Decimal> = None;
    let mut uTrib: Option<String> = None;
    let mut qTrib: Option<Decimal> = None;
    let mut vIS: Option<Decimal> = None;

    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                let txt = read_text(reader, &e)?;
                match e.name().as_ref() {
                    b"CSTIS" => is.CSTIS = txt,
                    b"cClassTribIS" => is.cClassTribIS = txt,
                    b"vBCIS" => vBCIS = Some(txt.parse()?),
                    b"pIS" => pIS = Some(txt.parse()?),
                    b"pISEspec" => pISEspec = Some(txt.parse()?),
                    b"uTrib" => uTrib = Some(txt),
                    b"qTrib" => qTrib = Some(txt.parse()?),
                    b"vIS" => vIS = Some(txt.parse()?),

                    tag => {
                        let tag_name = String::from_utf8_lossy(tag).to_string();
                        log::warn!("Elemento IS não mapeado: {}", tag_name);
                    }
                }
            }
            Event::End(e) if e.name().as_ref() == b"IS" => {
                if let (Some(vbc), Some(pis_val), Some(vis_val)) = (vBCIS, pIS, vIS) {
                    let unidade_tributavel =
                        if let (Some(utrib_val), Some(qtrib_val)) = (uTrib, qTrib) {
                            Some(UnidadeTributavel {
                                uTrib: utrib_val,
                                qTrib: qtrib_val,
                            })
                        } else {
                            None
                        };

                    is.calculo = Some(CalculoIS {
                        vBCIS: vbc,
                        pIS: pis_val,
                        pISEspec,
                        unidade_tributavel,
                        vIS: vis_val,
                    });
                }
                return Ok(is);
            }
            Event::Eof => return Err(Box::new(ParseError::UnexpectedEof("IS".to_string()))),
            _ => {}
        }
    }
}

fn parse_IBSCBS(reader: &mut XmlReader) -> Result<IBSCBS, Box<dyn Error>> {
    let mut ibscbs = IBSCBS::default();

    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                match e.name().as_ref() {
                    // --- Sub-contêineres (delega para outras funções) ---
                    b"gIBSCBS" => {
                        ibscbs.tributacao = Some(TributacaoIBS::gIBSCBS(parse_gIBSCBS(reader)?))
                    }
                    b"gIBSCBSMono" => {
                        ibscbs.tributacao =
                            Some(TributacaoIBS::gIBSCBSMono(parse_gIBSCBSMono(reader)?))
                    }
                    b"gTransfCred" => {
                        ibscbs.tributacao =
                            Some(TributacaoIBS::gTransfCred(parse_gTransfCred(reader)?))
                    }
                    b"gCredPresIBSZFM" => {
                        ibscbs.gCredPresIBSZFM = Some(parse_gCredPresIBSZFM(reader)?)
                    }

                    // --- Campos finais ---
                    name => {
                        let txt = read_text(reader, &e)?;
                        match name {
                            b"CST" => ibscbs.CST = txt,
                            b"cClassTrib" => ibscbs.cClassTrib = txt,
                            tag => {
                                let tag_name = String::from_utf8_lossy(tag).to_string();
                                log::warn!("Elemento IBSCBS não mapeado: {}", tag_name);
                            }
                        }
                    }
                }
            }
            Event::End(e) if e.name().as_ref() == b"IBSCBS" => {
                return Ok(ibscbs);
            }
            Event::Eof => {
                return Err(Box::new(ParseError::UnexpectedEof("IBSCBS".to_string())));
            }
            _ => {}
        }
    }
}

fn parse_gIBSCBS(reader: &mut XmlReader) -> Result<TCIBS, Box<dyn Error>> {
    let mut tcibs: TCIBS = TCIBS::default();
    loop {
        match reader.read_event()? {
            Event::Start(e) => match e.name().as_ref() {
                b"gIBSUF" => tcibs.gIBSUF = parse_gIBSUF(reader)?,
                b"gIBSMun" => tcibs.gIBSMun = parse_gIBSMun(reader)?,
                b"gCBS" => tcibs.gCBS = parse_gCBS(reader)?,
                b"gTribRegular" => tcibs.gTribRegular = Some(parse_gTribRegular(reader)?),
                b"gIBSCredPres" => tcibs.gIBSCredPres = Some(parse_gCredPres(reader, b"gIBSCredPres")?),
                b"gCBSCredPres" => tcibs.gCBSCredPres = Some(parse_gCredPres(reader, b"gCBSCredPres")?),
                b"gTribCompraGov" => tcibs.gTribCompraGov = Some(parse_gTribCompraGov(reader)?),
                name => {
                    let txt = read_text(reader, &e)?;
                    match name {
                        b"vBC" => tcibs.vBC = txt.parse()?,
                        b"vIBS" => tcibs.vIBS = txt.parse()?,
                        _ => (),
                    }
                }
            },
            Event::End(e) if e.name().as_ref() == b"gIBSCBS" => return Ok(tcibs),
            Event::Eof => {
                return Err(Box::new(ParseError::UnexpectedEof("gIBSCBS".to_string())));
            }
            _ => (),
        }
    }
}

fn parse_gIBSUF(reader: &mut XmlReader) -> Result<GIBSUF, Box<dyn Error>> {
    let mut g: GIBSUF = GIBSUF::default();
    loop {
        match reader.read_event()? {
            Event::Start(e) => match e.name().as_ref() {
                b"gDif" => g.gDif = Some(parse_gDif(reader)?),
                b"gDevTrib" => g.gDevTrib = Some(parse_gDevTrib(reader)?),
                b"gRed" => g.gRed = Some(parse_gRed(reader)?),
                name => {
                    let txt = read_text(reader, &e)?;
                    match name {
                        b"pIBSUF" => g.pIBSUF = txt.parse()?,
                        b"vIBSUF" => g.vIBSUF = txt.parse()?,
                        _ => (),
                    }
                }
            },
            Event::End(e) if e.name().as_ref() == b"gIBSUF" => return Ok(g),

            Event::Eof => {
                return Err(Box::new(ParseError::UnexpectedEof("gIBSUF".to_string())));
            }
            _ => (),
        }
    }
}

fn parse_gIBSMun(reader: &mut XmlReader) -> Result<GIBSMun, Box<dyn Error>> {
    let mut g = GIBSMun::default();
    loop {
        match reader.read_event()? {
            Event::Start(e) => match e.name().as_ref() {
                b"gDif" => g.gDif = Some(parse_gDif(reader)?),
                b"gDevTrib" => g.gDevTrib = Some(parse_gDevTrib(reader)?),
                b"gRed" => g.gRed = Some(parse_gRed(reader)?),
                name => {
                    let txt = read_text(reader, &e)?;
                    match name {
                        b"pIBSMun" => g.pIBSMun = txt.parse()?,
                        b"vIBSMun" => g.vIBSMun = txt.parse()?,
                        _ => (),
                    }
                }
            },
            Event::End(e) if e.name().as_ref() == b"gIBSMun" => return Ok(g),
            Event::Eof => {
                return Err(Box::new(ParseError::UnexpectedEof("gIBSMun".to_string())));
            }

            _ => (),
        }
    }
}

fn parse_gCBS(reader: &mut XmlReader) -> Result<GCBS, Box<dyn Error>> {
    let mut g = GCBS::default();
    loop {
        match reader.read_event()? {
            Event::Start(e) => match e.name().as_ref() {
                b"gDif" => g.gDif = Some(parse_gDif(reader)?),
                b"gDevTrib" => g.gDevTrib = Some(parse_gDevTrib(reader)?),
                b"gRed" => g.gRed = Some(parse_gRed(reader)?),
                name => {
                    let txt = read_text(reader, &e)?;
                    match name {
                        b"pCBS" => g.pCBS = txt.parse()?,
                        b"vCBS" => g.vCBS = txt.parse()?,
                        _ => (),
                    }
                }
            },
            Event::End(e) if e.name().as_ref() == b"gCBS" => return Ok(g),
            Event::Eof => return Err(Box::new(ParseError::UnexpectedEof("gCBS".to_string()))),
            _ => (),
        }
    }
}

fn parse_gTribRegular(reader: &mut XmlReader) -> Result<TTribRegular, Box<dyn Error>> {
    let mut g: TTribRegular = TTribRegular::default();
    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                let txt = read_text(reader, &e)?;
                match e.name().as_ref() {
                    b"CSTReg" => g.CSTReg = txt,
                    b"cClassTribReg" => g.cClassTribReg = txt,
                    b"pAliqEfetRegIBSUF" => g.pAliqEfetRegIBSUF = txt.parse()?,
                    b"vTribRegIBSUF" => g.vTribRegIBSUF = txt.parse()?,
                    b"pAliqEfetRegIBSMun" => g.pAliqEfetRegIBSMun = txt.parse()?,
                    b"vTribRegIBSMun" => g.vTribRegIBSMun = txt.parse()?,
                    b"pAliqEfetRegCBS" => g.pAliqEfetRegCBS = txt.parse()?,
                    b"vTribRegCBS" => g.vTribRegCBS = txt.parse()?,
                    _ => (),
                }
            }
            Event::End(e) if e.name().as_ref() == b"gTribRegular" => return Ok(g),
            Event::Eof => {
                return Err(Box::new(ParseError::UnexpectedEof(
                    "gTribRegular".to_string(),
                )));
            }
            _ => (),
        }
    }
}

fn parse_gDif(reader: &mut XmlReader) -> Result<TDif, Box<dyn Error>> {
    let mut g = TDif::default();
    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                let txt = read_text(reader, &e)?;
                match e.name().as_ref() {
                    b"pDif" => g.pDif = txt.parse()?,
                    b"vDif" => g.vDif = txt.parse()?,
                    _ => (),
                }
            }
            Event::End(e) if e.name().as_ref() == b"gDif" => return Ok(g),
            Event::Eof => return Err(Box::new(ParseError::UnexpectedEof("gDif".to_string()))),

            _ => (),
        }
    }
}

fn parse_gDevTrib(reader: &mut XmlReader) -> Result<TDevTrib, Box<dyn Error>> {
    let mut g: TDevTrib = TDevTrib::default();
    loop {
        match reader.read_event()? {
            Event::Start(e) if e.name().as_ref() == b"vDevTrib" => {
                g.vDevTrib = read_text(reader, &e)?.parse()?;
            }
            Event::End(e) if e.name().as_ref() == b"gDevTrib" => return Ok(g),

            Event::Eof => {
                return Err(Box::new(ParseError::UnexpectedEof("gDevTrib".to_string())));
            }
            _ => (),
        }
    }
}

fn parse_gRed(reader: &mut XmlReader) -> Result<TRed, Box<dyn Error>> {
    let mut g: TRed = TRed::default();
    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                let txt = read_text(reader, &e)?;
                match e.name().as_ref() {
                    b"pRedAliq" => g.pRedAliq = txt.parse()?,
                    b"pAliqEfet" => g.pAliqEfet = txt.parse()?,
                    _ => (),
                }
            }
            Event::End(e) if e.name().as_ref() == b"gRed" => return Ok(g),
            Event::Eof => return Err(Box::new(ParseError::UnexpectedEof("gRed".to_string()))),
            _ => (),
        }
    }
}

fn parse_gIBSCBSMono(reader: &mut XmlReader) -> Result<TMonofasia, Box<dyn Error>> {
    let mut monofasia = TMonofasia::default();

    loop {
        match reader.read_event()? {
            Event::Start(e) => match e.name().as_ref() {
                b"gMonoPadrao" => monofasia.gMonoPadrao = Some(parse_GMonoPadrao(reader)?),
                b"gMonoReten" => monofasia.gMonoReten = Some(parse_GMonoReten(reader)?),
                b"gMonoRet" => monofasia.gMonoRet = Some(parse_GMonoRet(reader)?),
                b"gMonoDif" => monofasia.gMonoDif = Some(parse_GMonoDif(reader)?),

                // Campos finais
                name => {
                    let txt = read_text(reader, &e)?;
                    match name {
                        b"vTotIBSMonoItem" => monofasia.vTotIBSMonoItem = txt.parse::<Decimal>()?,
                        b"vTotCBSMonoItem" => monofasia.vTotCBSMonoItem = txt.parse::<Decimal>()?,
                        _ => (),
                    }
                }
            },
            Event::End(e) if e.name().as_ref() == b"gIBSCBSMono" => return Ok(monofasia),
            Event::Eof => {
                return Err(Box::new(ParseError::UnexpectedEof(
                    "gIBSCBSMono".to_string(),
                )));
            }
            _ => (),
        }
    }
}

// --- Funções Auxiliares ---

fn parse_GMonoPadrao(reader: &mut XmlReader) -> Result<GMonoPadrao, Box<dyn Error>> {
    let mut g: GMonoPadrao = GMonoPadrao::default();
    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                let txt = read_text(reader, &e)?;
                match e.name().as_ref() {
                    b"qBCMono" => g.qBCMono = txt.parse::<Decimal>()?,
                    b"adRemIBS" => g.adRemIBS = txt.parse::<Decimal>()?,
                    b"adRemCBS" => g.adRemCBS = txt.parse::<Decimal>()?,
                    b"vIBSMono" => g.vIBSMono = txt.parse::<Decimal>()?,
                    b"vCBSMono" => g.vCBSMono = txt.parse::<Decimal>()?,
                    _ => (),
                }
            }
            Event::End(e) if e.name().as_ref() == b"gMonoPadrao" => return Ok(g),

            Event::Eof => {
                return Err(Box::new(ParseError::UnexpectedEof(
                    "GMonoPadrao".to_string(),
                )));
            }
            _ => (),
        }
    }
}

fn parse_GMonoReten(reader: &mut XmlReader) -> Result<GMonoReten, Box<dyn Error>> {
    let mut g: GMonoReten = GMonoReten::default();
    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                let txt = read_text(reader, &e)?;
                match e.name().as_ref() {
                    b"qBCMonoReten" => g.qBCMonoReten = txt.parse::<Decimal>()?,
                    b"adRemIBSReten" => g.adRemIBSReten = txt.parse::<Decimal>()?,
                    b"vIBSMonoReten" => g.vIBSMonoReten = txt.parse::<Decimal>()?,
                    b"adRemCBSReten" => g.adRemCBSReten = txt.parse::<Decimal>()?,
                    b"vCBSMonoReten" => g.vCBSMonoReten = txt.parse::<Decimal>()?,
                    _ => (),
                }
            }
            Event::End(e) if e.name().as_ref() == b"gMonoReten" => return Ok(g),
            Event::Eof => {
                return Err(Box::new(ParseError::UnexpectedEof(
                    "GMonoReten".to_string(),
                )));
            }
            _ => (),
        }
    }
}

fn parse_GMonoRet(reader: &mut XmlReader) -> Result<GMonoRet, Box<dyn Error>> {
    let mut g: GMonoRet = GMonoRet::default();
    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                let txt = read_text(reader, &e)?;
                match e.name().as_ref() {
                    b"qBCMonoRet" => g.qBCMonoRet = txt.parse::<Decimal>()?,
                    b"adRemIBSRet" => g.adRemIBSRet = txt.parse::<Decimal>()?,
                    b"vIBSMonoRet" => g.vIBSMonoRet = txt.parse::<Decimal>()?,
                    b"adRemCBSRet" => g.adRemCBSRet = txt.parse::<Decimal>()?,
                    b"vCBSMonoRet" => g.vCBSMonoRet = txt.parse::<Decimal>()?,
                    _ => (),
                }
            }
            Event::End(e) if e.name().as_ref() == b"gMonoRet" => return Ok(g),
            Event::Eof => {
                return Err(Box::new(ParseError::UnexpectedEof("GMonoRet".to_string())));
            }

            _ => (),
        }
    }
}

fn parse_GMonoDif(reader: &mut XmlReader) -> Result<GMonoDif, Box<dyn Error>> {
    let mut g: GMonoDif = GMonoDif::default();
    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                let txt = read_text(reader, &e)?;
                match e.name().as_ref() {
                    b"pDifIBS" => g.pDifIBS = txt.parse::<Decimal>()?,
                    b"vIBSMonoDif" => g.vIBSMonoDif = txt.parse::<Decimal>()?,
                    b"pDifCBS" => g.pDifCBS = txt.parse::<Decimal>()?,
                    b"vCBSMonoDif" => g.vCBSMonoDif = txt.parse::<Decimal>()?,
                    _ => (),
                }
            }
            Event::End(e) if e.name().as_ref() == b"gMonoDif" => return Ok(g),
            Event::Eof => {
                return Err(Box::new(ParseError::UnexpectedEof("GMonoDif".to_string())));
            }
            _ => (),
        }
    }
}

fn parse_gTransfCred(reader: &mut XmlReader) -> Result<TTransfCred, Box<dyn Error>> {
    let mut transf_cred: TTransfCred = TTransfCred::default();
    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                let txt = read_text(reader, &e)?;
                match e.name().as_ref() {
                    b"vIBS" => transf_cred.vIBS = txt.parse()?,
                    b"vCBS" => transf_cred.vCBS = txt.parse()?,
                    _ => (),
                }
            }
            Event::End(e) if e.name().as_ref() == b"gTransfCred" => return Ok(transf_cred),
            Event::Eof => return Err(Box::new(ParseError::UnexpectedEof("gTransfCred".to_string()))),
            
            _ => (),
        }
    }
}

fn parse_gCredPresIBSZFM(reader: &mut XmlReader) -> Result<TCredPresIBSZFM, Box<dyn Error>> {
    let mut cred_pres: TCredPresIBSZFM = TCredPresIBSZFM::default();
    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                let txt = read_text(reader, &e)?;
                match e.name().as_ref() {
                    b"tpCredPresIBSZFM" => cred_pres.tpCredPresIBSZFM = txt,
                    b"vCredPresIBSZFM" => cred_pres.vCredPresIBSZFM = Some(txt.parse()?),
                    _ => (),
                }
            }
            Event::End(e) if e.name().as_ref() == b"gCredPresIBSZFM" => return Ok(cred_pres),
            Event::Eof => return Err(Box::new(ParseError::UnexpectedEof("gCredPresIBSZFM".to_string()))),
            _ => (),
        }
    }
}

fn parse_gCredPres(reader: &mut XmlReader, end_tag: &[u8]) -> Result<TCredPres, Box<dyn Error>> {
    let mut g: TCredPres = TCredPres::default();
    let mut vCredPres: Option<Decimal> = None;
    let mut vCredPresCondSus: Option<Decimal> = None;

    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                let txt = read_text(reader, &e)?;
                match e.name().as_ref() {
                    b"cCredPres" => g.cCredPres = txt,
                    b"pCredPres" => g.pCredPres = txt.parse()?,
                    b"vCredPres" => vCredPres = Some(txt.parse()?),
                    b"vCredPresCondSus" => vCredPresCondSus = Some(txt.parse()?),
                    _ => (),
                }
            }
            Event::End(e) if e.name().as_ref() == end_tag => {
                if let Some(val) = vCredPres {
                    g.valor = ValorCredPres::vCredPres(val);
                } else if let Some(val) = vCredPresCondSus {
                    g.valor = ValorCredPres::vCredPresCondSus(val);
                } else {
                    // O schema define a <choice> como obrigatória, então um dos dois deve existir.
                    return Err("Estrutura de TCredPres inválida: vCredPres ou vCredPresCondSus não encontrado".into());
                }
                return Ok(g);
            }

            Event::Eof => return Err(Box::new(ParseError::UnexpectedEof("gCredPres".to_string()))),
            _ => (),
        }
    }
}


fn parse_gTribCompraGov(reader: &mut XmlReader) -> Result<TTribCompraGov, Box<dyn Error>> {
    let mut g: TTribCompraGov = TTribCompraGov::default();
    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                let txt = read_text(reader, &e)?;
                match e.name().as_ref() {
                    b"pAliqIBSUF" => g.pAliqIBSUF = txt.parse::<Decimal>()?,
                    b"vTribIBSUF" => g.vTribIBSUF = txt.parse::<Decimal>()?,
                    b"pAliqIBSMun" => g.pAliqIBSMun = txt.parse::<Decimal>()?,
                    b"vTribIBSMun" => g.vTribIBSMun = txt.parse::<Decimal>()?,
                    b"pAliqCBS" => g.pAliqCBS = txt.parse::<Decimal>()?,
                    b"vTribCBS" => g.vTribCBS = txt.parse::<Decimal>()?,
                    _ => (),
                }
            }
            Event::End(e) if e.name().as_ref() == b"gTribCompraGov" => return Ok(g),
            _ => (),
        }
    }
}

fn parse_COFINSAliq(reader: &mut XmlReader) -> Result<COFINSAliq, Box<dyn Error>> {
    let mut cofins_aliq: COFINSAliq = COFINSAliq::default();
    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                let txt = read_text(reader, &e)?;
                match e.name().as_ref() {
                    b"CST" => cofins_aliq.CST = txt,
                    b"vBC" => cofins_aliq.vBC = txt.parse()?,
                    b"pCOFINS" => cofins_aliq.pCOFINS = txt.parse()?,
                    b"vCOFINS" => cofins_aliq.vCOFINS = txt.parse()?,
                    _ => (),
                }
            }
            Event::End(e) if e.name().as_ref() == b"COFINSAliq" => return Ok(cofins_aliq),
            Event::Eof => {
                return Err(Box::new(ParseError::UnexpectedEof(
                    "COFINSAliq".to_string(),
                )));
            }
            _ => (),
        }
    }
}

fn parse_COFINSQtde(reader: &mut XmlReader) -> Result<COFINSQtde, Box<dyn Error>> {
    let mut cofins_qtde = COFINSQtde::default();
    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                let txt = read_text(reader, &e)?;
                match e.name().as_ref() {
                    b"CST" => cofins_qtde.CST = txt,
                    b"qBCProd" => cofins_qtde.qBCProd = txt.parse()?,
                    b"vAliqProd" => cofins_qtde.vAliqProd = txt.parse()?,
                    b"vCOFINS" => cofins_qtde.vCOFINS = txt.parse()?,
                    _ => (),
                }
            }
            Event::End(e) if e.name().as_ref() == b"COFINSQtde" => return Ok(cofins_qtde),
            Event::Eof => {
                return Err(Box::new(ParseError::UnexpectedEof(
                    "COFINSQtde".to_string(),
                )));
            }
            _ => (),
        }
    }
}
fn parse_COFINSNT(reader: &mut XmlReader) -> Result<String, Box<dyn Error>> {
    let mut cst: String = String::new();
    loop {
        match reader.read_event()? {
            Event::Start(e) if e.name().as_ref() == b"CST" => {
                cst = read_text(reader, &e)?;
            }
            Event::End(e) if e.name().as_ref() == b"COFINSNT" => return Ok(cst),
            Event::Eof => {
                return Err(Box::new(ParseError::UnexpectedEof("COFINSNT".to_string())));
            }
            _ => (),
        }
    }
}
fn parse_COFINSOutr(reader: &mut XmlReader) -> Result<COFINSOutr, Box<dyn Error>> {
    let mut cofins_outr = COFINSOutr::default();
    let mut vBC: Option<Decimal> = None;
    let mut pCOFINS: Option<Decimal> = None;
    let mut qBCProd: Option<Decimal> = None;
    let mut vAliqProd: Option<Decimal> = None;

    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                let txt = read_text(reader, &e)?;
                match e.name().as_ref() {
                    b"CST" => cofins_outr.CST = txt,
                    b"vCOFINS" => cofins_outr.vCOFINS = txt.parse()?,
                    b"vBC" => vBC = Some(txt.parse()?),
                    b"pCOFINS" => pCOFINS = Some(txt.parse()?),
                    b"qBCProd" => qBCProd = Some(txt.parse()?),
                    b"vAliqProd" => vAliqProd = Some(txt.parse()?),
                    _ => (),
                }
            }
            Event::End(e) if e.name().as_ref() == b"COFINSOutr" => {
                if let (Some(vbc_val), Some(pcofins_val)) = (vBC, pCOFINS) {
                    cofins_outr.calculo = CalculoCOFINSOutr::Aliquota {
                        vBC: vbc_val,
                        pCOFINS: pcofins_val,
                    };
                } else if let (Some(qbc_val), Some(valiq_val)) = (qBCProd, vAliqProd) {
                    cofins_outr.calculo = CalculoCOFINSOutr::Unidade {
                        qBCProd: qbc_val,
                        vAliqProd: valiq_val,
                    };
                } else {
                    return Err("Estrutura de cálculo de COFINSOutr inválida".into());
                }
                return Ok(cofins_outr);
            }
            Event::Eof => {
                return Err(Box::new(ParseError::UnexpectedEof(
                    "COFINSOutr".to_string(),
                )));
            }
            _ => (),
        }
    }
}

fn parse_PISAliq(reader: &mut XmlReader) -> Result<PISAliq, Box<dyn Error>> {
    let mut pis_aliq: PISAliq = PISAliq::default();
    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                let tag = e.name();
                let txt: String = read_text(reader, &e)?;
                match tag.as_ref() {
                    b"CST" => pis_aliq.CST = txt,
                    b"vBC" => pis_aliq.vBC = txt.parse::<Decimal>()?,
                    b"pPIS" => pis_aliq.pPIS = txt.parse::<Decimal>()?,
                    b"vPIS" => pis_aliq.vPIS = txt.parse::<Decimal>()?,

                    _ => {
                        let tag: &str = std::str::from_utf8(tag.as_ref())?;
                        log::warn!("Elemento PISAliq não mapeado: {}", tag)
                    }
                }
            }
            Event::End(e) if e.name().as_ref() == b"PISAliq" => return Ok(pis_aliq),

            Event::Eof => {
                return Err(Box::new(ParseError::UnexpectedEof("PISAliq".to_string())));
            }

            _ => {}
        }
    }
}

fn parse_PISQtde(reader: &mut XmlReader) -> Result<PISQtde, Box<dyn Error>> {
    let mut pis_qtde: PISQtde = PISQtde::default();

    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                let tag = e.name();
                let txt: String = read_text(reader, &e)?;
                match tag.as_ref() {
                    b"CST" => pis_qtde.CST = txt,
                    b"qBCProd" => pis_qtde.qBCProd = txt.parse::<Decimal>()?,
                    b"vAliqProd" => pis_qtde.vAliqProd = txt.parse::<Decimal>()?,
                    b"vPIS" => pis_qtde.vPIS = txt.parse::<Decimal>()?,

                    _ => {
                        let tag: &str = std::str::from_utf8(tag.as_ref())?;
                        log::warn!("Elemento PISQtde não mapeado: {}", tag)
                    }
                }
            }
            Event::End(e) if e.name().as_ref() == b"PISQtde" => return Ok(pis_qtde),

            Event::Eof => {
                return Err(Box::new(ParseError::UnexpectedEof("PISQtde".to_string())));
            }

            _ => {}
        }
    }
}

fn parse_PISOutr(reader: &mut XmlReader) -> Result<PISOutr, Box<dyn Error>> {
    let mut pis_outr: PISOutr = PISOutr::default();

    let mut vBC: Option<Decimal> = None;
    let mut pPIS: Option<Decimal> = None;

    let mut qBCProd: Option<Decimal> = None;
    let mut vAliqProd: Option<Decimal> = None;

    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                let tag = e.name();
                let txt: String = read_text(reader, &e)?;
                match tag.as_ref() {
                    b"CST" => pis_outr.CST = txt,
                    b"vPIS" => pis_outr.vPIS = txt.parse::<Decimal>()?,

                    b"vBC" => vBC = Some(txt.parse::<Decimal>()?),
                    b"pPIS" => pPIS = Some(txt.parse::<Decimal>()?),

                    b"qBCProd" => qBCProd = Some(txt.parse::<Decimal>()?),
                    b"vAliqProd" => vAliqProd = Some(txt.parse::<Decimal>()?),

                    _ => {}
                }
            }
            Event::End(e) if e.name().as_ref() == b"PISOutr" => {
                pis_outr.calculo = if let (Some(vBC), Some(pPIS)) = (vBC, pPIS) {
                    CalculoPISOutr::Aliquota {
                        vBC: vBC,
                        pPIS: pPIS,
                    }
                } else if let (Some(qBCProd), Some(vAliqProd)) = (qBCProd, vAliqProd) {
                    CalculoPISOutr::Unidade {
                        qBCProd: qBCProd,
                        vAliqProd: vAliqProd,
                    }
                } else {
                    return Err(Box::new(ParseError::ModeloDesconhecido));
                };
                return Ok(pis_outr);
            }

            Event::Eof => {
                return Err(Box::new(ParseError::UnexpectedEof("PISOutr".to_string())));
            }

            _ => {}
        }
    }
}

fn parse_IPITrib(reader: &mut XmlReader) -> Result<IPITrib, Box<dyn Error>> {
    let mut ipi_trib: IPITrib = IPITrib::default();
    let mut vBC: Option<Decimal> = None;
    let mut pIPI: Option<Decimal> = None;
    let mut qUnid: Option<Decimal> = None;
    let mut vUnid: Option<Decimal> = None;

    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                let name = e.name();
                let txt = read_text(reader, &e)?;

                match name.as_ref() {
                    b"CST" => ipi_trib.CST = txt,
                    b"vIPI" => ipi_trib.vIPI = txt.parse::<Decimal>()?,

                    // Sequence choice
                    b"vBC" => vBC = Some(txt.parse()?),
                    b"pIPI" => pIPI = Some(txt.parse()?),
                    b"qUnid" => qUnid = Some(txt.parse()?),
                    b"vUnid" => vUnid = Some(txt.parse()?),
                    _ => {}
                }
            }

            Event::End(e) if e.name().as_ref() == b"IPITrib" => {
                // Tenta construir a variante 'Aliquota'
                if let (Some(vbc_val), Some(pipi_val)) = (vBC, pIPI) {
                    ipi_trib.calculo = CalculoIpi::Aliquota {
                        vBC: vbc_val,
                        pIPI: pipi_val,
                    };
                // Senão, tenta construir a variante 'Unidade'
                } else if let (Some(qunid_val), Some(vunid_val)) = (qUnid, vUnid) {
                    ipi_trib.calculo = CalculoIpi::Unidade {
                        qUnid: qunid_val,
                        vUnid: vunid_val,
                    };
                // Se nenhum par de campos foi encontrado, o XML é inválido
                } else {
                    return Err("Estrutura de cálculo do IPITrib inválida".into());
                }

                return Ok(ipi_trib);
            }

            Event::Eof => {
                return Err(Box::new(ParseError::UnexpectedEof("IPITrib".to_string())));
            }

            _ => {}
        }
    }
}

fn parse_enderEmit(reader: &mut XmlReader, end_tag: &[u8]) -> Result<EnderEmi, Box<dyn Error>> {
    let mut enderEmi: EnderEmi = EnderEmi::default();
    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                let name = e.name();
                let txt: String = read_text(reader, &e)?;

                match name.as_ref() {
                    b"xLgr" => enderEmi.xLgr = txt,
                    b"nro" => enderEmi.nro = txt,
                    b"xCpl" => enderEmi.xCpl = Some(txt),
                    b"xBairro" => enderEmi.xBairro = txt,
                    b"cMun" => enderEmi.cMun = txt.parse::<u32>()?,
                    b"xMun" => enderEmi.xMun = txt,
                    b"UF" => enderEmi.UF = UF::from(txt.as_str()),
                    b"CEP" => enderEmi.CEP = Some(txt),
                    b"cPais" => enderEmi.cPais = Some(txt),
                    b"xPais" => enderEmi.xPais = Some(txt),
                    b"fone" => enderEmi.fone = Some(txt),
                    _ => {}
                }
            }

            Event::End(e) if e.name().as_ref() == end_tag => return Ok(enderEmi),

            Event::Eof => {
                return Err(Box::new(ParseError::UnexpectedEof(String::from_utf8_lossy(end_tag).to_string())));
            }
            _ => {}
        }
    }
}

fn parse_TLocal(reader: &mut XmlReader, end_tag: &[u8]) -> Result<Local, Box<dyn Error>> {
    let mut local = Local::default();
    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                let txt = read_text(reader, &e)?;
                match e.name().as_ref() {
                    // Tratamento da <choice> de identificação
                    b"CNPJ" => local.EmitenteId = EmitenteId::CNPJ(txt),
                    b"CPF" => local.EmitenteId = EmitenteId::CPF(txt),


                    // Outros campos
                    b"xNome" => local.xNome = Some(txt),
                    b"xLgr" => local.xLgr = txt,
                    b"nro" => local.nro = txt,
                    b"xCpl" => local.xCpl = Some(txt),
                    b"xBairro" => local.xBairro = txt,
                    b"cMun" => local.cMun = txt.parse()?,
                    b"xMun" => local.xMun = txt,
                    b"UF" => local.UF = UF::from(txt.as_str()),
                    b"CEP" => local.CEP = Some(txt),
                    b"cPais" => local.cPais = Some(txt),
                    b"xPais" => local.xPais = Some(txt),
                    b"fone" => local.fone = Some(txt),
                    b"email" => local.email = Some(txt),
                    b"IE" => local.IE = Some(txt),
                    _ => (),
                }
            }
            // Usa o argumento 'end_tag' para a condição de parada
            Event::End(e) if e.name().as_ref() == end_tag => return Ok(local),
            Event::Eof => return Err(Box::new(ParseError::UnexpectedEof(String::from_utf8_lossy(end_tag).to_string()))),
            _ => (),
        }
    }
}

fn parse_nfref(reader: &mut XmlReader) -> Result<NFRef, Box<dyn Error>> {
    loop {
        match reader.read_event()? {
            Event::Start(e) => match e.name().as_ref() {
                b"refNF" => return parse_refNF(reader),
                b"refNFP" => return parse_refNFP(reader),
                b"refECF" => return parse_refECF(reader),

                name => {
                    let txt: String = read_text(reader, &e)?;
                    match name {
                        b"refNFe" => return Ok(NFRef::refNFe(txt)),
                        b"refNFeSig" => return Ok(NFRef::refNFeSig(txt)),
                        b"refCTe" => return Ok(NFRef::refCTe(txt)),
                        _ => {
                            break;
                        } // Desconhecido. Forçar erro
                    }
                }
            },

            Event::Eof => return Err(Box::new(ParseError::UnexpectedEof("NFref".to_string()))),
            _ => {}
        }
    }
    panic!("Unexpected error while parsing NFRef.");
}

fn parse_refNF(reader: &mut XmlReader) -> Result<NFRef, Box<dyn Error>> {
    let mut refNF: RefNFData = RefNFData::default();

    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                let txt = read_text(reader, &e)?;
                match e.name().as_ref() {
                    b"cUF" => refNF.cUF = txt.parse::<u8>()?,
                    b"AAMM" => refNF.AAMM = txt,
                    b"CNPJ" => refNF.CNPJ = txt,
                    b"mod" => refNF.r#mod = txt.parse::<u8>()?,
                    b"serie" => refNF.serie = txt.parse::<u16>()?,
                    b"nNF" => refNF.nNF = txt.parse::<u32>()?,
                    _ => {
                        break;
                    }
                }
            }

            Event::End(e) if e.name().as_ref() == b"refNF" => {
                return Ok(NFRef::refNF(refNF));
            }

            Event::Eof => {
                log::error!("Unexpected Eof while parsing refNF");
                break;
            }
            _ => {}
        }
    }
    panic!("Unexpected error while parsing refNF.");
}

fn parse_refNFP(reader: &mut XmlReader) -> Result<NFRef, Box<dyn Error>> {
    let mut refNFP: RefNFPData = RefNFPData::default();

    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                let txt = read_text(reader, &e)?;
                match e.name().as_ref() {
                    b"cUF" => refNFP.cUF = txt.parse::<u8>()?,
                    b"AAMM" => refNFP.AAMM = txt,
                    b"CNPJ" => refNFP.EmitenteId = EmitenteId::CNPJ(txt),
                    b"CPF" => refNFP.EmitenteId = EmitenteId::CPF(txt),
                    b"IE" => refNFP.IE = txt,
                    b"mod" => refNFP.r#mod = txt.parse::<u8>()?,
                    b"serie" => refNFP.serie = txt.parse::<u16>()?,
                    b"nNF" => refNFP.nNF = txt.parse::<u32>()?,
                    _ => {
                        break;
                    }
                }
            }

            Event::End(e) if e.name().as_ref() == b"refNFP" => {
                return Ok(NFRef::refNFP(refNFP));
            }

            Event::Eof => {
                log::error!("Unexpected Eof while parsing refNFP");
                break;
            }
            _ => {}
        }
    }
    panic!("Unexpected error while parsing refNFP.");
}

fn parse_refECF(reader: &mut XmlReader) -> Result<NFRef, Box<dyn Error>> {
    let mut refECF: RefECFData = RefECFData::default();

    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                let txt = read_text(reader, &e)?;
                match e.name().as_ref() {
                    b"mod" => refECF.r#mod = txt,
                    b"nECF" => refECF.nECF = txt,
                    b"nCOO" => refECF.nCOO = txt,
                    _ => {
                        break;
                    }
                }
            }

            Event::End(e) if e.name().as_ref() == b"refECF" => {
                return Ok(NFRef::refECF(refECF));
            }

            Event::Eof => {
                log::error!("Unexpected Eof while parsing refECF");
                break;
            }
            _ => {}
        }
    }
    panic!("Unexpected error while parsing refECF.");
}

fn parse_gCompraGov(reader: &mut XmlReader) -> Result<CompraGov, Box<dyn Error>> {
    let mut cg: CompraGov = CompraGov::default();

    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                let txt: String = read_text(reader, &e)?;

                match e.name().as_ref() {
                    b"tpEnteGov" => cg.tpEnteGov = txt.parse()?,
                    b"pRedutor" => cg.pRedutor = txt.parse()?,
                    b"tpOperGov" => cg.tpOperGov = txt.parse()?,
                    _ => {
                        log::warn!(
                            "Elemento CompraGov não mapeado: {}",
                            std::str::from_utf8(e.name().as_ref())?
                        );
                        break;
                    }
                }
            }

            Event::End(e) if e.name().as_ref() == b"gCompraGov" => {
                return Ok(cg);
            }

            Event::Eof => {
                return Err(Box::new(ParseError::UnexpectedEof(
                    "gCompraGov".to_string(),
                )));
            }

            _ => {}
        }
    }
    panic!("Unexpected error while parsing gCompraGov.");
}

fn parse_gPagAntecipado(reader: &mut XmlReader) -> Result<Vec<String>, Box<dyn Error>> {
    let mut refNfes: Vec<String> = Vec::new();

    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                if e.name().as_ref() == b"refNFe" {
                    refNfes.push(read_text(reader, &e)?);
                }
            }

            // Tag terminou
            Event::End(e) if e.name().as_ref() == b"gPagAntecipado" => 
                return Ok(refNfes),

            Event::Eof => return Err(Box::new(ParseError::UnexpectedEof("gPagAntecipado".to_string(),))),

            _ => {}
        }
    }
}


