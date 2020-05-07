
#[derive(PartialEq, Debug, Clone)]
pub enum SymbolKind{
    Property,
    Function(SymbolTable),
    Local,
}

#[derive(PartialEq, Debug, Clone)]
pub struct Symbol{
    pub name: String,
    pub type_: String,
    pub kind: SymbolKind
}

#[derive(PartialEq, Debug, Clone)]
pub struct SymbolTable{
    symbols: Vec<Symbol>,
}

impl SymbolTable{
    pub fn new() -> Self {
        Self{
            symbols: Vec::new()
        }
    }

    pub fn push_property(&mut self, name: String, type_: String){
        self.symbols.push(Symbol{
            name,
            type_,
            kind: SymbolKind::Property
        });
    }

    pub fn push_function(&mut self, name: String, return_type: String){
        self.symbols.push(Symbol{
            name,
            type_: return_type,
            kind: SymbolKind::Property
        });
    }

    pub fn push_local(&mut self, func_name: String, local_name: String, type_: String){
        for symbol in self.symbols.iter(){
            if *symbol.name == func_name{
                if let SymbolKind::Function(mut table) = symbol.clone().kind{
                    &table.symbols.push(Symbol{
                        name: local_name,
                        type_,
                        kind: SymbolKind::Property
                    });
                    break;
                }
            }
        }
    }
}