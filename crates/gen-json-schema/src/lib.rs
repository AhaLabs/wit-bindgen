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

pub mod schema;

pub struct JSONSchema {
    deps: Deps,
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
        let mut r = JSONSchema::new();
        r
    }
}

fn unwrap_option<'a>(iface: &'a Interface, ty: &'a Type) -> Option<&'a Type> {
    match ty {
        Type::Id(id) => {
            let ty = &iface.types[*id];
            match &ty.kind {
                TypeDefKind::Option(t) => Some(&t),
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
        }
    }

    // fn

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
            Type::Handle(id) => {
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
                    TypeDefKind::Stream(s) => {
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
        builder.desc(docs)
    }

    //     pub fn build_type_header(name: &str) {
    //         if self.types == 0 {
    //             self.src.push_str("# Types\n\n");
    //         }
    //         self.types += 1;
    //         self.src.push_str(&format!(
    //             "## <a href=\"#{}\" name=\"{0}\"></a> `{}`: ",
    //             name.to_snake_case(),
    //             name,
    //         ));
    //         self.hrefs
    //             .insert(name.to_string(), format!("#{}", name.to_snake_case()));
    //     }

    //     pub fn build_type_info(ty: TypeId, docs: &Docs) {
    //         Self::docs(docs);
    //         self.src.push_str("\n");
    //         self.src
    //             .push_str(&format!("Size: {}, ", self.sizes.size(&Type::Id(ty))));
    //         self.src
    //             .push_str(&format!("Alignment: {}\n", self.sizes.align(&Type::Id(ty))));
    //     }
}

// struct JsonBuilder<'a> {

// }

impl Generator for JSONSchema {
    fn preprocess_one(&mut self, iface: &Interface, _dir: Direction) {
        // self.sizes.fill(iface);
        schema::add_primitives(&mut self.deps.0);
    }

    fn type_record(
        &mut self,
        iface: &Interface,
        _id: TypeId,
        name: &str,
        record: &Record,
        docs: &Docs,
    ) {
        let mut d = mem::take(&mut self.deps);
        d.0.schema(name, |builder| {
            // builder.id(name);
            builder.object();
            builder.additional_properties(false);
            Self::docs(docs, builder);
            let mut req = vec![];
            builder.properties(|hash| {
                for Field { docs, name, ty } in record.fields.iter() {
                    hash.insert(name, |builder| {
                        let ty = unwrap_option(iface, ty).unwrap_or_else(|| {
                            req.push(name.to_string());
                            ty
                        });
                        Self::build_ty(iface, ty, builder);
                        Self::docs(docs, builder)
                    })
                }
            });
            if req.len() > 0 {
                builder.required(req);
            }
        });
        self.deps = d;
    }

    fn type_tuple(
        &mut self,
        iface: &Interface,
        _id: TypeId,
        name: &str,
        tuple: &Tuple,
        docs: &Docs,
    ) {
        self.deps.schema(name, |builder| {
            Self::build_tuple(iface, tuple, builder);
            Self::docs(docs, builder);
        });
    }

    fn type_flags(
        &mut self,
        _iface: &Interface,
        id: TypeId,
        name: &str,
        flags: &Flags,
        docs: &Docs,
    ) {
    }

    fn type_variant(
        &mut self,
        iface: &Interface,
        id: TypeId,
        name: &str,
        variant: &Variant,
        docs: &Docs,
    ) {
    }

    fn type_union(
        &mut self,
        iface: &Interface,
        id: TypeId,
        name: &str,
        union: &Union,
        docs: &Docs,
    ) {
    }

    fn type_enum(&mut self, _iface: &Interface, id: TypeId, name: &str, enum_: &Enum, docs: &Docs) {
    }

    fn type_option(
        &mut self,
        iface: &Interface,
        _id: TypeId,
        name: &str,
        payload: &Type,
        docs: &Docs,
    ) {
        self.deps.schema(name, |builder| {
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
        self.deps.schema(name, |builder| {
            Self::build_expected(iface, expected, builder);
            Self::docs(docs, builder);
        });
    }

    fn type_resource(&mut self, iface: &Interface, ty: ResourceId) {
        drop((iface, ty));
    }

    fn type_alias(&mut self, iface: &Interface, _id: TypeId, name: &str, ty: &Type, docs: &Docs) {
        self.deps.schema(name, |builder| {
            Self::build_ty(iface, ty, builder);
            Self::docs(docs, builder)
        });
    }

    fn type_list(&mut self, iface: &Interface, id: TypeId, name: &str, _ty: &Type, docs: &Docs) {
        self.type_alias(iface, id, name, &Type::Id(id), docs);
    }

    fn type_builtin(&mut self, iface: &Interface, id: TypeId, name: &str, ty: &Type, docs: &Docs) {
        self.type_alias(iface, id, name, ty, docs)
    }

    fn import(&mut self, iface: &Interface, func: &Function) {
        // let mut d = mem::take(&mut self.deps);
        // let Function {
        //     name,
        //     is_async,
        //     docs,
        //     kind,
        //     params,
        //     result,
        // } = func;
        // d.0.schema(name, |builder| {
        //     Self::build_ty(iface, ty, builder);
        //     Self::docs(docs, builder)
        // });
        // self.deps = d;
    }

    fn export(&mut self, iface: &Interface, func: &Function) {
        self.import(iface, func);
    }

    fn finish_one(&mut self, _iface: &Interface, files: &mut Files) {
        let mut builder = valico::json_schema::Builder::new();
        // builder.dependencies(build)
        builder.schema("http://json-schema.org/draft-07/schema#");
        builder.dependencies(|deps| {
            let d = mem::take(&mut self.deps).0;
            let _ = mem::replace(deps, d);
        });

        println!("{:#?}", builder.into_json());

        // let parser = Parser::new(&self.src);
        // let mut events = Vec::new();
        // for event in parser {
        //     if let Event::Code(code) = &event {
        //         if let Some(dst) = self.hrefs.get(code.as_ref()) {
        //             let tag = Tag::Link(LinkType::Inline, dst.as_str().into(), "".into());
        //             events.push(Event::Start(tag.clone()));
        //             events.push(event.clone());
        //             events.push(Event::End(tag));
        //             continue;
        //         }
        //     }
        //     events.push(event);
        // }
        // let mut html_output = String::new();
        // html::push_html(&mut html_output, events.into_iter());

        // files.push("bindings.md", self.src.as_bytes());
        // files.push("bindings.html", html_output.as_bytes());
    }
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
        let mut schema = JSONSchema::new();
        println!("{:#?}", i);
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
type bytes = list<u8>
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
    fn builder() {
        let mut builder = valico::json_schema::Builder::new();
        // builder.dependencies(build)
        // builder.schema("https://json");
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
