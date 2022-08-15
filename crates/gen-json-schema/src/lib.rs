use std::{
    mem,
    ops::{Deref, DerefMut},
};

use heck::*;
pub use valico::json_schema::{
    builder::{Builder, Dependencies},
    PrimitiveType,
};
use wit_bindgen_gen_core::{wit_parser, Direction, Files, Generator};
use wit_parser::*;

pub mod gen;
pub mod schema;

#[derive(Default)]
pub struct JSONSchema {
    deps: Deps,
    add_primitives: bool,
}

pub struct Deps(Dependencies);

impl Default for Deps {
    fn default() -> Self {
        Self(Dependencies::new())
    }
}

impl Deref for Deps {
    type Target = Dependencies;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Deps {
    fn deref_mut(&mut self) -> &mut Dependencies {
        &mut self.0
    }
}
trait AddDep {
    fn add_dep(&mut self, s: &str);
}

impl AddDep for Builder {
    fn add_dep(&mut self, s: &str) {
        self.ref_(&format!("#/dependencies/{s}"))
    }
}

#[derive(Default, Debug, Clone)]
#[cfg_attr(feature = "structopt", derive(structopt::StructOpt))]
pub struct Opts {
    // ...
}

impl Opts {
    pub fn build(&self) -> JSONSchema {
        JSONSchema::new()
    }
}

fn unwrap_option<'a>(iface: &'a Interface, ty: &'a Type) -> Option<&'a Type> {
    match ty {
        Type::Id(id) => {
            let ty = &iface.types[*id];
            match &ty.kind {
                TypeDefKind::Option(t) => Some(t),
                _ => None,
            }
        }
        _ => None,
    }
}

impl JSONSchema {
    pub fn new() -> Self {
        Self {
            deps: Deps::default(),
            add_primitives: true,
        }
    }

