use std::{collections::HashMap, rc::Rc};

use crate::{
    error::{Error, Result},
    parsing::ast::{Expression, SpannedString, Statement, Type, UnaryOperator},
    span::{spanned, Span},
};

pub(crate) type SymbolId = usize;
pub(crate) type SymbolTable = HashMap<String, SymbolId>;

#[derive(Clone)]
pub(crate) struct CodegenState {
    pub(crate) symbol_table: SymbolTable,
    pub(crate) symbols: Vec<(SymbolId, SymbolType)>,
}

#[derive(Clone)]
pub(crate) enum SymbolType {
    Variable,
    Function,
    Struct,
    Enum,
    Union,
}

impl CodegenState {
    pub(crate) fn new() -> Self {
        Self {
            symbol_table: HashMap::new(),
            symbols: Vec::new(),
        }
    }

    pub(crate) fn get_symbol_from_id(&self, id: SymbolId) -> String {
        for (i, symbol) in self.symbols.iter().enumerate() {
            if symbol.0 == id {
                return self
                    .symbol_table
                    .iter()
                    .find(|(_, v)| **v == i)
                    .unwrap()
                    .0
                    .clone();
            }
        }
        unreachable!()
    }
}

pub(crate) fn get_symbols(state: &mut CodegenState, statements: &Vec<Statement>) -> Result<()> {
    for statement in statements {
        get_symbol(state, statement)?;
    }
    Ok(())
}

pub(crate) fn get_symbol(state: &mut CodegenState, statement: &Statement) -> Result<()> {
    match statement {
        Statement::Variable(spanned_id, _, _) | Statement::Constant(spanned_id, _, _) => {
            let symbol_id: SymbolId = state.symbols.len();
            state.symbol_table.insert(spanned_id.0.clone(), symbol_id);
            state.symbols.push((symbol_id, SymbolType::Variable));
            Ok(())
        }
        Statement::Enum { spanned_id, .. } => {
            let symbol_id: SymbolId = state.symbols.len();
            state.symbol_table.insert(spanned_id.0.clone(), symbol_id);
            state.symbols.push((symbol_id, SymbolType::Enum));
            Ok(())
        }
        Statement::Block(statements) => {
            for statement in statements {
                get_symbol(state, statement)?;
            }
            Ok(())
        }
        Statement::Function(spanned_id, _, _, body) => {
            let symbol_id: SymbolId = state.symbols.len();
            state.symbol_table.insert(spanned_id.0.clone(), symbol_id);
            state.symbols.push((symbol_id, SymbolType::Function));
            get_symbol(state, body)?;
            Ok(())
        }
        _ => Ok(()),
    }
}

pub(crate) fn codegen(
    state: &mut CodegenState,
    filename: &str,
    statements: Vec<Statement>,
) -> Result<(String, String)> {
    let mut cpp: String = format!("#include <prelude.hpp>\n#include \"{}.hpp\"\n", filename);
    let mut hpp: String = format!("#pragma once\n#include <prelude.hpp>\n");
    for statement in statements {
        let (cpp_statement, hpp_statement) = codegen_statement(state, &statement)?;
        cpp.push_str(&cpp_statement);
        hpp.push_str(&hpp_statement);
    }
    Ok((cpp, hpp))
}

