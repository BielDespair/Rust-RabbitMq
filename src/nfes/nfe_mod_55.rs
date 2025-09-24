


#[derive(Debug)]
pub struct Ide {
    pub cUF: String,
    pub natOp: String,
}


impl Default for Ide {
    fn default() -> Self {
        Ide {
            cUF: String::new(),
            natOp: String::new(),
        }
    }
}

#[derive(Debug)]
pub struct NfeMod55 {
    pub ide: Ide,
}

// Também podemos derivar Default para NfeMod55 se todos os campos tiverem Default
impl Default for NfeMod55 {
    fn default() -> Self {
        NfeMod55 {
            ide: Ide::default(),
        }
    }
}