    /// Convert the special doc annotations to properties.
    /// Returns the docs without the annotations.
    pub fn build_special_properties(props: &[&str], builder: &mut Builder) -> String {
        props
            .iter()
            .filter_map(|line| comment_to_attr(line, builder))
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn build_ty(iface: &Interface, ty: &Type, builder: &mut Builder) {
        match ty {
            Type::Unit => builder.add_dep("unit"),
            Type::Bool => builder.boolean(),
            Type::U8 => builder.add_dep("u8"),
            Type::S8 => builder.add_dep("i8"),
            Type::U16 => builder.add_dep("u16"),
            Type::S16 => builder.add_dep("i16"),
            Type::U32 => builder.add_dep("u32"),
            Type::S32 => builder.add_dep("i32"),
            Type::U64 => builder.add_dep("u64"),
            Type::S64 => builder.add_dep("i64"),
            Type::Float32 => builder.add_dep("f32"),
            Type::Float64 => builder.add_dep("f64"),
            Type::Char => builder.add_dep("char"),
            Type::String => builder.string(),
            Type::Handle(_id) => {
                // self.src.push_str("handle<");
                // self.src.push_str(&iface.resources[*id].name);
                // self.src.push_str(">");
            }
            Type::Id(id) => {
                let ty = &iface.types[*id];
                if let Some(name) = &ty.name {
                    builder.add_dep(&name.to_camel_case());
                    return;
                }
                match &ty.kind {
                    TypeDefKind::Type(t) => Self::build_ty(iface, t, builder),
                    TypeDefKind::Tuple(t) => Self::build_tuple(iface, t, builder),
                    TypeDefKind::Record(_)
                    | TypeDefKind::Flags(_)
                    | TypeDefKind::Enum(_)
                    | TypeDefKind::Variant(_)
                    | TypeDefKind::Union(_) => {
                        unreachable!()
                    }
                    TypeDefKind::Option(t) => Self::build_option(iface, t, builder),
                    TypeDefKind::Expected(e) => Self::build_expected(iface, e, builder),
                    TypeDefKind::List(t) => {
                        builder.array();
                        builder.items_array(|s| s.push(|s| Self::build_ty(iface, t, s)));
                    }
                    TypeDefKind::Stream(_s) => {
                        // self.src.push_str("stream<");
                        // Self::build_ty(iface, &s.element, false);
                        // self.src.push_str(", ");
                        // Self::build_ty(iface, &s.end, false);
                        // self.src.push_str(">");
                    }
                }
            }
        }
    }

    pub fn build_tuple(iface: &Interface, tuple: &Tuple, builder: &mut Builder) {
        builder.array();
        let len = tuple.types.len() as u64;
        builder.max_length(len);
        builder.min_length(len);
        builder.items_array(|s| {
            for ty in tuple.types.iter() {
                s.push(|b| Self::build_ty(iface, ty, b));
            }
        });
    }

    pub fn build_option(iface: &Interface, ty: &Type, builder: &mut Builder) {
        Self::build_ty(iface, ty, builder);
        builder.null()
    }

    pub fn build_expected(iface: &Interface, e: &Expected, builder: &mut Builder) {
        builder.one_of(|s| {
            s.push(|s| {
                Self::build_ty(iface, &e.ok, s);
                Self::build_ty(iface, &e.err, s);
            })
        })
    }

    pub fn docs(docs: &Docs, builder: &mut Builder) {
        let docs = match &docs.contents {
            Some(docs) => docs,
            None => return,
        };
        let docs = &Self::build_special_properties(&docs.split('\n').collect::<Vec<_>>(), builder);
        if !docs.is_empty() {
            builder.desc(docs)
        }
    }

    /// Adds `@immutable` to function if no mutable or no comment
    pub fn func_docs(docs: &Docs) -> Docs {
        let contents = Some(docs.contents.as_ref().map_or_else(
            || "@immutable".to_string(),
            |docs| {
                let mut docs = docs.clone();
                if !docs.contains("@mutable") {
                    docs.push_str("\n@immutable");
                }
                docs
            },
        ));
        Docs { contents }
    }


    pub fn build_enum(enm_: &Enum, builder: &mut Builder) {
        // TODO: Allow comments
        builder.enum_(|b| enm_.cases.iter().for_each(|case| b.push(case.name.clone())));
    }

    pub fn build_union(iface: &Interface, union: &Union, builder: &mut Builder) {
        // TODO: Allow comments
        builder.one_of(|schema_array| {
            union
                .cases
                .iter()
                .for_each(|case| schema_array.push(|cb| Self::build_ty(iface, &case.ty, cb)))
        });
    }

    pub fn build_flags(flags: &Flags, builder: &mut Builder) {
        builder.one_of(|arr| {
            flags.flags.iter().enumerate().for_each(|(i, flag)| {
                arr.push(|b| {
                    b.integer();
                    b.id(&flag.name);
                    let num = 1u128 << i;
                    b.maximum(num as f64);
                    b.minimum(num as f64);
                    Self::docs(&flag.docs, b);
                })
            })
        })
    }

    pub fn build_variant(iface: &Interface, variant: &Variant, builder: &mut Builder) {
        // TODO: Allow comments
        builder.object();
        builder.one_of(|schema_array| {
            variant.cases.iter().for_each(|case| {
                schema_array.push(|cb| {
                    cb.object();
                    cb.title(&case.name);
                    Self::build_ty(iface, &case.ty, cb);
                    Self::docs(&case.docs, cb);
                })
            })
        });
    }

    pub fn build_list(iface: &Interface, ty: &Type, builder: &mut Builder) {
        builder.array();
        builder.items_schema(|b| {
            Self::build_ty(iface, ty, b);
        })
    }

    pub fn build_func(iface: &Interface, func: &Function, builder: &mut Builder) {
        let Function {
            // name,
            // is_async,
            docs,
            // kind,
            params,
            result,
            ..
        } = func;
        let docs = Self::func_docs(docs);
        builder.object();        
        builder.properties(|schema_hash| {
            schema_hash.insert("arguments", |builder| {
                Self::build_record(
                    iface,
                    params
                        .iter()
                        .map(|(name, ty)| Field {
                            docs: Docs::default(),
                            name: name.to_string(),
                            ty: ty.clone(),
                        })
                        .collect::<Vec<_>>()
                        .as_slice(),
                    builder,
                );
                Self::docs(&docs, builder);
            });
            schema_hash.insert("result", |builder| Self::build_ty(iface, result, builder));
        });
    }

    pub fn build_record(iface: &Interface, fields: &[Field], builder: &mut Builder) {
        builder.object();
        builder.additional_properties(false);
        let mut req = vec![];
        builder.properties(|hash| {
            for Field { docs, name, ty } in fields.iter() {
                let name = &name.to_snek_case();
                hash.insert(&name, |builder| {
                    let ty = unwrap_option(iface, ty).unwrap_or_else(|| {
                        req.push(name.to_string());
                        ty
                    });
                    Self::build_ty(iface, ty, builder);
                    Self::docs(docs, builder)
                })
            }
        });
        if !req.is_empty() {
            builder.required(req);
        }
    }

    pub(crate) fn schema<F>(&mut self, name: &str, builder: F)
    where
        F: FnOnce(&mut Builder),
    {
        self.deps.schema(&name.to_camel_case(), builder)
    }
}

impl Generator for JSONSchema {
    fn preprocess_one(&mut self, _iface: &Interface, _dir: Direction) {
        if self.add_primitives {
            schema::add_primitives(&mut self.deps.0);
        }
    }