fn codegen_statement(state: &mut CodegenState, statement: &Statement) -> Result<(String, String)> {
    let cpp: String;
    let hpp: String;
    match statement {
        Statement::Import(spanned_id) => {
            hpp = format!("#include <{}.hpp>\n", spanned_id.0);
            Ok((String::new(), hpp))
        }
        Statement::Struct {
            spanned_id,
            generic_parameters,
            fields,
        } => {
            let mut hpp_fields: String = String::new();
            for field in fields {
                hpp_fields.push_str(&format!(
                    "    {} {};\n",
                    codegen_type(state, &field.1),
                    field.0 .0
                ));
            }
            let mut hpp_generic_parameters: String = String::new();
            for (i, generic_parameter) in generic_parameters.iter().enumerate() {
                hpp_generic_parameters.push_str(&format!("typename {}", generic_parameter.0));
                if i != generic_parameters.len() - 1 {
                    hpp_generic_parameters.push_str(", ");
                }
            }
            let mut hpp_generic_parameters_no_typename: String = String::new();
            for (i, generic_parameter) in generic_parameters.iter().enumerate() {
                hpp_generic_parameters_no_typename.push_str(&format!("{}", generic_parameter.0));
                if i != generic_parameters.len() - 1 {
                    hpp_generic_parameters_no_typename.push_str(", ");
                }
            }
            let mut constructor_code: String = format!("{}(", spanned_id.0.clone());
            for (i, field) in fields.iter().enumerate() {
                constructor_code.push_str(&format!(
                    "{} {}",
                    codegen_type(state, &field.1),
                    field.0 .0
                ));
                if i != fields.len() - 1 {
                    constructor_code.push_str(", ");
                }
            }
            constructor_code.push_str(
                format!(
                    ") {}",
                    if fields.len() == 0 {
                        String::new()
                    } else {
                        format!(": ")
                    }
                )
                .as_str(),
            );
            for (i, field) in fields.iter().enumerate() {
                constructor_code.push_str(&format!("{}({})", field.0 .0, field.0 .0));
                if i != fields.len() - 1 {
                    constructor_code.push_str(", ");
                }
            }
            constructor_code.push_str(" {}\n");
            constructor_code.push_str(format!("    ~{}() = default;\n", spanned_id.0).as_str());
            let mut formatter_code: String = "std::string format() const {\n".to_string();
            formatter_code.push_str("    std::stringstream ss;\n");
            formatter_code.push_str(format!("    ss << \"{}{{\"", spanned_id.0.clone()).as_str());
            for (i, field) in fields.iter().enumerate() {
                formatter_code.push_str(&format!(" << \"{} = \" << {}", field.0 .0, field.0 .0));
                if i != fields.len() - 1 {
                    formatter_code.push_str(" << \", \"");
                }
            }
            formatter_code.push_str(" << \"}\";\n");
            formatter_code.push_str("    return ss.str();\n");
            formatter_code.push_str("}\n");
            hpp = format!(
                "{}struct {} : public Formatter<{}{}> {{\n{}\n{}\n{}\n}};\n",
                if generic_parameters.len() == 0 {
                    String::new()
                } else {
                    format!("template <{}>\n", hpp_generic_parameters)
                },
                spanned_id.0.clone(),
                spanned_id.0.clone(),
                if generic_parameters.len() == 0 {
                    String::new()
                } else {
                    format!("<{}>", hpp_generic_parameters_no_typename)
                },
                hpp_fields.clone(),
                constructor_code.clone(),
                formatter_code.clone()
            );
            Ok((String::new(), hpp))
        }
        Statement::Enum {
            spanned_id,
            generic_parameters,
            base_type,
            variants,
        } => {
            let mut hpp_variants: String = String::new();
            for variant in variants {
                hpp_variants.push_str(&format!("    {}", variant.0 .0));
                if let Some(expression) = &variant.1 {
                    hpp_variants
                        .push_str(&format!(" = {}", codegen_expression(state, expression)?));
                }
                hpp_variants.push_str(",\n");
            }
            let mut hpp_generic_parameters: String = String::new();
            for (i, generic_parameter) in generic_parameters.iter().enumerate() {
                hpp_generic_parameters.push_str(&format!("typename {}", generic_parameter.0));
                if i != generic_parameters.len() - 1 {
                    hpp_generic_parameters.push_str(", ");
                }
            }
            hpp = format!(
                "{}enum class {}{} {{\n{}}};\n",
                if generic_parameters.len() == 0 {
                    String::new()
                } else {
                    format!("template <{}>\n", hpp_generic_parameters)
                },
                spanned_id.0.clone(),
                if let None = base_type {
                    String::new()
                } else {
                    format!(" : {}", codegen_type(state, base_type.as_ref().unwrap()))
                },
                hpp_variants.clone()
            );
            Ok((String::new(), hpp))
        }
        Statement::Union {
            spanned_id,
            generic_parameters,
            tagged,
            variants,
        } => {
            let mut hpp_generic_parameters: String = String::new();
            for (i, generic_parameter) in generic_parameters.iter().enumerate() {
                hpp_generic_parameters.push_str(&format!("typename {}", generic_parameter.0));
                if i != generic_parameters.len() - 1 {
                    hpp_generic_parameters.push_str(", ");
                }
            }
            if *tagged {
                let mut code = format!("using {} = TaggedUnion<", spanned_id.0.clone());
                for (i, variant) in variants.iter().enumerate() {
                    code.push_str(&codegen_type(state, &variant.1));
                    if i != variants.len() - 1 {
                        code.push_str(", ");
                    }
                }
                code.push_str(">;\n");
                hpp = format!(
                    "{}{}",
                    if generic_parameters.len() == 0 {
                        String::new()
                    } else {
                        format!("template <{}>\n", hpp_generic_parameters)
                    },
                    code
                );
            } else {
                let mut hpp_variants: String = String::new();
                for variant in variants {
                    hpp_variants.push_str(&format!("    {}", codegen_type(state, &variant.1)));
                    if let Some(spanned_id) = &variant.0 {
                        hpp_variants.push_str(&format!(" {}", spanned_id.0));
                    }
                    hpp_variants.push_str(";\n");
                }
                hpp = format!(
                    "{}union {} {{\n{}}};\n",
                    if generic_parameters.len() == 0 {
                        String::new()
                    } else {
                        format!("template <{}>\n", hpp_generic_parameters)
                    },
                    spanned_id.0.clone(),
                    hpp_variants.clone()
                );
            }
            Ok((String::new(), hpp))
        }
        Statement::Function(spanned_id, parameters, ret, body) => {
            let mut hpp_parameters: String = String::new();
            for (i, parameter) in parameters.iter().enumerate() {
                hpp_parameters.push_str(&format!(
                    "{} {}",
                    codegen_type(state, &parameter.1),
                    parameter.0 .0
                ));
                if i != parameters.len() - 1 {
                    hpp_parameters.push_str(", ");
                }
            }
            hpp = format!(
                "{} {}({});\n",
                if let None = ret {
                    "void".to_string()
                } else {
                    codegen_type(state, ret.as_ref().unwrap())
                },
                spanned_id.0.clone(),
                hpp_parameters.clone()
            );
            let mut cpp_parameters: String = String::new();
            for (i, parameter) in parameters.iter().enumerate() {
                cpp_parameters.push_str(&format!(
                    "{} {}",
                    codegen_type(state, &parameter.1),
                    parameter.0 .0
                ));
                if i != parameters.len() - 1 {
                    cpp_parameters.push_str(", ");
                }
            }
            let (cpp_body, _) = codegen_statement(state, body)?;
            cpp = format!(
                "{} {}({}) {{\n{}}}\n",
                if let None = ret {
                    "void".to_string()
                } else {
                    codegen_type(state, ret.as_ref().unwrap())
                },
                spanned_id.0.clone(),
                cpp_parameters.clone(),
                cpp_body.clone()
            );
            Ok((cpp, hpp))
        }
        Statement::Constant(spanned_id, ty, expression) => {
            let expression = codegen_expression(state, expression)?;
            cpp = format!(
                "const {} {} = {};\n",
                if let None = ty {
                    "auto".to_string()
                } else {
                    codegen_type(state, ty.as_ref().unwrap())
                },
                spanned_id.0,
                expression
            );
            Ok((cpp, String::new()))
        }
        Statement::Variable(spanned_id, ty, expression) => {
            let expression = codegen_expression(state, expression)?;
            cpp = format!(
                "{} {} = {};\n",
                if let None = ty {
                    "auto".to_string()
                } else {
                    codegen_type(state, ty.as_ref().unwrap())
                },
                spanned_id.0,
                expression
            );
            Ok((cpp, String::new()))
        }
        Statement::Block(statements) => {
            let mut cpp_statements: String = "{\n".to_string();
            for statement in statements {
                let (cpp_statement, _) = codegen_statement(state, statement)?;
                cpp_statements.push_str(&cpp_statement);
            }
            cpp_statements.push_str("}\n");
            Ok((cpp_statements, String::new()))
        }
        Statement::Return(expression) => {
            if let None = expression {
                cpp = "return;\n".to_string();
            } else {
                let expression = codegen_expression(state, expression.as_ref().unwrap())?;
                cpp = format!("return {};\n", expression);
            }
            Ok((cpp, String::new()))
        }
        Statement::For(spanned_id, expression, body) => {
            let expression = codegen_expression(state, expression)?;
            let (cpp_body, _) = codegen_statement(state, body)?;
            cpp = format!(
                "for (auto {} : {}) {{\n{}}}\n",
                spanned_id.0.clone(),
                expression,
                cpp_body
            );
            Ok((cpp, String::new()))
        }
        Statement::Expression(expression) => {
            let expression = codegen_expression(state, expression)?;
            cpp = format!("{};\n", expression);
            Ok((cpp, String::new()))
        }
        _ => Ok((String::new(), String::new())),
    }
}

