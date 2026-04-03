use std::collections::HashMap;
use std::path::Path;

use pdb::FallibleIterator;

use crate::error::AppError;
use crate::extractors::pdb::helpers::{calling_convention_str, resolve_argument_list, resolve_enum_values, resolve_field_list, resolve_type_name};
use crate::extractors::pdb::info::{EnumInfo, FunctionInfo, FunctionSignature, ResolvedFields, StructInfo};

pub mod info;
mod helpers;

/// Parsed PDB database
#[derive(Debug)]
pub struct PdbExtractor {
    structs: HashMap<String, StructInfo>,
    enums: HashMap<String, EnumInfo>,
    functions: HashMap<String, FunctionInfo>,
}

impl PdbExtractor {
    /// Open and parse a PDB file, extracting all structs, enums, and functions
    pub fn open(path: &Path) -> Result<Self, AppError> {
        let file = std::fs::File::open(path)
            .map_err(|e| AppError::Pdb(format!("cannot open {}: {e}", path.display())))?;
        let mut pdb = pdb::PDB::open(file)
            .map_err(|e| AppError::Pdb(format!("cannot parse PDB: {e}")))?;

        let (structs, enums, fn_sigs) = Self::parse_types(&mut pdb)?;

        // Re-open for symbols
        let file2 = std::fs::File::open(path)
            .map_err(|e| AppError::Pdb(format!("cannot reopen {}: {e}", path.display())))?;
        let mut pdb2 = pdb::PDB::open(file2)
            .map_err(|e| AppError::Pdb(format!("cannot parse PDB (2): {e}")))?;

        let functions = Self::parse_symbols(&mut pdb2, &fn_sigs)?;

        Ok(Self {
            structs,
            enums,
            functions,
        })
    }

    /// Look up a struct by exact name, returning JSON
    pub fn get_struct(&self, name: &str) -> Option<String> {
        self.structs
            .get(name)
            .map(|s| serde_json::to_string_pretty(s).unwrap())
    }

    /// Look up a function by exact name, returning JSON
    pub fn get_function(&self, name: &str) -> Option<String> {
        self.functions
            .get(name)
            .map(|f| serde_json::to_string_pretty(f).unwrap())
    }

    /// Look up an enum by exact name, returning JSON
    pub fn get_enum(&self, name: &str) -> Option<String> {
        self.enums
            .get(name)
            .map(|e| serde_json::to_string_pretty(e).unwrap())
    }

    /// Look up a struct by exact name, returning a C/C++ declaration
    pub fn get_struct_cpp(&self, name: &str) -> Option<String> {
        self.structs.get(name).map(format_struct_cpp)
    }

    /// Look up a function by exact name, returning a C/C++ declaration
    pub fn get_function_cpp(&self, name: &str) -> Option<String> {
        self.functions.get(name).map(format_function_cpp)
    }

    /// Look up an enum by exact name, returning a C/C++ declaration
    pub fn get_enum_cpp(&self, name: &str) -> Option<String> {
        self.enums.get(name).map(format_enum_cpp)
    }

    /// All parsed struct names
    pub fn struct_names(&self) -> Vec<&str> {
        self.structs.keys().map(String::as_str).collect()
    }

    /// All parsed function names
    pub fn function_names(&self) -> Vec<&str> {
        self.functions.keys().map(String::as_str).collect()
    }

    /// All parsed enum names
    pub fn enum_names(&self) -> Vec<&str> {
        self.enums.keys().map(String::as_str).collect()
    }

    // PRIVAT HELPERS

