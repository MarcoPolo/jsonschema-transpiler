/// https://avro.apache.org/docs/current/spec.html
use super::ast;
use super::TranslateFrom;
use super::{Context, ResolveMethod};
use dhall::builtins::Builtin;
use dhall::syntax as s;
use dhall::syntax::ExprKind;
use serde_json::{json, Value};

impl<SubExpr> TranslateFrom<ast::Tag> for ExprKind<SubExpr> {
    type Error = String;
    fn translate_from(tag: ast::Tag, context: Context) -> Result<Self, Self::Error> {
        // Ok(ExprKind::Num(s::NumKind::Natural(3)))

        let fmt_reason =
            |reason: &str| -> String { format!("{} - {}", tag.fully_qualified_name(), reason) };
        let handle_error = |reason: &str| -> Result<ExprKind<SubExpr>, Self::Error> {
            let message = fmt_reason(reason);
            match context.resolve_method {
                ResolveMethod::Cast => {
                    warn!("{}", message);
                    Ok(ExprKind::TextLit(s::InterpolatedText::<SubExpr>::from(
                        String::from("Fail"),
                    )))
                }
                ResolveMethod::Drop => Err(message),
                ResolveMethod::Panic => panic!(message),
            }
        };

        let data_type = match &tag.data_type {
            // ast::Type::Null => Type::Primitive(Primitive::Null),
            // ast::Type::Atom(atom) => Type::Primitive(match atom {
            //     ast::Atom::Boolean => Primitive::Boolean,
            //     ast::Atom::Integer => Primitive::Long,
            //     ast::Atom::Number => Primitive::Double,
            //     ast::Atom::String => Primitive::String,
            //     ast::Atom::Datetime => Primitive::String,
            //     ast::Atom::Bytes => Primitive::Bytes,
            //     ast::Atom::JSON => match handle_error("json atom") {
            //         Ok(_) => Primitive::String,
            //         Err(reason) => return Err(reason),
            //     },
            // }),
            // ast::Type::Object(object) => {
            //     let mut fields: Vec<Field> = if object.fields.is_empty() {
            //         Vec::new()
            //     } else {
            //         object
            //             .fields
            //             .iter()
            //             .map(|(k, v)| {
            //                 let default = if v.nullable { Some(json!(null)) } else { None };
            //                 (
            //                     k.to_string(),
            //                     Type::translate_from(*v.clone(), context),
            //                     default,
            //                 )
            //             })
            //             .filter(|(_, v, _)| v.is_ok())
            //             .map(|(name, data_type, default)| Field {
            //                 name,
            //                 data_type: data_type.unwrap(),
            //                 default,
            //                 ..Default::default()
            //             })
            //             .collect()
            //     };

            //     if fields.is_empty() {
            //         handle_error("empty object")?
            //     } else {
            //         fields.sort_by_key(|v| v.name.to_string());
            //         let record = Record {
            //             common: CommonAttributes {
            //                 // This is not a safe assumption
            //                 name: tag.name.clone().unwrap_or_else(|| "__UNNAMED__".into()),
            //                 namespace: tag.namespace.clone(),
            //                 ..Default::default()
            //             },
            //             fields,
            //         };
            //         if record.common.name == "__UNNAMED__" {
            //             warn!("{} - Unnamed field", tag.fully_qualified_name());
            //         }
            //         Type::Complex(Complex::Record(record))
            //     }
            // }
            // ast::Type::Tuple(tuple) => {
            //     let fields = tuple
            //         .items
            //         .iter()
            //         .enumerate()
            //         .map(|(i, v)| {
            //             let default = if v.nullable { Some(json!(null)) } else { None };
            //             (
            //                 format!("f{}_", i),
            //                 Type::translate_from(v.clone(), context),
            //                 default,
            //             )
            //         })
            //         .filter(|(_, v, _)| v.is_ok())
            //         .map(|(name, data_type, default)| Field {
            //             name,
            //             data_type: data_type.unwrap(),
            //             default,
            //             ..Default::default()
            //         })
            //         .collect();
            //     let record = Record {
            //         common: CommonAttributes {
            //             name: tag.name.clone().unwrap_or_else(|| "__UNNAMED__".into()),
            //             namespace: tag.namespace.clone(),
            //             ..Default::default()
            //         },
            //         fields,
            //     };
            //     if record.common.name == "__UNNAMED__" {
            //         warn!("{} - Unnamed field", tag.fully_qualified_name());
            //     }
            //     Type::Complex(Complex::Record(record))
            // }
            // ast::Type::Array(array) => {
            //     let child_is_array = match &array.items.data_type {
            //         ast::Type::Array(_) => true,
            //         _ => false,
            //     };
            //     match Type::translate_from(*array.items.clone(), context) {
            //         Ok(data_type) => {
            //             if child_is_array {
            //                 Type::Complex(Complex::Array(Array {
            //                     items: Box::new(Type::Complex(Complex::Record(Record {
            //                         common: CommonAttributes {
            //                             name: tag
            //                                 .name
            //                                 .clone()
            //                                 .unwrap_or_else(|| "__UNNAMED__".into()),
            //                             namespace: tag.namespace.clone(),
            //                             ..Default::default()
            //                         },
            //                         fields: vec![Field {
            //                             name: "list".into(),
            //                             data_type,
            //                             ..Default::default()
            //                         }],
            //                     }))),
            //                 }))
            //             } else {
            //                 Type::Complex(Complex::Array(Array {
            //                     items: Box::new(data_type),
            //                 }))
            //             }
            //         }
            //         Err(_) => return Err(fmt_reason("untyped array")),
            //     }
            // }
            // ast::Type::Map(map) => match Type::translate_from(*map.value.clone(), context) {
            //     Ok(data_type) => Type::Complex(Complex::Map(Map {
            //         values: Box::new(data_type),
            //     })),
            //     // Err is only reachable when context.resolve_method is Drop
            //     Err(_) => {
            //         return if context.allow_maps_without_value {
            //             Err(fmt_reason("map value cannot be dropped in avro"))
            //         } else {
            //             Err(fmt_reason("untyped map value"))
            //         }
            //     }
            // },
            _ => handle_error("unknown type")?,
        };
        // if tag.nullable && !tag.is_null() {
        //     Ok(Type::Union(vec![
        //         Type::Primitive(Primitive::Null),
        //         data_type,
        //     ]))
        // } else {
        //     Ok(data_type)
        // }
        return Ok(ExprKind::Num(s::NumKind::Natural(3)));
    }
}