fn codegen_expression(state: &mut CodegenState, expr: &Expression) -> Result<String> {
    match expr {
        Expression::Integer(spanned_int) => Ok(spanned_int.0.clone().to_string()),
        Expression::FloatingPoint(spanned_float) => Ok(spanned_float.0.clone().to_string()),
        Expression::Character(spanned_char) => Ok(spanned_char.0.clone().to_string()),
        Expression::String(spanned_string) => Ok(format!("\"{}\"", spanned_string.0.clone())),
        Expression::Boolean(spanned_bool) => Ok(spanned_bool.0.clone().to_string()),
        Expression::Identifier(spanned_id) => Ok(spanned_id.0.clone()),
        Expression::ArrayLiteral(expressions) => {
            let mut result: String = String::new();
            result.push_str("{");
            for (i, expression) in expressions.iter().enumerate() {
                result.push_str(&codegen_expression(state, expression)?);
                if i != expressions.len() - 1 {
                    result.push_str(", ");
                }
            }
            result.push_str("}");
            Ok(result)
        }
        Expression::StructLiteral(expression, fields) => {
            let mut result: String = String::new();
            result.push_str(&codegen_expression(state, expression)?);
            result.push_str("{");
            for (i, field) in fields.iter().enumerate() {
                result.push_str(&format!("{}", codegen_expression(state, &field.1)?));
                if i != fields.len() - 1 {
                    result.push_str(", ");
                }
            }
            result.push_str("}");
            Ok(result)
        }
        Expression::Call(expression, arguments) => {
            let mut result: String = String::new();
            result.push_str(&codegen_expression(state, expression)?);
            result.push_str("(");
            for (i, argument) in arguments.iter().enumerate() {
                result.push_str(&codegen_expression(state, argument)?);
                if i != arguments.len() - 1 {
                    result.push_str(", ");
                }
            }
            result.push_str(")");
            Ok(result)
        }
        Expression::Member(expression, member) => {
            let mut result: String = String::new();
            result.push_str(&codegen_expression(state, expression)?);
            result.push_str(match get_checked_type_of_expression(state, expression)? {
                CheckedType::Enum(_) => "::",
                _ => ".",
            });
            result.push_str(&format!("{}", member.0));
            Ok(result)
        }
        Expression::Unary(operator, expression) => {
            let expression = codegen_expression(state, expression)?;
            match operator {
                UnaryOperator::Minus => Ok(format!("-{}", expression)),
                UnaryOperator::Bang => Ok(format!("!{}", expression)),
                UnaryOperator::Plus => Ok(format!("+{}", expression)),
                UnaryOperator::AddressOf => Ok(format!("{}", expression)),
                UnaryOperator::MutableAddressOf => Ok(format!("{}", expression)),
                _ => unreachable!(),
            }
        }
        _ => Ok(String::new()),
    }
}

