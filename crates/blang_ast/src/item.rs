//! Item (declaration) types for the AST.

use crate::attr::Attr;
use crate::expr::{Block, Expr};
use crate::lit::Lit;
use crate::pat::Pat;
use crate::path::{Ident, Path};
use crate::ty::{GenericParam, Ty, TraitBound};
use blang_span::Span;
use serde::{Deserialize, Serialize};

/// An item (top-level declaration).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Item {
    /// The kind of item.
    pub kind: ItemKind,
    /// The span of this item in the source.
    pub span: Span,
}

impl Item {
    /// Create a new item with the given kind and span.
    pub fn new(kind: ItemKind, span: Span) -> Self {
        Item { kind, span }
    }
}

/// The kind of an item.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ItemKind {
    /// A module declaration
    Module(ModuleDecl),

    /// A use/import declaration
    Use(UseDecl),

    /// A function declaration
    Function(FunctionDecl),

    /// A struct declaration
    Struct(StructDecl),

    /// An enum declaration
    Enum(EnumDecl),

    /// A trait declaration
    Trait(TraitDecl),

    /// An implementation
    Impl(ImplDecl),

    /// A type alias
    TypeAlias(TypeAliasDecl),

    /// A const declaration
    Const(ConstDecl),

    /// A static declaration
    Static(StaticDecl),

    /// A component declaration
    Component(ComponentDecl),

    /// A script declaration
    Script(ScriptDecl),

    /// An actor declaration
    Actor(ActorDecl),

    /// An unsafe block declaration
    UnsafeBlock(UnsafeBlockDecl),

    /// An item that couldn't be parsed (error recovery)
    Error,
}

/// A module declaration.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ModuleDecl {
    /// Attributes on this module.
    pub attrs: Vec<Attr>,
    /// Whether this module is public.
    pub public: bool,
    /// The module name.
    pub name: Ident,
    /// The module items (None for external modules like `mod foo;`).
    pub items: Option<Vec<Item>>,
}

/// A use declaration.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UseDecl {
    /// The use tree.
    pub tree: UseTree,
}

/// A use tree.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UseTree {
    /// A simple path (e.g., `use foo::bar;`)
    Path(Path),

    /// A path with alias (e.g., `use foo::bar as baz;`)
    Alias(Path, Ident),

    /// A glob import (e.g., `use foo::*;`)
    Glob(Path),

    /// A nested tree (e.g., `use foo::{bar, baz};`)
    Nested(Path, Vec<UseTree>),
}

/// A function declaration.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FunctionDecl {
    /// Attributes on this function.
    pub attrs: Vec<Attr>,
    /// The function signature.
    pub sig: FunctionSig,
    /// The function body (None for trait method declarations).
    pub body: Option<Block>,
}

/// A function signature.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FunctionSig {
    /// Whether this function is public.
    pub public: bool,
    /// Whether this function is unsafe.
    pub unsafe_: bool,
    /// The function name.
    pub name: Ident,
    /// Generic parameters.
    pub generic_params: Vec<GenericParam>,
    /// Function parameters.
    pub params: Vec<Param>,
    /// Return type (None for unit return).
    pub return_ty: Option<Ty>,
}

/// A function parameter.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Param {
    /// The parameter pattern.
    pub pat: Pat,
    /// The parameter type.
    pub ty: Ty,
}

/// A struct declaration.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StructDecl {
    /// Attributes on this struct.
    pub attrs: Vec<Attr>,
    /// Whether this struct is public.
    pub public: bool,
    /// The struct name.
    pub name: Ident,
    /// Generic parameters.
    pub generic_params: Vec<GenericParam>,
    /// The struct body.
    pub body: StructBody,
}

/// A struct body.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StructBody {
    /// A struct with named fields (e.g., `struct Point { x: i32, y: i32 }`)
    Named(Vec<StructField>),

    /// A tuple struct (e.g., `struct Point(i32, i32);`)
    Tuple(Vec<TupleField>),

    /// A unit struct (e.g., `struct Unit;`)
    Unit,
}

/// A named struct field.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StructField {
    /// Attributes on this field.
    pub attrs: Vec<Attr>,
    /// Whether this field is public.
    pub public: bool,
    /// The field name.
    pub name: Ident,
    /// The field type.
    pub ty: Ty,
}

/// A tuple struct field.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TupleField {
    /// Attributes on this field.
    pub attrs: Vec<Attr>,
    /// Whether this field is public.
    pub public: bool,
    /// The field type.
    pub ty: Ty,
}

/// An enum declaration.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EnumDecl {
    /// Attributes on this enum.
    pub attrs: Vec<Attr>,
    /// Whether this enum is public.
    pub public: bool,
    /// The enum name.
    pub name: Ident,
    /// Generic parameters.
    pub generic_params: Vec<GenericParam>,
    /// The enum variants.
    pub variants: Vec<EnumVariant>,
}

/// An enum variant.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EnumVariant {
    /// Attributes on this variant.
    pub attrs: Vec<Attr>,
    /// The variant name.
    pub name: Ident,
    /// The variant body.
    pub body: EnumVariantBody,
}

