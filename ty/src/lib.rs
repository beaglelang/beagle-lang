use core::pos::BiPos;
///A type, the meat of the sandwhich.
///Ty represents a type which is used to represent a specific type. Type info is generated or inferred by context.
///A Ty can be inferred or generated depending upon building blocks or sister componenets. Smart casting using the given context to ensure that while within a conditional block that checks for a type's instance, that we safely cast an object's type to the checked type.
///```
/// if(a is B){
///     //Object 'a' was checked in the condition against the type B, so therefore we can safely smart cast 'a' to type B.
///     a.doSomething()
/// }
///```
#[derive(Debug, Clone)]
pub struct Ty{
    ///The name of the type. This is used for comparison.
    pub ident: String,
    ///The location in source code of the type. This can be one of the following:
    /// * A type annotation
    ///     * Function return type
    ///     * Property/local type annotation
    ///     * Class/Struct constructor
    /// * Reference
    ///     * Referencing an object
    ///     * Calling a function
    ///     * Metaprogramming features
    pub pos: BiPos
}

///A value from input and it's type.
///This can include primitive data such as literals, Unit values (aka, void or nothing), or custom types when parsing Constructors.
///See [TyValueElement] for more information.
#[derive(Debug, Clone, PartialEq)]
pub struct TyValue{
    pub ty: Ty,
    pub elem: TyValueElement
}

///An element of a [TyValue]. 
///This has to do with primitive data since primitives are built in. This also allows for convenience when checking for non-primitives types.
///If you have a class called `A`, then the TyValueElement for `A` would be `TyValueElement::Custom("A"). This will then be used during type checking to ensure that
///Values such as calling A's constructor is matched with the required type of the statement.
///```
/////This will have a TyValueElement of Custom("A") during the type checking of the constructor.
///let a = A()
///```
#[derive(Debug, Clone, PartialEq)]
pub enum TyValueElement{
    Integer(i32),
    Float(f32),
    String(String),
    Bool(bool),
    Custom(String),
    Unit
}

impl PartialEq<Ty> for Ty{
    fn eq(&self, other: &Ty) -> bool {
        self.ident == other.ident
    }
}