fn codegen_type(state: &mut CodegenState, ty: &Type) -> String {
    match ty {
        Type::Integer => "int".to_string(),
        Type::UnsignedInteger => "unsigned int".to_string(),
        Type::FloatingPoint => "float".to_string(),
        Type::Character => "char".to_string(),
        Type::Boolean => "bool".to_string(),
        Type::Reference(inner) => format!("const {}&", codegen_type(state, inner)),
        Type::MutableReference(inner) => format!("{}&", codegen_type(state, inner)),
        Type::Function(types, ret) => {
            let mut result: String = String::new();
            result.push_str(&format!("{}(", codegen_type(state, ret)));
            for (i, ty) in types.iter().enumerate() {
                result.push_str(&codegen_type(state, ty));
                if i != types.len() - 1 {
                    result.push_str(", ");
                }
            }
            result.push_str(")");
            result
        }
        Type::Member(inner, member) => format!("{}::{}", codegen_type(state, inner), member.0),
        Type::Identifier(spanned_id) => spanned_id.0.clone(),
        Type::Polymorphic(spanned_id, types) => {
            let mut result: String = String::new();
            result.push_str(&format!("{}<", spanned_id.0));
            for (i, ty) in types.iter().enumerate() {
                result.push_str(&codegen_type(state, ty));
                if i != types.len() - 1 {
                    result.push_str(", ");
                }
            }
            result.push_str(">");
            result
        }
    }
}