    fn type_record(
        &mut self,
        iface: &Interface,
        _id: TypeId,
        name: &str,
        record: &Record,
        docs: &Docs,
    ) {
        self.schema(name, |builder| {
            Self::build_record(iface, &record.fields, builder);
            Self::docs(docs, builder);
        });
    }

    fn type_tuple(
        &mut self,
        iface: &Interface,
        _id: TypeId,
        name: &str,
        tuple: &Tuple,
        docs: &Docs,
    ) {
        self.schema(name, |builder| {
            Self::build_tuple(iface, tuple, builder);
            Self::docs(docs, builder);
        });
    }

    fn type_flags(
        &mut self,
        _iface: &Interface,
        _id: TypeId,
        name: &str,
        flags: &Flags,
        docs: &Docs,
    ) {
        self.schema(name, |builder| {
            Self::build_flags(flags, builder);
            Self::docs(docs, builder);
        });
    }

    fn type_variant(
        &mut self,
        iface: &Interface,
        _id: TypeId,
        name: &str,
        variant: &Variant,
        docs: &Docs,
    ) {
        self.schema(name, |builder| {
            Self::build_variant(iface, variant, builder);
            Self::docs(docs, builder);
        })
    }

    fn type_union(
        &mut self,
        iface: &Interface,
        _id: TypeId,
        name: &str,
        union: &Union,
        docs: &Docs,
    ) {
        self.schema(name, |builder| {
            Self::build_union(iface, union, builder);
            Self::docs(docs, builder);
        });
    }

    fn type_enum(
        &mut self,
        _iface: &Interface,
        _id: TypeId,
        name: &str,
        enum_: &Enum,
        docs: &Docs,
    ) {
        self.schema(name, |builder| {
            Self::build_enum(enum_, builder);
            Self::docs(docs, builder);
        });
    }

    fn type_option(
        &mut self,
        iface: &Interface,
        _id: TypeId,
        name: &str,
        payload: &Type,
        docs: &Docs,
    ) {
        self.schema(name, |builder| {
            Self::build_option(iface, payload, builder);
            Self::docs(docs, builder);
        });
    }

    fn type_expected(
        &mut self,
        iface: &Interface,
        _id: TypeId,
        name: &str,
        expected: &Expected,
        docs: &Docs,
    ) {
        self.schema(name, |builder| {
            Self::build_expected(iface, expected, builder);
            Self::docs(docs, builder);
        });
    }

    //TODO: resource
    fn type_resource(&mut self, iface: &Interface, ty: ResourceId) {
        drop((iface, ty));
    }

    fn type_alias(&mut self, iface: &Interface, _id: TypeId, name: &str, ty: &Type, docs: &Docs) {
        self.schema(name, |builder| {
            Self::build_ty(iface, ty, builder);
            Self::docs(docs, builder);
        });
    }

    fn type_list(&mut self, iface: &Interface, _id: TypeId, name: &str, ty: &Type, docs: &Docs) {
        self.schema(name, |builder| {
            Self::build_list(iface, ty, builder);
            Self::docs(docs, builder);
        })
    }

    fn type_builtin(&mut self, iface: &Interface, id: TypeId, name: &str, ty: &Type, docs: &Docs) {
        self.type_alias(iface, id, name, ty, docs)
    }

    fn import(&mut self, iface: &Interface, func: &Function) {
        // let mut d = mem::take(&mut self.deps);
        let Function {
            name,
            // is_async,
            docs,
            // // kind,
            // params,
            // result,
            ..
        } = func;

        self.deps.schema(&name.to_snek_case(), |builder| {
            Self::build_func(iface, func, builder);
            Self::docs(&docs, builder);
        });

    }

    fn export(&mut self, iface: &Interface, func: &Function) {
        self.import(iface, func);
    }

