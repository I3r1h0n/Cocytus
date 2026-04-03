use crate::extractors::pdb::info::{
    BaseClassInfo, EnumValue, MethodInfo, NestedTypeInfo, ResolvedFields, StaticFieldInfo, StructField
};

/// Walk a FieldList chain and collect all entries.
pub fn resolve_field_list(finder: &pdb::TypeFinder, start: pdb::TypeIndex) -> ResolvedFields {
    let mut result = ResolvedFields::default();
    let mut idx = Some(start);

    while let Some(current) = idx {
        let Ok(item) = finder.find(current) else { break };
        let Ok(pdb::TypeData::FieldList(fl)) = item.parse() else {
            break;
        };
        for field in &fl.fields {
            match field {
                pdb::TypeData::Member(m) => {
                    result.fields.push(StructField {
                        name: m.name.to_string().into_owned(),
                        type_name: resolve_type_name(finder, m.field_type),
                        offset: m.offset,
                        access: access_str(m.attributes.access()).to_string(),
                    });
                }
                pdb::TypeData::BaseClass(bc) => {
                    let name = resolve_type_name(finder, bc.base_class);
                    result.base_classes.push(BaseClassInfo {
                        name,
                        offset: bc.offset,
                        is_virtual: None,
                    });
                }
                pdb::TypeData::VirtualBaseClass(vbc) => {
                    let name = resolve_type_name(finder, vbc.base_class);
                    result.base_classes.push(BaseClassInfo {
                        name,
                        offset: vbc.base_pointer_offset,
                        is_virtual: Some(true),
                    });
                }
                pdb::TypeData::Method(m) => {
                    let (ret, params) = resolve_method_signature(finder, m.method_type);
                    result.methods.push(MethodInfo {
                        name: m.name.to_string().into_owned(),
                        access: access_str(m.attributes.access()).to_string(),
                        return_type: ret,
                        parameters: params,
                        is_virtual: m.attributes.is_virtual()
                            || m.attributes.is_pure_virtual()
                            || m.attributes.is_intro_virtual(),
                        is_static: m.attributes.is_static(),
                    });
                }
                pdb::TypeData::OverloadedMethod(om) => {
                    let method_name = om.name.to_string().into_owned();
                    // Resolve the method list to get individual overloads
                    if let Ok(ml_item) = finder.find(om.method_list) {
                        if let Ok(pdb::TypeData::MethodList(ml)) = ml_item.parse() {
                            for entry in &ml.methods {
                                let (ret, params) =
                                    resolve_method_signature(finder, entry.method_type);
                                result.methods.push(MethodInfo {
                                    name: method_name.clone(),
                                    access: access_str(entry.attributes.access()).to_string(),
                                    return_type: ret,
                                    parameters: params,
                                    is_virtual: entry.attributes.is_virtual()
                                        || entry.attributes.is_pure_virtual()
                                        || entry.attributes.is_intro_virtual(),
                                    is_static: entry.attributes.is_static(),
                                });
                            }
                        }
                    }
                }
                pdb::TypeData::StaticMember(sm) => {
                    result.static_fields.push(StaticFieldInfo {
                        name: sm.name.to_string().into_owned(),
                        type_name: resolve_type_name(finder, sm.field_type),
                        access: access_str(sm.attributes.access()).to_string(),
                    });
                }
                pdb::TypeData::Nested(n) => {
                    let type_name = resolve_type_name(finder, n.nested_type);
                    result.nested_types.push(NestedTypeInfo {
                        name: n.name.to_string().into_owned(),
                        type_name,
                    });
                }
                _ => {}
            }
        }
        idx = fl.continuation;
    }

    result
}