    /// Parse the type stream
    fn parse_types(
        pdb: &mut pdb::PDB<'_, std::fs::File>,
    ) -> Result<
        (
            HashMap<String, StructInfo>,
            HashMap<String, EnumInfo>,
            HashMap<pdb::TypeIndex, FunctionSignature>,
        ),
        AppError,
    > {
        let type_info = pdb
            .type_information()
            .map_err(|e| AppError::Pdb(format!("type information: {e}")))?;
        let mut finder = type_info.finder();

        struct PendingStruct {
            name: String,
            kind: String,
            size: u64,
            fields_idx: Option<pdb::TypeIndex>,
        }

        struct PendingEnum {
            name: String,
            underlying_type: pdb::TypeIndex,
            fields_idx: pdb::TypeIndex,
        }

        let mut pending_structs: Vec<PendingStruct> = Vec::new();
        let mut pending_enums: Vec<PendingEnum> = Vec::new();
        let mut fn_type_indices: Vec<pdb::TypeIndex> = Vec::new();

        let mut iter = type_info.iter();
        while let Some(item) = iter.next().map_err(|e| AppError::Pdb(format!("{e}")))? {
            finder.update(&iter);
            let idx = item.index();
            let Ok(data) = item.parse() else {
                continue;
            };
            match data {
                pdb::TypeData::Class(c) if !c.properties.forward_reference() => {
                    let kind = match c.kind {
                        pdb::ClassKind::Class => "class",
                        pdb::ClassKind::Struct => "struct",
                        pdb::ClassKind::Interface => "interface",
                    };
                    pending_structs.push(PendingStruct {
                        name: c.name.to_string().into_owned(),
                        kind: kind.to_string(),
                        size: c.size,
                        fields_idx: c.fields,
                    });
                }
                pdb::TypeData::Union(u) if !u.properties.forward_reference() => {
                    pending_structs.push(PendingStruct {
                        name: u.name.to_string().into_owned(),
                        kind: "union".to_string(),
                        size: u.size,
                        fields_idx: Some(u.fields),
                    });
                }
                pdb::TypeData::Enumeration(e) if !e.properties.forward_reference() => {
                    pending_enums.push(PendingEnum {
                        name: e.name.to_string().into_owned(),
                        underlying_type: e.underlying_type,
                        fields_idx: e.fields,
                    });
                }
                pdb::TypeData::Procedure(_) | pdb::TypeData::MemberFunction(_) => {
                    fn_type_indices.push(idx);
                }
                _ => {}
            }
        }
        drop(iter);

        // Resolve structs
        let mut struct_map = HashMap::with_capacity(pending_structs.len());
        for p in pending_structs {
            let resolved = match p.fields_idx {
                Some(idx) => resolve_field_list(&finder, idx),
                None => ResolvedFields::default(),
            };
            struct_map.insert(
                p.name.clone(),
                StructInfo {
                    name: p.name,
                    kind: p.kind,
                    size: p.size,
                    fields: resolved.fields,
                    base_classes: resolved.base_classes,
                    methods: resolved.methods,
                    static_fields: resolved.static_fields,
                    nested_types: resolved.nested_types,
                },
            );
        }

        // Resolve enums
        let mut enum_map = HashMap::with_capacity(pending_enums.len());
        for p in pending_enums {
            let underlying = resolve_type_name(&finder, p.underlying_type);
            let values = resolve_enum_values(&finder, p.fields_idx);
            enum_map.insert(
                p.name.clone(),
                EnumInfo {
                    name: p.name,
                    underlying_type: underlying,
                    values,
                },
            );
        }

        // Pre-compute function signatures
        let mut fn_sigs = HashMap::with_capacity(fn_type_indices.len());
        for idx in fn_type_indices {
            if let Ok(item) = finder.find(idx) {
                if let Ok(data) = item.parse() {
                    match data {
                        pdb::TypeData::Procedure(ref p) => {
                            let ret = p.return_type.map(|ti| resolve_type_name(&finder, ti));
                            let params = resolve_argument_list(&finder, p.argument_list);
                            let cc = calling_convention_str(p.attributes.calling_convention());
                            fn_sigs.insert(
                                idx,
                                FunctionSignature {
                                    return_type: ret,
                                    parameters: params,
                                    calling_convention: cc.to_string(),
                                },
                            );
                        }
                        pdb::TypeData::MemberFunction(ref mf) => {
                            let ret = Some(resolve_type_name(&finder, mf.return_type));
                            let params = resolve_argument_list(&finder, mf.argument_list);
                            let cc = calling_convention_str(mf.attributes.calling_convention());
                            fn_sigs.insert(
                                idx,
                                FunctionSignature {
                                    return_type: ret,
                                    parameters: params,
                                    calling_convention: cc.to_string(),
                                },
                            );
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok((struct_map, enum_map, fn_sigs))
    }

    fn parse_symbols(
        pdb: &mut pdb::PDB<'_, std::fs::File>,
        fn_sigs: &HashMap<pdb::TypeIndex, FunctionSignature>,
    ) -> Result<HashMap<String, FunctionInfo>, AppError> {
        let address_map = pdb.address_map().ok();
        let globals = pdb
            .global_symbols()
            .map_err(|e| AppError::Pdb(format!("global symbols: {e}")))?;

        let mut map = HashMap::new();
        let mut iter = globals.iter();
        while let Some(sym) = iter.next().map_err(|e| AppError::Pdb(format!("{e}")))? {
            let Ok(data) = sym.parse() else {
                continue;
            };
            let info = match data {
                pdb::SymbolData::Public(ref p) if p.function => {
                    let rva = address_map
                        .as_ref()
                        .and_then(|am| p.offset.to_rva(am))
                        .map(|r| r.0)
                        .unwrap_or(0);
                    FunctionInfo {
                        name: p.name.to_string().into_owned(),
                        rva,
                        len: None,
                        return_type: None,
                        parameters: Vec::new(),
                        calling_convention: None,
                        is_global: false,
                    }
                }
                pdb::SymbolData::Procedure(ref p) => {
                    let rva = address_map
                        .as_ref()
                        .and_then(|am| p.offset.to_rva(am))
                        .map(|r| r.0)
                        .unwrap_or(0);
                    let sig = fn_sigs.get(&p.type_index);
                    FunctionInfo {
                        name: p.name.to_string().into_owned(),
                        rva,
                        len: Some(p.len),
                        return_type: sig.and_then(|s| s.return_type.clone()),
                        parameters: sig.map(|s| s.parameters.clone()).unwrap_or_default(),
                        calling_convention: sig.map(|s| s.calling_convention.clone()),
                        is_global: p.global,
                    }
                }
                _ => continue,
            };
            // Prefer Procedure entries
            let dominated = info.return_type.is_none()
                && info.parameters.is_empty()
                && info.calling_convention.is_none();
            if dominated {
                map.entry(info.name.clone()).or_insert(info);
            } else {
                map.insert(info.name.clone(), info);
            }
        }

        Ok(map)
    }
}

// C/C++ formatting helpers

fn offset_width(max_val: u64) -> usize {
    if max_val <= 0xFF {
        4
    } else if max_val <= 0xFFFF {
        6
    } else if max_val <= 0xFFFFFF {
        8
    } else {
        10
    }
}

fn format_struct_cpp(s: &StructInfo) -> String {
    use std::fmt::Write;
    let mut out = String::new();

    let _ = writeln!(out, "// Size: {:#X}", s.size);

    for base in &s.base_classes {
        let virt = if base.is_virtual == Some(true) {
            "virtual "
        } else {
            ""
        };
        let _ = writeln!(out, "// Base: {}{} at {:#X}", virt, base.name, base.offset);
    }

    let _ = writeln!(out, "{} {} {{", s.kind, s.name);

    let max_offset = s
        .fields
        .iter()
        .map(|f| f.offset)
        .max()
        .unwrap_or(0)
        .max(s.size);
    let w = offset_width(max_offset);

    if s.kind == "union" {
        // Standalone union: flat field list, all at the same offset
        for field in &s.fields {
            let _ = writeln!(
                out,
                "    /* {:#0w$X} */ {} {};",
                field.offset, field.type_name, field.name,
                w = w
            );
        }
    } else {
        // Struct/class: group consecutive same-offset fields into anonymous unions
        let mut i = 0;
        while i < s.fields.len() {
            let offset = s.fields[i].offset;
            let group_start = i;
            while i < s.fields.len() && s.fields[i].offset == offset {
                i += 1;
            }
            let group = &s.fields[group_start..i];

            if group.len() == 1 {
                // Single field at this offset
                let field = &group[0];
                let _ = writeln!(
                    out,
                    "    /* {:#0w$X} */ {} {};",
                    field.offset, field.type_name, field.name,
                    w = w
                );
            } else {
                // Multiple fields at same offset → anonymous union
                let _ = writeln!(out, "    union");
                let _ = writeln!(out, "    {{");

                let mut has_bitfields = false;
                for field in group {
                    if field.type_name.contains(':') {
                        has_bitfields = true;
                    } else {
                        let _ = writeln!(
                            out,
                            "        /* {:#0w$X} */ {} {};",
                            field.offset, field.type_name, field.name,
                            w = w
                        );
                    }
                }

                if has_bitfields {
                    let _ = writeln!(out, "        struct");
                    let _ = writeln!(out, "        {{");
                    for field in group {
                        if field.type_name.contains(':') {
                            let _ = writeln!(
                                out,
                                "            /* {:#0w$X} */ {} {};",
                                field.offset, field.type_name, field.name,
                                w = w
                            );
                        }
                    }
                    let _ = writeln!(out, "        }};");
                }

                let _ = writeln!(out, "    }};");
            }
        }
    }

    if !s.static_fields.is_empty() {
        if !s.fields.is_empty() {
            out.push('\n');
        }
        for sf in &s.static_fields {
            let _ = writeln!(out, "    static {} {};", sf.type_name, sf.name);
        }
    }

    if !s.methods.is_empty() {
        if !s.fields.is_empty() || !s.static_fields.is_empty() {
            out.push('\n');
        }
        for m in &s.methods {
            let ret = m.return_type.as_deref().unwrap_or("void");
            let params = m.parameters.join(", ");
            if m.is_static {
                let _ = writeln!(out, "    static {} {}({});", ret, m.name, params);
            } else if m.is_virtual {
                let _ = writeln!(out, "    virtual {} {}({});", ret, m.name, params);
            } else {
                let _ = writeln!(out, "    {} {}({});", ret, m.name, params);
            }
        }
    }

    out.push_str("};\n");
    out
}

fn format_enum_cpp(e: &EnumInfo) -> String {
    use std::fmt::Write;
    let mut out = String::new();

    let _ = writeln!(out, "enum {} : {} {{", e.name, e.underlying_type);

    let last = e.values.len().saturating_sub(1);
    for (i, v) in e.values.iter().enumerate() {
        let comma = if i < last { "," } else { "" };
        let val_str = if v.value < 0 {
            format!("-{:#X}", v.value.unsigned_abs())
        } else {
            format!("{:#X}", v.value)
        };
        let _ = writeln!(out, "    {} = {}{}", v.name, val_str, comma);
    }

    out.push_str("};\n");
    out
}

fn format_function_cpp(f: &FunctionInfo) -> String {
    use std::fmt::Write;
    let mut out = String::new();

    let _ = writeln!(out, "// RVA: {:#X}", f.rva);
    if let Some(len) = f.len {
        let _ = writeln!(out, "// Length: {:#X}", len);
    }

    let ret = f.return_type.as_deref().unwrap_or("void");
    let cc = f
        .calling_convention
        .as_deref()
        .map(|c| format!("{} ", c))
        .unwrap_or_default();
    let params = if f.parameters.is_empty() {
        String::new()
    } else {
        f.parameters.join(", ")
    };

    let _ = writeln!(out, "{} {}{}({});", ret, cc, f.name, params);
    out
}