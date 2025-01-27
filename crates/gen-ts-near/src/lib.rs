use heck::*;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::mem;
use wit_bindgen_gen_core::wit_parser::abi::AbiVariant;
use wit_bindgen_gen_core::{wit_parser::*, Direction, Files, Generator};

mod gen;
pub use gen::generate_typescript;

#[derive(Default)]
pub struct Ts {
    src: Source,
    in_import: bool,
    opts: Opts,
    guest_imports: HashMap<String, Imports>,
    guest_exports: HashMap<String, Exports>,
    sizes: SizeAlign,
    #[allow(dead_code)]
    needs_get_export: bool,
    #[allow(dead_code)]
    imported_resources: BTreeSet<ResourceId>,
    #[allow(dead_code)]
    exported_resources: BTreeSet<ResourceId>,
    needs_ty_option: bool,
    needs_ty_result: bool,
    needs_ty_push_buffer: bool,
    needs_ty_pull_buffer: bool,
}

#[derive(Default)]
struct Imports {
    freestanding_funcs: Vec<(String, Source)>,
    resource_funcs: BTreeMap<ResourceId, Vec<(String, Source)>>,
}

#[derive(Default)]
struct Exports {
    freestanding_funcs: Vec<Source>,
    resource_funcs: BTreeMap<ResourceId, Vec<Source>>,
}

#[derive(Default, Debug, Clone)]
#[cfg_attr(feature = "structopt", derive(structopt::StructOpt))]
pub struct Opts {}

impl Opts {
    pub fn build(self) -> Ts {
        let mut r = Ts::new();
        r.opts = self;
        r
    }
}

impl Ts {
    pub fn new() -> Ts {
        Ts::default()
    }

    fn abi_variant(dir: Direction) -> AbiVariant {
        match dir {
            Direction::Import => AbiVariant::GuestExport,
            Direction::Export => AbiVariant::GuestImport,
        }
    }