#[derive(Clone)]
pub(crate) enum CheckedType {
    Integer,
    UnsignedInteger,
    FloatingPoint,
    Boolean,
    Character,
    Reference(Box<CheckedType>),                  // &T
    MutableReference(Box<CheckedType>),           // &mut T
    Identifier(SpannedString),                    // T
    Enum(SpannedString),                          // T
    Polymorphic(SpannedString, Vec<CheckedType>), // T[T, ...]
    Member(Box<CheckedType>, SpannedString),      // T.T
    Function(Vec<CheckedType>, Box<CheckedType>), // fn(T, ...) -> T
}

impl PartialEq for CheckedType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            _ => false,
        }
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

fn get_checked_type_of_expression(
    state: &mut CodegenState,
    expression: &Expression,
) -> Result<CheckedType> {
    match expression {
        Expression::Integer(_) => Ok(CheckedType::Integer),
        Expression::FloatingPoint(_) => Ok(CheckedType::FloatingPoint),
        Expression::Character(_) => Ok(CheckedType::Character),
        Expression::String(_) => unreachable!(),
        Expression::Boolean(_) => Ok(CheckedType::Boolean),
        Expression::Identifier(spanned_id) => {
            let symbol_id: Option<&SymbolId> = state.symbol_table.get(&spanned_id.0);
            if symbol_id == None {
                return Err(Error::new(
                    format!("undefined symbol `{}`", spanned_id.0),
                    spanned_id.1.clone(),
                ));
            }
            let symbol_id: SymbolId = symbol_id.unwrap().clone();
            let symbol_type: SymbolType = state.symbols[symbol_id].1.clone();
            match symbol_type {
                SymbolType::Enum => {
                    let symbol_type: SymbolId = state.symbols[symbol_id].0.clone();
                    Ok(CheckedType::Enum(spanned(
                        state.get_symbol_from_id(symbol_type),
                        spanned_id.1.clone(),
                    )))
                }
                SymbolType::Variable => {
                    let symbol_type: SymbolId = state.symbols[symbol_id].0.clone();
                    Ok(CheckedType::Identifier(spanned(
                        state.get_symbol_from_id(symbol_type),
                        spanned_id.1.clone(),
                    )))
                }
                _ => unreachable!(),
            }
        }
        Expression::ArrayLiteral(expressions) => {
            let mut types: Vec<CheckedType> = Vec::new();
            for expression in expressions {
                types.push(get_checked_type_of_expression(state, expression)?);
            }
            let mut result: CheckedType = types[0].clone();
            for ty in types {
                if ty != result {
                    result = CheckedType::Polymorphic(
                        (
                            "Array".to_string(),
                            (Rc::new("".to_string()), (0, 0)..(0, 0)),
                        ),
                        vec![result, ty],
                    );
                }
            }
            Ok(CheckedType::Polymorphic(
                (
                    "Array".to_string(),
                    (Rc::new("".to_string()), (0, 0)..(0, 0)),
                ),
                vec![result],
            ))
        }
        _ => unreachable!(),
    }
}