/// An enum variant body.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EnumVariantBody {
    /// A unit variant (e.g., `None`)
    Unit,

    /// A tuple variant (e.g., `Some(T)`)
    Tuple(Vec<TupleField>),

    /// A struct variant (e.g., `Point { x: i32, y: i32 }`)
    Struct(Vec<StructField>),
}

/// A trait declaration.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TraitDecl {
    /// Attributes on this trait.
    pub attrs: Vec<Attr>,
    /// Whether this trait is public.
    pub public: bool,
    /// The trait name.
    pub name: Ident,
    /// Generic parameters.
    pub generic_params: Vec<GenericParam>,
    /// Supertraits (e.g., `trait Foo: Bar + Baz`).
    pub supertraits: Vec<TraitBound>,
    /// Trait items (associated types and methods).
    pub items: Vec<TraitItem>,
}

/// A trait item.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TraitItem {
    /// An associated type declaration.
    AssociatedType(AssociatedType),

    /// A trait method declaration.
    Function(FunctionDecl),
}

/// An associated type declaration.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssociatedType {
    /// The type name.
    pub name: Ident,
    /// Optional trait bounds.
    pub bounds: Vec<TraitBound>,
}

/// An implementation.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ImplDecl {
    /// Attributes on this impl.
    pub attrs: Vec<Attr>,
    /// Generic parameters.
    pub generic_params: Vec<GenericParam>,
    /// The trait being implemented (None for inherent impls).
    pub trait_: Option<Path>,
    /// The type being implemented for.
    pub self_ty: Ty,
    /// Implementation items.
    pub items: Vec<ImplItem>,
}

/// An implementation item.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ImplItem {
    /// An associated type implementation.
    AssociatedType(AssociatedTypeImpl),

    /// A method implementation.
    Function(FunctionDecl),
}

/// An associated type implementation.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssociatedTypeImpl {
    /// The type name.
    pub name: Ident,
    /// The concrete type.
    pub ty: Ty,
}

/// A type alias declaration.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TypeAliasDecl {
    /// Attributes on this type alias.
    pub attrs: Vec<Attr>,
    /// Whether this type alias is public.
    pub public: bool,
    /// The alias name.
    pub name: Ident,
    /// Generic parameters.
    pub generic_params: Vec<GenericParam>,
    /// The aliased type.
    pub ty: Ty,
}

/// A const declaration.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConstDecl {
    /// Attributes on this const.
    pub attrs: Vec<Attr>,
    /// Whether this const is public.
    pub public: bool,
    /// The const name.
    pub name: Ident,
    /// The const type.
    pub ty: Ty,
    /// The const value.
    pub value: Expr,
}

/// A static declaration.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StaticDecl {
    /// Attributes on this static.
    pub attrs: Vec<Attr>,
    /// Whether this static is public.
    pub public: bool,
    /// Whether this static is mutable.
    pub mutable: bool,
    /// The static name.
    pub name: Ident,
    /// The static type.
    pub ty: Ty,
    /// The static value.
    pub value: Expr,
}

/// A component declaration (for reactive UI).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ComponentDecl {
    /// Attributes on this component.
    pub attrs: Vec<Attr>,
    /// The component name.
    pub name: Ident,
    /// Generic parameters.
    pub generic_params: Vec<GenericParam>,
    /// Component items.
    pub items: Vec<ComponentItem>,
}

/// A component item.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComponentItem {
    /// Props declaration.
    Props(Vec<PropDecl>),

    /// State declaration.
    State(Vec<StateDecl>),

    /// Computed values.
    Computed(Vec<ComputedDecl>),

    /// View (template) declaration.
    View(ViewContent),

    /// Style declaration.
    Style(String),

    /// Lifecycle hooks.
    Lifecycle(LifecycleHook),

    /// Helper function.
    Function(FunctionDecl),
}

/// A prop declaration.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PropDecl {
    /// The prop name.
    pub name: Ident,
    /// The prop type.
    pub ty: Ty,
    /// Optional default value.
    pub default: Option<Expr>,
}

/// A state declaration.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StateDecl {
    /// The state name.
    pub name: Ident,
    /// The state type.
    pub ty: Ty,
    /// Optional initializer.
    pub init: Option<Expr>,
}

/// A computed value declaration.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ComputedDecl {
    /// The computed value name.
    pub name: Ident,
    /// The computed value type.
    pub ty: Ty,
    /// The computation expression.
    pub expr: Block,
}

/// View content (simplified for now).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ViewContent {
    /// The raw view content (we'll parse this properly in a later phase).
    pub content: String,
    /// The span of the view content.
    pub span: Span,
}

/// A lifecycle hook.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LifecycleHook {
    /// On mount hook.
    OnMount(Block),

    /// On update hook.
    OnUpdate(Block),

    /// On unmount hook.
    OnUnmount(Block),
}

/// A script declaration (for job orchestration).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ScriptDecl {
    /// Attributes on this script.
    pub attrs: Vec<Attr>,
    /// The script name.
    pub name: Ident,
    /// Script items.
    pub items: Vec<ScriptItem>,
}