    fn array_ty(&self, iface: &Interface, ty: &Type) -> Option<&'static str> {
        match ty {
            Type::U8 | Type::CChar => Some("Uint8Array"),
            Type::S8 => Some("Int8Array"),
            Type::U16 => Some("Uint16Array"),
            Type::S16 => Some("Int16Array"),
            Type::U32 | Type::Usize => Some("Uint32Array"),
            Type::S32 => Some("Int32Array"),
            Type::U64 => Some("BigUint64Array"),
            Type::S64 => Some("BigInt64Array"),
            Type::F32 => Some("Float32Array"),
            Type::F64 => Some("Float64Array"),
            Type::Char => None,
            Type::Handle(_) => None,
            Type::Id(id) => match &iface.types[*id].kind {
                TypeDefKind::Type(t) => self.array_ty(iface, t),
                _ => None,
            },
        }
    }

    fn print_ty(&mut self, iface: &Interface, ty: &Type) {
        match ty {
            Type::U8
            | Type::CChar
            | Type::S8
            | Type::U16
            | Type::S16
            | Type::U32
            | Type::Usize
            | Type::S32
            | Type::F32
            | Type::F64 => self.src.ts("number"),
            Type::U64 => self.src.ts("u64"),
            Type::S64 => self.src.ts("i64"),
            Type::Char => self.src.ts("string"),
            Type::Handle(id) => self.src.ts(&iface.resources[*id].name.to_camel_case()),
            Type::Id(id) => {
                let ty = &iface.types[*id];
                if let Some(name) = &ty.name {
                    return self.src.ts(&name.to_camel_case());
                }
                match &ty.kind {
                    TypeDefKind::Type(t) => self.print_ty(iface, t),
                    TypeDefKind::Record(r) if r.is_tuple() => self.print_tuple(iface, r),
                    TypeDefKind::Record(_) => panic!("anonymous record"),
                    TypeDefKind::Variant(v) if v.is_bool() => self.src.ts("boolean"),
                    TypeDefKind::Variant(v) => {
                        if iface.is_nullable_option(v) {
                            self.print_ty(iface, v.cases[1].ty.as_ref().unwrap());
                            self.src.ts(" | null");
                        } else if let Some(t) = v.as_option() {
                            self.needs_ty_option = true;
                            self.src.ts("Option<");
                            self.print_ty(iface, t);
                            self.src.ts(">");
                        } else if let Some((ok, err)) = v.as_expected() {
                            self.needs_ty_result = true;
                            self.src.ts("Result<");
                            match ok {
                                Some(ok) => self.print_ty(iface, ok),
                                None => self.src.ts("undefined"),
                            }
                            self.src.ts(", ");
                            match err {
                                Some(err) => self.print_ty(iface, err),
                                None => self.src.ts("undefined"),
                            }
                            self.src.ts(">");
                        } else {
                            panic!("anonymous variant");
                        }
                    }
                    TypeDefKind::List(v) => self.print_list(iface, v),
                    TypeDefKind::PushBuffer(v) => self.print_buffer(iface, true, v),
                    TypeDefKind::PullBuffer(v) => self.print_buffer(iface, false, v),
                    TypeDefKind::Pointer(_) | TypeDefKind::ConstPointer(_) => {
                        self.src.ts("number");
                    }
                }
            }
        }
    }


    fn print_list(&mut self, iface: &Interface, ty: &Type) {
        if let Some(r) = self.hash_map(iface, ty) {
            self.src.ts("Record<");
            self.print_ty(iface, &r.fields[0].ty);
            self.src.ts(", ");
            self.print_ty(iface, &r.fields[1].ty);
            self.src.ts(">");
        } else {
            match self.array_ty(iface, ty) {
                Some(ty) => self.src.ts(ty),
                None => {
                    if let Type::Char = ty {
                        self.src.ts("string");
                    } else {
                        self.print_ty(iface, ty);
                        self.src.ts("[]");
                    }
                }
            }
        }
    }


    fn print_tuple(&mut self, iface: &Interface, record: &Record) {
        self.src.ts("[");
        for (i, field) in record.fields.iter().enumerate() {
            if i > 0 {
                self.src.ts(", ");
            }
            self.print_ty(iface, &field.ty);
        }
        self.src.ts("]");
    }

    fn print_buffer(&mut self, iface: &Interface, push: bool, ty: &Type) {
        match self.array_ty(iface, ty) {
            Some(ty) => self.src.ts(ty),
            None => {
                if push {
                    self.needs_ty_push_buffer = true;
                    self.src.ts("PushBuffer");
                } else {
                    self.needs_ty_pull_buffer = true;
                    self.src.ts("PullBuffer");
                }
                self.src.ts("<");
                self.print_ty(iface, ty);
                self.src.ts(">");
            }
        }
    }

    fn docs(&mut self, docs: &Docs) {
        self.doc_str(&docs.contents);
    }

    fn doc_str(&mut self, contents: &Option<String>) {
        let docs = match &contents {
            Some(docs) => docs,
            None => return,
        };
        let lines = docs
            .lines()
            .filter(|line| *line != "change" && *line != "view")
            .collect::<Vec<&str>>();
        if lines.len() > 0 {
            self.src.ts("/**\n");
            for line in lines {
                self.src.ts(&format!("* {}\n", line));
            }
            self.src.ts("*/\n");
        }
    }

    fn print_args(&mut self, iface: &Interface, func: &Function, param_start: usize) {
        self.src.ts("(args: {");
        let none: Option<String> = None;

        let arg_fields: Vec<(&str, &Type, &Option<String>)> = func.params[param_start..]
            .iter()
            .map(|(name, ty)| (name.as_str(), ty, &none))
            .collect();
        self.print_fields(iface, arg_fields);
        self.src.ts("}");
        if func.params.len() == 0 {
            self.src.ts(" = {}");
        };
        self.src.ts(", options?: ");
        self.src.ts(if is_change(func) {
            "ChangeMethodOptions"
        } else {
            "ViewFunctionOptions"
        });
        self.src.ts("): ");
    }

    fn ts_func(&mut self, iface: &Interface, func: &Function) {
        self.docs(&func.docs);
        if is_change(func) {
            self.src.ts("async ");
        }
        let mut name_printed = false;
        if let FunctionKind::Static { .. } = &func.kind {
            // static methods in imports are still wired up to an imported host
            // object, but static methods on exports are actually static
            // methods on the resource object.
            if self.in_import {
                name_printed = true;
                self.src.ts(&func.name.to_snake_case());
            } else {
                self.src.ts("static ");
            }
        }
        if !name_printed {
            self.src.ts(&func.item_name().to_snake_case());
        }

        let param_start = match &func.kind {
            FunctionKind::Freestanding => 0,
            FunctionKind::Static { .. } if self.in_import => 0,
            FunctionKind::Static { .. } => {
                // the 0th argument for exported static methods will be the
                // instantiated interface
                self.src.ts(&iface.name.to_mixed_case());
                self.src.ts(": ");
                self.src.ts(&iface.name.to_camel_case());
                if func.params.len() > 0 {
                    self.src.ts(", ");
                }
                0
            }
            // skip the first parameter on methods which is `this`
            FunctionKind::Method { .. } => 1,
        };
        let name = func.item_name().to_snake_case();

        // Always async
        self.src.ts("Promise<");

        match func.results.len() {
            0 => self.src.ts("void"),
            1 => self.print_ty(iface, &func.results[0].1),
            _ => {
                if func.results.iter().any(|(n, _)| n.is_empty()) {
                    self.src.ts("[");
                    for (i, (_, ty)) in func.results.iter().enumerate() {
                        if i > 0 {
                            self.src.ts(", ");
                        }
                        self.print_ty(iface, ty);
                    }
                    self.src.ts("]");
                } else {
                    self.src.ts("{ ");
                    for (i, (name, ty)) in func.results.iter().enumerate() {
                        if i > 0 {
                            self.src.ts(", ");
                        }
                        self.src.ts(&name.to_mixed_case());
                        self.src.ts(": ");
                        self.print_ty(iface, ty);
                    }
                    self.src.ts(" }");
                }
            }
        }

        self.src.ts("> {\n");

        if is_change(func) {
            self.src.ts(&format!(
                "return providers.getTransactionLastResult(await this.{name}Raw(args, options));\n}}\n"
            ));
            self.docs(&func.docs);
            self.src.ts(&format!("{name}Raw"));
            self.print_args(iface, func, param_start);
            self.src.ts("Promise<providers.FinalExecutionOutcome> {{\n");
            self.src.ts(&format!("return this.account.functionCall({{contractId: this.contractId, methodName: \"{name}\", args, ...options}});\n}}\n"));
            self.docs(&func.docs);
            self.src.ts(&format!("{name}Tx"));
            self.print_args(iface, func, param_start);
            self.src.ts(&format!("transactions.Action {{\n return transactions.functionCall(\"{name}\", args, options?.gas ?? DEFAULT_FUNCTION_CALL_GAS, options?.attachedDeposit ?? new BN(0))\n}}\n"));
        } else {
            self.src.ts(&format!(
                "return this.account.viewFunction(this.contractId, \"{name}\", args, options);\n}}\n"
            ));
        }
    }

    fn print_fields(&mut self, iface: &Interface, fields: Vec<(&str, &Type, &Option<String>)>) {
        for (name, ty, docs) in fields.iter() {
            self.doc_str(docs);
            self.src.ts(&name.to_snake_case());
            if iface.is_ty_nullable_option(ty) {
                self.src.ts("?");
            }
            let ty= iface.get_nullable_option(ty).unwrap_or(ty);
            // self.src.ts(name.to_snake_case());
            self.src.ts(": ");
            self.print_ty(iface, &ty);
            self.src.ts(",\n");
        }
    }
}