// impl TranslateFrom<ast::Tag> for Type {
//     type Error = String;

//     fn translate_from(tag: ast::Tag, context: Context) -> Result<Self, Self::Error> {
//         let mut tag = tag;
//         if tag.is_root {
//             // Name inference is run only from the root for the proper
//             // construction of the namespace. Fully qualified names require a
//             // top-down approach.
//             tag.collapse();
//             tag.name = Some("root".into());
//             tag.infer_name(context.normalize_case);
//         }
//         tag.infer_nullability(context.force_nullable);

//         let fmt_reason =
//             |reason: &str| -> String { format!("{} - {}", tag.fully_qualified_name(), reason) };
//         let handle_error = |reason: &str| -> Result<Type, Self::Error> {
//             let message = fmt_reason(reason);
//             match context.resolve_method {
//                 ResolveMethod::Cast => {
//                     warn!("{}", message);
//                     Ok(Type::Primitive(Primitive::String))
//                 }
//                 ResolveMethod::Drop => Err(message),
//                 ResolveMethod::Panic => panic!(message),
//             }
//         };

//         let data_type = match &tag.data_type {
//             ast::Type::Null => Type::Primitive(Primitive::Null),
//             ast::Type::Atom(atom) => Type::Primitive(match atom {
//                 ast::Atom::Boolean => Primitive::Boolean,
//                 ast::Atom::Integer => Primitive::Long,
//                 ast::Atom::Number => Primitive::Double,
//                 ast::Atom::String => Primitive::String,
//                 ast::Atom::Datetime => Primitive::String,
//                 ast::Atom::Bytes => Primitive::Bytes,
//                 ast::Atom::JSON => match handle_error("json atom") {
//                     Ok(_) => Primitive::String,
//                     Err(reason) => return Err(reason),
//                 },
//             }),
//             ast::Type::Object(object) => {
//                 let mut fields: Vec<Field> = if object.fields.is_empty() {
//                     Vec::new()
//                 } else {
//                     object
//                         .fields
//                         .iter()
//                         .map(|(k, v)| {
//                             let default = if v.nullable { Some(json!(null)) } else { None };
//                             (
//                                 k.to_string(),
//                                 Type::translate_from(*v.clone(), context),
//                                 default,
//                             )
//                         })
//                         .filter(|(_, v, _)| v.is_ok())
//                         .map(|(name, data_type, default)| Field {
//                             name,
//                             data_type: data_type.unwrap(),
//                             default,
//                             ..Default::default()
//                         })
//                         .collect()
//                 };

