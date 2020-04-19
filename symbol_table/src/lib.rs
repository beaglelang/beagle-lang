use ir::hir::HIR;

#[derive(Debug, Clone)]
pub enum SymbolKind<'a>{
    Property,
    Function(SymbolTable<'a>),
    Local,
}

#[derive(Debug, Clone)]
pub struct SymbolTable<'a>{
    names: Vec<String>,
    hirs: Vec<&'a HIR>,
    kinds: Vec<SymbolKind<'a>>
}

impl<'a> SymbolTable<'a>{
    pub fn push_property(&mut self, name: String, hir: &'a HIR){
        self.names.push(name);
        self.hirs.push(hir);
        self.kinds.push(SymbolKind::Property);
    }

    pub fn push_function(&mut self, name: String, hir: &'a HIR){
        self.names.push(name);
        self.hirs.push(hir);
        self.kinds.push(SymbolKind::Function(SymbolTable::default()));
    }

    pub fn push_local(&mut self, func_name: String, local_name: String, hir: &'a HIR){
        for (name, kind) in self.names.iter().zip(self.kinds.clone()){
            if *name == func_name{
                if let SymbolKind::Function(mut table) = kind{
                    table.names.push(local_name.clone());
                    table.hirs.push(hir);
                    table.kinds.push(SymbolKind::Local);
                }
            }
        }
    }
}

impl<'a> Default for SymbolTable<'a>{
    fn default() -> Self {
        Self{
            names: Vec::new(),
            hirs: Vec::new(),
            kinds: Vec::new()
        }
    }
}