/// Resolve a MemberFunctionType index to (return_type, parameters).
pub fn resolve_method_signature(
    finder: &pdb::TypeFinder,
    type_idx: pdb::TypeIndex,
) -> (Option<String>, Vec<String>) {
    if let Ok(item) = finder.find(type_idx) {
        if let Ok(pdb::TypeData::MemberFunction(mf)) = item.parse() {
            let ret = Some(resolve_type_name(finder, mf.return_type));
            let params = resolve_argument_list(finder, mf.argument_list);
            return (ret, params);
        }
    }
    (None, Vec::new())
}

/// Resolve an ArgumentList type index to a vec of type name strings.
pub fn resolve_argument_list(finder: &pdb::TypeFinder, idx: pdb::TypeIndex) -> Vec<String> {
    let Ok(item) = finder.find(idx) else {
        return Vec::new();
    };
    let Ok(pdb::TypeData::ArgumentList(al)) = item.parse() else {
        return Vec::new();
    };
    al.arguments
        .iter()
        .map(|&ti| resolve_type_name(finder, ti))
        .collect()
}

/// Resolve enum field list to variant values.
pub fn resolve_enum_values(finder: &pdb::TypeFinder, start: pdb::TypeIndex) -> Vec<EnumValue> {
    let mut values = Vec::new();
    let mut idx = Some(start);

    while let Some(current) = idx {
        let Ok(item) = finder.find(current) else { break };
        let Ok(pdb::TypeData::FieldList(fl)) = item.parse() else {
            break;
        };
        for field in &fl.fields {
            if let pdb::TypeData::Enumerate(e) = field {
                values.push(EnumValue {
                    name: e.name.to_string().into_owned(),
                    value: variant_to_i64(&e.value),
                });
            }
        }
        idx = fl.continuation;
    }

    values
}

/// Convert a pdb Variant to i64.
pub fn variant_to_i64(v: &pdb::Variant) -> i64 {
    match *v {
        pdb::Variant::U8(x) => x as i64,
        pdb::Variant::U16(x) => x as i64,
        pdb::Variant::U32(x) => x as i64,
        pdb::Variant::U64(x) => x as i64,
        pdb::Variant::I8(x) => x as i64,
        pdb::Variant::I16(x) => x as i64,
        pdb::Variant::I32(x) => x as i64,
        pdb::Variant::I64(x) => x,
    }
}

/// Access level string from FieldAttributes.
pub fn access_str(access: u8) -> &'static str {
    match access {
        1 => "private",
        2 => "protected",
        3 => "public",
        _ => "",
    }
}

/// Calling convention byte to human-readable name.
pub fn calling_convention_str(cc: u8) -> &'static str {
    match cc {
        0x00 | 0x01 => "cdecl",
        0x02 | 0x03 => "pascal",
        0x04 | 0x05 => "fastcall",
        0x07 | 0x08 => "stdcall",
        0x09 | 0x0A => "syscall",
        0x0B => "thiscall",
        0x0D => "generic",
        0x11 => "clrcall",
        _ => "unknown",
    }
}

/// Best-effort human-readable name for a type index.
pub fn resolve_type_name(finder: &pdb::TypeFinder, index: pdb::TypeIndex) -> String {
    if let Ok(item) = finder.find(index) {
        if let Ok(data) = item.parse() {
            return match data {
                pdb::TypeData::Primitive(ref p) => primitive_name(p),
                pdb::TypeData::Class(ref c) => c.name.to_string().into_owned(),
                pdb::TypeData::Union(ref u) => u.name.to_string().into_owned(),
                pdb::TypeData::Enumeration(ref e) => e.name.to_string().into_owned(),
                pdb::TypeData::Pointer(ref p) => {
                    format!("{}*", resolve_type_name(finder, p.underlying_type))
                }
                pdb::TypeData::Array(ref a) => {
                    format!("{}[]", resolve_type_name(finder, a.element_type))
                }
                pdb::TypeData::Modifier(ref m) => {
                    let inner = resolve_type_name(finder, m.underlying_type);
                    if m.constant {
                        format!("const {inner}")
                    } else if m.volatile {
                        format!("volatile {inner}")
                    } else {
                        inner
                    }
                }
                pdb::TypeData::Bitfield(ref b) => {
                    let base = resolve_type_name(finder, b.underlying_type);
                    format!("{base}:{}", b.length)
                }
                pdb::TypeData::Procedure(ref p) => {
                    let ret = p
                        .return_type
                        .map(|ti| resolve_type_name(finder, ti))
                        .unwrap_or_else(|| "void".to_string());
                    let params = resolve_argument_list(finder, p.argument_list);
                    format!("{ret}({})", params.join(", "))
                }
                pdb::TypeData::MemberFunction(ref mf) => {
                    let ret = resolve_type_name(finder, mf.return_type);
                    let params = resolve_argument_list(finder, mf.argument_list);
                    format!("{ret}({})", params.join(", "))
                }
                _ => format!("type@{:#x}", index.0),
            };
        }
    }

    // Primitive type indices (< 0x1000) are not stored in the TPI stream
    if index.0 < 0x1000 {
        decode_primitive(index.0)
    } else {
        format!("type@{:#x}", index.0)
    }
}

