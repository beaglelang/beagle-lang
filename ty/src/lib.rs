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
    ///In Beagle, references are integral dependent types, thus they deserve their own spot in TyValueElement
    ///The String value is a term to which the reference binds to.
    ///Dependent types:
    ///```norust
    ///     a: A
    ///     x: r(a)
    ///```  
    ///In this case, we are making an object with the term `a` and type `A`. Then we create an object with term `x` and type `r(a)`.
    ///Where `r` is a dependent type that binds to the term `a`.
    ///
    ///More formally, this can be expressed as, "there exists, for every term 'a' of type 'A', a type that binds to term 'a'"
    ///```norust
    ///     ∃a∈A r(a)
    ///``` 
    ///Furthermore, we can express a reference as a relation between two terms, where the domain of such relation takes on the type of the codomain.
    ///```norust
    ///     x -> a
    ///```
    ///In this case, we have a relation between term 'x' and term 'a', where term 'a' is type 'A', and 'x' is type 'r(a)'.
    ///What we can extrapolate from this, is the ability to acquire the contravariant relation between an object and it's references
    ///```norust
    ///     x <- a
    ///```
    ///However, without the proof for this, this relation means nothing. So here is a bit of logic:
    /// "There exists for every reference termed 'r' of type 'r(a)' an object termed 'a' of type 'A'"
    ///```norust
    ///     ∃r∈r(a) a: A
    ///```
    ///References can thus be described as bidirectional relations with objects.
    Ref(String),
    Unit
}

impl PartialEq<Ty> for Ty{
    fn eq(&self, other: &Ty) -> bool {
        self.ident == other.ident
    }
}