//                 if fields.is_empty() {
//                     handle_error("empty object")?
//                 } else {
//                     fields.sort_by_key(|v| v.name.to_string());
//                     let record = Record {
//                         common: CommonAttributes {
//                             // This is not a safe assumption
//                             name: tag.name.clone().unwrap_or_else(|| "__UNNAMED__".into()),
//                             namespace: tag.namespace.clone(),
//                             ..Default::default()
//                         },
//                         fields,
//                     };
//                     if record.common.name == "__UNNAMED__" {
//                         warn!("{} - Unnamed field", tag.fully_qualified_name());
//                     }
//                     Type::Complex(Complex::Record(record))
//                 }
//             }
//             ast::Type::Tuple(tuple) => {
//                 let fields = tuple
//                     .items
//                     .iter()
//                     .enumerate()
//                     .map(|(i, v)| {
//                         let default = if v.nullable { Some(json!(null)) } else { None };
//                         (
//                             format!("f{}_", i),
//                             Type::translate_from(v.clone(), context),
//                             default,
//                         )
//                     })
//                     .filter(|(_, v, _)| v.is_ok())
//                     .map(|(name, data_type, default)| Field {
//                         name,
//                         data_type: data_type.unwrap(),
//                         default,
//                         ..Default::default()
//                     })
//                     .collect();
//                 let record = Record {
//                     common: CommonAttributes {
//                         name: tag.name.clone().unwrap_or_else(|| "__UNNAMED__".into()),
//                         namespace: tag.namespace.clone(),
//                         ..Default::default()
//                     },
//                     fields,
//                 };
//                 if record.common.name == "__UNNAMED__" {
//                     warn!("{} - Unnamed field", tag.fully_qualified_name());
//                 }
//                 Type::Complex(Complex::Record(record))
//             }
//             ast::Type::Array(array) => {
//                 let child_is_array = match &array.items.data_type {
//                     ast::Type::Array(_) => true,
//                     _ => false,
//                 };
//                 match Type::translate_from(*array.items.clone(), context) {
//                     Ok(data_type) => {
//                         if child_is_array {
//                             Type::Complex(Complex::Array(Array {
//                                 items: Box::new(Type::Complex(Complex::Record(Record {
//                                     common: CommonAttributes {
//                                         name: tag
//                                             .name
//                                             .clone()
//                                             .unwrap_or_else(|| "__UNNAMED__".into()),
//                                         namespace: tag.namespace.clone(),
//                                         ..Default::default()
//                                     },
//                                     fields: vec![Field {
//                                         name: "list".into(),
//                                         data_type,
//                                         ..Default::default()
//                                     }],
//                                 }))),
//                             }))
//                         } else {
//                             Type::Complex(Complex::Array(Array {
//                                 items: Box::new(data_type),
//                             }))
//                         }
//                     }
//                     Err(_) => return Err(fmt_reason("untyped array")),
//                 }
//             }
//             ast::Type::Map(map) => match Type::translate_from(*map.value.clone(), context) {
//                 Ok(data_type) => Type::Complex(Complex::Map(Map {
//                     values: Box::new(data_type),
//                 })),
//                 // Err is only reachable when context.resolve_method is Drop
//                 Err(_) => {
//                     return if context.allow_maps_without_value {
//                         Err(fmt_reason("map value cannot be dropped in avro"))
//                     } else {
//                         Err(fmt_reason("untyped map value"))
//                     }
//                 }
//             },
//             _ => handle_error("unknown type")?,
//         };
//         if tag.nullable && !tag.is_null() {
//             Ok(Type::Union(vec![
//                 Type::Primitive(Primitive::Null),
//                 data_type,
//             ]))
//         } else {
//             Ok(data_type)
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    fn assert_from_ast_eq<S: std::fmt::Debug + std::cmp::PartialEq>(
        dhall: ExprKind<S>,
        ast: Value,
    ) {
        let context = Context {
            ..Default::default()
        };
        let tag: ast::Tag = serde_json::from_value(ast).unwrap();
        let from_tag = ExprKind::<S>::translate_from(tag, context).unwrap();
        assert_eq!(dhall, from_tag)
    }

    #[test]
    fn from_ast_datetime() {
        let context = Context {
            ..Default::default()
        };

        // let ast = json!({"type": {"atom": "datetime"}});
        let ast = json!({ "type": "integer" });
        let dhall = ExprKind::Builtin(Builtin::Natural);
        println!("Dhall: {}", dhall);
        let tag: ast::Tag = serde_json::from_value(ast).unwrap();
        let from_tag = ExprKind::<String>::translate_from(tag, context).unwrap();
        assert_eq!(dhall, from_tag)
    }
}