impl Generator for Ts {
    fn preprocess_one(&mut self, iface: &Interface, dir: Direction) {
        let variant = Self::abi_variant(dir);
        self.sizes.fill(variant, iface);
        self.in_import = variant == AbiVariant::GuestImport;
        self.src
            .ts("import { Account, transactions, providers, DEFAULT_FUNCTION_CALL_GAS } from 'near-api-js';\n\n");
        self.src.ts("
        import BN from 'bn.js';
        export interface ChangeMethodOptions {
          gas?: BN;
          attachedDeposit?: BN;
          walletMeta?: string;
          walletCallbackUrl?: string;
      }
      export interface ViewFunctionOptions {
        parse?: (response: Uint8Array) => any;
        stringify?: (input: any) => any;
      }

      /** 64 bit unsigned integer less than 2^53 -1 */
      type u64 = number;
      /** 64 bit signed integer less than 2^53 -1 */
      type i64 = number;
        ")
    }

    fn type_record(
        &mut self,
        iface: &Interface,
        _id: TypeId,
        name: &str,
        record: &Record,
        docs: &Docs,
    ) {
        self.docs(docs);
        if record.is_tuple() {
            self.src
                .ts(&format!("export type {} = ", name.to_camel_case()));
            self.print_tuple(iface, record);
            self.src.ts(";\n");
        } else if record.is_flags() {
            let repr = iface
                .flags_repr(record)
                .expect("unsupported number of flags");
            let suffix = if repr == Int::U64 {
                self.src
                    .ts(&format!("export type {} = u64;\n", name.to_camel_case()));
                "n"
            } else {
                self.src
                    .ts(&format!("export type {} = number;\n", name.to_camel_case()));
                ""
            };
            let name = name.to_shouty_snake_case();
            for (i, field) in record.fields.iter().enumerate() {
                let field = field.name.to_shouty_snake_case();
                self.src.ts(&format!(
                    "export const {}_{} = {}{};\n",
                    name,
                    field,
                    1u64 << i,
                    suffix,
                ));
            }
        } else {
            self.src
                .ts(&format!("export interface {} {{\n", name.to_camel_case()));

            let fields = record
                .fields
                .iter()
                .map(|f| (f.name.as_str(), &f.ty, &f.docs.contents))
                .collect();
            self.print_fields(iface, fields);
            self.src.ts("}\n");
        }
    }

    fn type_variant(
        &mut self,
        iface: &Interface,
        _id: TypeId,
        name: &str,
        variant: &Variant,
        docs: &Docs,
    ) {
        self.docs(docs);
        if variant.is_bool() {
            self.src.ts(&format!(
                "export type {} = boolean;\n",
                name.to_camel_case(),
            ));
        } else if iface.is_nullable_option(variant) {
            self.src
                .ts(&format!("export type {} = ", name.to_camel_case()));
            self.print_ty(iface, variant.cases[1].ty.as_ref().unwrap());
            self.src.ts(" | null;\n");
        } else if variant.is_enum() {
            self.src
                .ts(&format!("export enum {} {{\n", name.to_camel_case()));
            for (i, case) in variant.cases.iter().enumerate() {
                self.docs(&case.docs);
                let name = case.name.to_camel_case();
                self.src.ts(&format!("{} = {},\n", name, i));
            }
            self.src.ts("}\n");
        } else {
            self.src
                .ts(&format!("export type {} = ", name.to_camel_case()));
            for (i, case) in variant.cases.iter().enumerate() {
                if i > 0 {
                    self.src.ts(" | ");
                }
                self.src
                    .ts(&format!("{}_{}", name, case.name).to_camel_case());
            }
            self.src.ts(";\n");
            for case in variant.cases.iter() {
                self.docs(&case.docs);
                self.src.ts(&format!(
                    "export interface {} {{\n",
                    format!("{}_{}", name, case.name).to_camel_case()
                ));
                self.src.ts("tag: \"");
                self.src.ts(&case.name);
                self.src.ts("\",\n");
                if let Some(ty) = &case.ty {
                    self.src.ts("val: ");
                    self.print_ty(iface, ty);
                    self.src.ts(",\n");
                }
                self.src.ts("}\n");
            }
        }
    }

    fn type_resource(&mut self, _iface: &Interface, ty: ResourceId) {
        if !self.in_import {
            self.exported_resources.insert(ty);
        }
    }

    fn type_alias(&mut self, iface: &Interface, _id: TypeId, name: &str, ty: &Type, docs: &Docs) {
        self.docs(docs);
        self.src
            .ts(&format!("export type {} = ", name.to_camel_case()));
        self.print_ty(iface, ty);
        self.src.ts(";\n");
    }

    fn type_list(&mut self, iface: &Interface, _id: TypeId, name: &str, ty: &Type, docs: &Docs) {
        self.docs(docs);
        self.src
            .ts(&format!("export type {} = ", name.to_camel_case()));
        self.print_list(iface, ty);
        self.src.ts(";\n");
    }

    fn type_pointer(
        &mut self,
        iface: &Interface,
        _id: TypeId,
        name: &str,
        const_: bool,
        ty: &Type,
        docs: &Docs,
    ) {
        drop((iface, _id, name, const_, ty, docs));
    }

    fn type_builtin(&mut self, iface: &Interface, _id: TypeId, name: &str, ty: &Type, docs: &Docs) {
        drop((iface, _id, name, ty, docs));
    }

    fn type_push_buffer(
        &mut self,
        iface: &Interface,
        _id: TypeId,
        name: &str,
        ty: &Type,
        docs: &Docs,
    ) {
        self.docs(docs);
        self.src
            .ts(&format!("export type {} = ", name.to_camel_case()));
        self.print_buffer(iface, true, ty);
        self.src.ts(";\n");
    }

    fn type_pull_buffer(
        &mut self,
        iface: &Interface,
        _id: TypeId,
        name: &str,
        ty: &Type,
        docs: &Docs,
    ) {
        self.docs(docs);
        self.src
            .ts(&format!("export type {} = ", name.to_camel_case()));
        self.print_buffer(iface, false, ty);
        self.src.ts(";\n");
    }

    // As with `abi_variant` above, we're generating host-side bindings here
    // so a user "export" uses the "guest import" ABI variant on the inside of
    // this `Generator` implementation.
    #[allow(dead_code, unused_variables)]
    fn export(&mut self, iface: &Interface, func: &Function) {}

    // As with `abi_variant` above, we're generating host-side bindings here
    // so a user "import" uses the "export" ABI variant on the inside of
    // this `Generator` implementation.
    fn import(&mut self, iface: &Interface, func: &Function) {
        let prev = mem::take(&mut self.src);
        self.ts_func(iface, func);

        let exports = self
            .guest_exports
            .entry(iface.name.to_string())
            .or_insert_with(Exports::default);

        let func_body = mem::replace(&mut self.src, prev);
        match &func.kind {
            FunctionKind::Freestanding => {
                exports.freestanding_funcs.push(func_body);
            }
            FunctionKind::Static { resource, .. } | FunctionKind::Method { resource, .. } => {
                exports
                    .resource_funcs
                    .entry(*resource)
                    .or_insert(Vec::new())
                    .push(func_body);
            }
        }
    }

    fn finish_one(&mut self, iface: &Interface, files: &mut Files) {
        for (module, funcs) in mem::take(&mut self.guest_imports) {
            self.src
                .ts(&format!("export interface {} {{\n", module.to_camel_case()));

            for (_, src) in funcs.freestanding_funcs.iter() {
                self.src.ts(&src.ts);
            }

            self.src.ts("}\n");

            for (resource, _) in iface.resources.iter() {
                self.src.ts(&format!(
                    "export interface {} {{\n",
                    iface.resources[resource].name.to_camel_case()
                ));
                if let Some(funcs) = funcs.resource_funcs.get(&resource) {
                    for (_, src) in funcs {
                        self.src.ts(&src.ts);
                    }
                }
                self.src.ts("}\n");
            }
        }
        let imports = mem::take(&mut self.src);

        for (_module, exports) in mem::take(&mut self.guest_exports) {
            self.src.ts("\nexport class Contract {
                  
                  constructor(public account: Account, public readonly contractId: string){}\n\n");
            for func in exports.freestanding_funcs.iter() {
                self.src.ts(&func.ts);
            }
            self.src.ts("}\n");
        }

        if mem::take(&mut self.needs_ty_option) {
            self.src
                .ts("export type Option<T> = { tag: \"none\" } | { tag: \"some\", val; T };\n");
        }
        if mem::take(&mut self.needs_ty_result) {
            self.src.ts(
                "export type Result<T, E> = { tag: \"ok\", val: T } | { tag: \"err\", val: E };\n",
            );
        }

        self.src.ts(&imports.ts);

        let src = mem::take(&mut self.src);
        let name = iface.name.to_kebab_case();
        files.push(&format!("{}.ts", name), src.ts.as_bytes());
    }

    fn finish_all(&mut self, _files: &mut Files) {
        assert!(self.src.ts.is_empty());
    }
}

impl Ts {
    fn hash_map<'a>(&self, iface: &'a Interface, ty: &Type) -> Option<&'a Record> {
        iface
            .get_record(ty)
            .filter(|r| r.is_tuple() && r.fields.len() == 2)
    }
}

pub fn to_js_ident(name: &str) -> &str {
    match name {
        "in" => "in_",
        "import" => "import_",
        s => s,
    }
}

#[derive(Default)]
struct Source {
    ts: wit_bindgen_gen_core::Source,
}

impl Source {
    fn ts(&mut self, s: &str) {
        self.ts.push_str(s);
    }
}

fn is_change(func: &Function) -> bool {
    if let Some(docs) = &func.docs.contents {
        let x = docs
            .split("\n")
            .filter(|s| *s == "change")
            .collect::<Vec<_>>();
        if x.len() == 1 {
            return true;
        }
    }
    false
}