/// A script item.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ScriptItem {
    /// Configuration.
    Config(Vec<ConfigOption>),

    /// Schedule configuration.
    Schedule(Vec<ScheduleOption>),

    /// Script-level state.
    State(Vec<StateDecl>),

    /// A job declaration.
    Job(JobDecl),

    /// Helper function.
    Function(FunctionDecl),
}

/// A configuration option.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConfigOption {
    /// The option name.
    pub name: Ident,
    /// The option value.
    pub value: Lit,
}

/// A schedule option.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ScheduleOption {
    /// The option name.
    pub name: Ident,
    /// The option value.
    pub value: Lit,
}

/// A job declaration.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct JobDecl {
    /// The job name.
    pub name: Ident,
    /// Jobs this depends on.
    pub depends_on: Vec<Ident>,
    /// Job steps.
    pub steps: Vec<StepDecl>,
}

/// A step declaration.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StepDecl {
    /// The step name.
    pub name: Ident,
    /// Step configuration options.
    pub config: Vec<StepOption>,
    /// The step body.
    pub body: Block,
    /// Whether this step is in a parallel block.
    pub parallel: bool,
}

/// A step configuration option.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StepOption {
    /// The option name.
    pub name: Ident,
    /// The option value (literal or identifier).
    pub value: StepOptionValue,
}

/// A step option value.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StepOptionValue {
    /// A literal value.
    Literal(Lit),

    /// An identifier reference.
    Ident(Ident),
}

/// An actor declaration.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ActorDecl {
    /// Attributes on this actor.
    pub attrs: Vec<Attr>,
    /// The actor name.
    pub name: Ident,
    /// Generic parameters.
    pub generic_params: Vec<GenericParam>,
    /// Actor items.
    pub items: Vec<ActorItem>,
}

/// An actor item.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActorItem {
    /// Actor state.
    State(Vec<StateDecl>),

    /// A receiver (message handler).
    Receiver(ReceiverDecl),

    /// Helper function.
    Function(FunctionDecl),
}

/// A receiver declaration.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ReceiverDecl {
    /// The receiver name.
    pub name: Ident,
    /// Receiver parameters.
    pub params: Vec<Param>,
    /// Return type (None for unit return).
    pub return_ty: Option<Ty>,
    /// The receiver body.
    pub body: Block,
}

/// An unsafe block declaration.
///
/// Unsafe blocks are low-level performance primitives with strict constraints:
/// - Only primitive numeric types and pointers allowed as parameters and locals
/// - No access to DOM, signals, jobs, or high-level runtime APIs
/// - No panics allowed; errors must be represented as return codes or result structs
/// - Compiled directly to efficient WASM without high-level runtime overhead
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UnsafeBlockDecl {
    /// Attributes on this unsafe block.
    pub attrs: Vec<Attr>,
    /// The block name.
    pub name: Ident,
    /// Block parameters (must be primitive or pointer types).
    pub params: Vec<Param>,
    /// Return type (must be primitive, pointer, or unit).
    pub return_ty: Option<Ty>,
    /// The block body.
    pub body: Block,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_decl() {
        let sig = FunctionSig {
            public: true,
            unsafe_: false,
            name: Ident::dummy("test".to_string()),
            generic_params: vec![],
            params: vec![],
            return_ty: None,
        };
        let func = FunctionDecl {
            attrs: vec![],
            sig,
            body: None,
        };

        assert_eq!(func.sig.name.name, "test");
        assert!(func.sig.public);
        assert!(!func.sig.unsafe_);
        assert!(func.body.is_none());
    }

    #[test]
    fn test_struct_decl() {
        let struct_decl = StructDecl {
            attrs: vec![],
            public: true,
            name: Ident::dummy("Point".to_string()),
            generic_params: vec![],
            body: StructBody::Unit,
        };

        assert_eq!(struct_decl.name.name, "Point");
        assert!(struct_decl.public);
        assert!(matches!(struct_decl.body, StructBody::Unit));
    }

    #[test]
    fn test_enum_decl() {
        let variant = EnumVariant {
            attrs: vec![],
            name: Ident::dummy("None".to_string()),
            body: EnumVariantBody::Unit,
        };

        let enum_decl = EnumDecl {
            attrs: vec![],
            public: true,
            name: Ident::dummy("Option".to_string()),
            generic_params: vec![],
            variants: vec![variant],
        };

        assert_eq!(enum_decl.name.name, "Option");
        assert_eq!(enum_decl.variants.len(), 1);
        assert_eq!(enum_decl.variants[0].name.name, "None");
    }

    #[test]
    fn test_component_decl() {
        let component = ComponentDecl {
            attrs: vec![],
            name: Ident::dummy("Counter".to_string()),
            generic_params: vec![],
            items: vec![],
        };

        assert_eq!(component.name.name, "Counter");
        assert_eq!(component.items.len(), 0);
    }

    #[test]
    fn test_use_decl() {
        let path = Path::from_ident(Ident::dummy("foo".to_string()));
        let use_tree = UseTree::Path(path);
        let use_decl = UseDecl { tree: use_tree };

        match use_decl.tree {
            UseTree::Path(_) => {}
            _ => panic!("Expected path use tree"),
        }
    }
}