    fn finish_one(&mut self, iface: &Interface, files: &mut Files) {
        let mut builder = valico::json_schema::Builder::new();

        builder.schema("http://json-schema.org/draft-07/schema#");
        builder.dependencies(|deps| {
            let d = mem::take(&mut self.deps).0;
            let _ = mem::replace(deps, d);
        });
        let json = builder.into_json();
        let output = json.to_string();
        let name = iface.name.to_kebab_case();
        files.push(&format!("{}.json", name), output.as_bytes());
    }
}

fn comment_to_attr(line: &str, builder: &mut Builder) -> Option<String> {
    if !line.trim_start().starts_with("@") {
        return Some(line.to_string());
    }
    let mut parts = line.split_whitespace().peekable();
    let (name, value) = match (parts.next().and_then(|s| s.strip_prefix('@')), parts.next()) {
        (Some(mutability @ ("mutable" | "immutable")), None) => ("funcType", mutability),
        (Some(int_arg @ ("minLength" | "maxLength")), Some(value)) => {
            builder.custom_vocabulary(int_arg, value.parse::<i64>().ok()?);
            return None;
        }
        (Some(name), value) => (name, value.unwrap_or_default()),
        _ => return Some(line.to_string()),
    };
    builder.custom_vocabulary(name, value);
    None
}

#[cfg(test)]
mod test {
    use super::JSONSchema;
    use crate::schema::add_primitives;
    use anyhow::Result;
    use valico::json_schema::PrimitiveType;
    use wit_bindgen_gen_core::{wit_parser::Interface, Direction, Files, Generator};

    fn parse_wit_str(s: &str) -> Result<Interface> {
        Interface::parse("a", s)
    }

    fn gen_interface(i: Interface) {
        let mut schema = JSONSchema::default();
        // println!("{:#?}", i);
        let imports = vec![i];
        schema.preprocess_all(&imports, &[]);
        let mut files = Files::default();
        schema.generate_one(&imports[0], Direction::Import, &mut files);
        schema.finish_all(&mut files);
    }

    fn get_str(s: &str) {
        gen_interface(parse_wit_str(s).expect(s));
    }

    #[test]
    fn simple() {
        get_str(
            r#"
/// This is a doc string
type u128 = string
      "#,
        );
    }

    #[test]
    fn record() {
        get_str(
            r#"
/// This is a doc string
record foo {
  req-field: string,
  optional: option<u32>,
}
    "#,
        );
    }

    #[test]
    fn list() {
        get_str(
            r#"
/// This is a doc string
type bytes = list<bool>
      "#,
        );
    }

    #[test]
    fn tuple() {
        get_str(
            r#"
/// This is a doc string
type t = tuple<u8, string, option<bool>>
      "#,
        );
    }

    #[test]
    fn option() {
        get_str(
            r#"
record foo {}

type t = option<foo>
    "#,
        );
    }

    #[test]
    fn enum_() {
        get_str(
            r#"
/// Letters
enum letters {
  a,
  b,
  c,
  d,
}
    "#,
        );
    }

    #[test]
    fn union() {
        get_str(
            r#"
/// different types
union union-example {
  bool,
  string,
  list<string>,
}

fon-do: func(a: union-example)
    "#,
        );
    }

    #[test]
    fn variant() {
        get_str(
            r#"
variant v1 {
  /// bool variant
  a(bool),
  /// String
  b(string),
  /// tuple
  c(tuple<bool, bool>),
}
    "#,
        );
    }

    #[test]
    fn func() {
        get_str(
            r#"
/// Function Doc
f3: func(a: u32, b: u32)
    "#,
        );
    }

    #[test]
    fn func_mut() {
        get_str(
            r#"
/// @mutable
f3: func(b: u32, a: u32) -> bool
    "#,
        );
    }

    #[test]
    fn func_name() {
        get_str(
            r#"
function-three: func(bees-knees: u32, a: u32) -> bool
    "#,
        );
    }

    #[test]
    fn builder() {
        let mut builder = valico::json_schema::Builder::new();
        builder.ref_("#/dependencies/Balance");
        builder.dependencies(|deps| {
            deps.schema("Balance", |builder| {
                builder.desc("balance is a type");
                builder.ref_("#/dependencies/U128");
            });
            deps.schema("U128", |b| {
                b.desc("String rep of u128");
                b.pattern("^[0-9]+");
                b.type_(PrimitiveType::String)
            });
            add_primitives(deps);
        });
        println!("{}", builder.into_json());
    }
}