/// Format a pdb::PrimitiveType into a C-like name.
pub fn primitive_name(p: &pdb::PrimitiveType) -> String {
    use pdb::PrimitiveKind::*;
    let base = match p.kind {
        NoType => "...",
        Void => "void",
        Char | RChar => "char",
        UChar => "unsigned char",
        WChar => "wchar_t",
        RChar16 => "char16_t",
        RChar32 => "char32_t",
        I8 => "int8_t",
        U8 => "uint8_t",
        Short => "short",
        UShort => "unsigned short",
        I16 => "int16_t",
        U16 => "uint16_t",
        Long => "long",
        ULong => "unsigned long",
        I32 => "int32_t",
        U32 => "uint32_t",
        Quad | I64 => "int64_t",
        UQuad | U64 => "uint64_t",
        I128 => "int128_t",
        U128 => "uint128_t",
        F16 => "float16",
        F32 => "float",
        F64 => "double",
        F80 => "long double",
        F128 => "float128",
        Bool8 => "bool",
        Bool16 => "bool16",
        Bool32 => "bool32",
        Bool64 => "bool64",
        HRESULT => "HRESULT",
        _ => "?",
    };
    match p.indirection {
        Some(_) => format!("{base}*"),
        None => base.to_string(),
    }
}

/// Decode a raw primitive TypeIndex (< 0x1000) into a C-like name.
/// Layout: bits 0-7 = type kind, bits 8-11 = pointer indirection.
pub fn decode_primitive(raw: u32) -> String {
    let kind = raw & 0xFF;
    let ptr = (raw >> 8) & 0xF;

    let base = match kind {
        0x00 => "...",
        0x03 => "void",
        0x08 => "HRESULT",
        0x10 => "char",
        0x11 => "short",
        0x12 => "long",
        0x13 => "int64",
        0x14 => "int128",
        0x20 => "unsigned char",
        0x21 => "unsigned short",
        0x22 => "unsigned long",
        0x23 => "uint64",
        0x24 => "uint128",
        0x30 => "bool",
        0x31 => "bool16",
        0x32 => "bool32",
        0x33 => "bool64",
        0x40 => "float",
        0x41 => "double",
        0x42 => "long double",
        0x43 => "float128",
        0x46 => "float16",
        0x68 => "int8_t",
        0x69 => "uint8_t",
        0x70 => "wchar_t",
        0x71 => "char16_t",
        0x72 => "int16_t",
        0x73 => "uint16_t",
        0x74 => "int32_t",
        0x75 => "uint32_t",
        0x76 => "int64_t",
        0x77 => "uint64_t",
        0x78 => "int128_t",
        0x79 => "uint128_t",
        0x7a => "char32_t",
        _ => "?",
    };

    if ptr != 0 {
        format!("{base}*")
    } else {
        base.to_string()
    }
}
