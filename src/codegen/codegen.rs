use crate::{
    error::{Error, Result},
    parsing::ast::{Expression, Statement, Type, UnaryOperator},
};

pub(crate) fn codegen(filename: &str, statements: Vec<Statement>) -> Result<(String, String)> {
    let mut cpp: String = format!("#include <prelude.hpp>\n#include \"{}.hpp\"\n", filename);
    let mut hpp: String = format!("#pragma once\n#include <prelude.hpp>\n");
    for statement in statements {
        let (cpp_statement, hpp_statement) = codegen_statement(&statement)?;
        cpp.push_str(&cpp_statement);
        hpp.push_str(&hpp_statement);
    }
    Ok((cpp, hpp))
}

fn codegen_statement(statement: &Statement) -> Result<(String, String)> {
    let cpp: String;
    let hpp: String;
    match statement {
        Statement::Import(spanned_id) => {
            hpp = format!("#include <{}.hpp>\n", spanned_id.0);
            Ok((String::new(), hpp))
        }
        Statement::Struct(spanned_id, fields) => {
            let mut hpp_fields: String = String::new();
            for field in fields {
                hpp_fields.push_str(&format!("    {} {};\n", codegen_type(&field.1), field.0 .0));
            }
            hpp = format!(
                "struct {} {{\n{}}};\n",
                spanned_id.0.clone(),
                hpp_fields.clone()
            );
            Ok((String::new(), hpp))
        }
        Statement::Enum {
            spanned_id,
            base_type,
            variants,
        } => {
            let mut hpp_variants: String = String::new();
            for variant in variants {
                hpp_variants.push_str(&format!("    {}", variant.0 .0));
                if let Some(expression) = &variant.1 {
                    hpp_variants.push_str(&format!(" = {}", codegen_expression(expression)?));
                }
                hpp_variants.push_str(",\n");
            }
            hpp = format!(
                "enum class {}{} {{\n{}}};\n",
                spanned_id.0.clone(),
                if let None = base_type {
                    String::new()
                } else {
                    format!(" : {}", codegen_type(base_type.as_ref().unwrap()))
                },
                hpp_variants.clone()
            );
            Ok((String::new(), hpp))
        }
        Statement::Union {
            spanned_id,
            tagged,
            variants,
        } => {
            if *tagged {
                let mut code = format!("using {} = TaggedUnion<", spanned_id.0.clone());
                for (i, variant) in variants.iter().enumerate() {
                    code.push_str(&codegen_type(&variant.1));
                    if i != variants.len() - 1 {
                        code.push_str(", ");
                    }
                }
                code.push_str(">;\n");
                hpp = code;
            } else {
                let mut hpp_variants: String = String::new();
                for variant in variants {
                    hpp_variants.push_str(&format!("    {}", codegen_type(&variant.1)));
                    if let Some(spanned_id) = &variant.0 {
                        hpp_variants.push_str(&format!(" {}", spanned_id.0));
                    }
                    hpp_variants.push_str(";\n");
                }
                hpp = format!(
                    "union {} {{\n{}}};\n",
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
                    codegen_type(&parameter.1),
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
                    codegen_type(ret.as_ref().unwrap())
                },
                spanned_id.0.clone(),
                hpp_parameters.clone()
            );
            let mut cpp_parameters: String = String::new();
            for (i, parameter) in parameters.iter().enumerate() {
                cpp_parameters.push_str(&format!(
                    "{} {}",
                    codegen_type(&parameter.1),
                    parameter.0 .0
                ));
                if i != parameters.len() - 1 {
                    cpp_parameters.push_str(", ");
                }
            }
            let (cpp_body, _) = codegen_statement(body)?;
            cpp = format!(
                "{} {}({}) {{\n{}}}\n",
                if let None = ret {
                    "void".to_string()
                } else {
                    codegen_type(ret.as_ref().unwrap())
                },
                spanned_id.0.clone(),
                cpp_parameters.clone(),
                cpp_body.clone()
            );
            Ok((cpp, hpp))
        }
        Statement::Constant(spanned_id, ty, expression) => {
            let expression = codegen_expression(expression)?;
            cpp = format!(
                "const {} {} = {};\n",
                if let None = ty {
                    "auto".to_string()
                } else {
                    codegen_type(ty.as_ref().unwrap())
                },
                spanned_id.0,
                expression
            );
            Ok((cpp, String::new()))
        }
        Statement::Variable(spanned_id, ty, expression) => {
            let expression = codegen_expression(expression)?;
            cpp = format!(
                "{} {} = {};\n",
                if let None = ty {
                    "auto".to_string()
                } else {
                    codegen_type(ty.as_ref().unwrap())
                },
                spanned_id.0,
                expression
            );
            Ok((cpp, String::new()))
        }
        Statement::Block(statements) => {
            let mut cpp_statements: String = "{\n".to_string();
            for statement in statements {
                let (cpp_statement, _) = codegen_statement(statement)?;
                cpp_statements.push_str(&cpp_statement);
            }
            cpp_statements.push_str("}\n");
            Ok((cpp_statements, String::new()))
        }
        Statement::Return(expression) => {
            if let None = expression {
                cpp = "return;\n".to_string();
            } else {
                let expression = codegen_expression(expression.as_ref().unwrap())?;
                cpp = format!("return {};\n", expression);
            }
            Ok((cpp, String::new()))
        }
        Statement::For(spanned_id, expression, body) => {
            let expression = codegen_expression(expression)?;
            let (cpp_body, _) = codegen_statement(body)?;
            cpp = format!(
                "for (auto {} : {}) {{\n{}}}\n",
                spanned_id.0.clone(),
                expression,
                cpp_body
            );
            Ok((cpp, String::new()))
        }
        Statement::Expression(expression) => {
            let expression = codegen_expression(expression)?;
            cpp = format!("{};\n", expression);
            Ok((cpp, String::new()))
        }
        _ => Ok((String::new(), String::new())),
    }
}

fn codegen_expression(expr: &Expression) -> Result<String> {
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
                result.push_str(&codegen_expression(expression)?);
                if i != expressions.len() - 1 {
                    result.push_str(", ");
                }
            }
            result.push_str("}");
            Ok(result)
        }
        Expression::StructLiteral(expression, fields) => {
            let mut result: String = String::new();
            result.push_str(&codegen_expression(expression)?);
            result.push_str("{");
            for (i, field) in fields.iter().enumerate() {
                result.push_str(&format!("{}", codegen_expression(&field.1)?));
                if i != fields.len() - 1 {
                    result.push_str(", ");
                }
            }
            result.push_str("}");
            Ok(result)
        }
        Expression::Call(expression, arguments) => {
            let mut result: String = String::new();
            result.push_str(&codegen_expression(expression)?);
            result.push_str("(");
            for (i, argument) in arguments.iter().enumerate() {
                result.push_str(&codegen_expression(argument)?);
                if i != arguments.len() - 1 {
                    result.push_str(", ");
                }
            }
            result.push_str(")");
            Ok(result)
        }
        Expression::Member(expression, member) => {
            let mut result: String = String::new();
            result.push_str(&codegen_expression(expression)?);
            result.push_str(&format!(".{}", member.0));
            Ok(result)
        }
        Expression::Unary(operator, expression) => {
            let expression = codegen_expression(expression)?;
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

fn codegen_type(ty: &Type) -> String {
    match ty {
        Type::Integer => "int".to_string(),
        Type::UnsignedInteger => "unsigned int".to_string(),
        Type::FloatingPoint => "float".to_string(),
        Type::Character => "char".to_string(),
        Type::Boolean => "bool".to_string(),
        Type::Reference(inner) => format!("const {}&", codegen_type(inner)),
        Type::MutableReference(inner) => format!("{}&", codegen_type(inner)),
        Type::Function(types, ret) => {
            let mut result: String = String::new();
            result.push_str(&format!("{}(", codegen_type(ret)));
            for (i, ty) in types.iter().enumerate() {
                result.push_str(&codegen_type(ty));
                if i != types.len() - 1 {
                    result.push_str(", ");
                }
            }
            result.push_str(")");
            result
        }
        Type::Member(inner, member) => format!("{}::{}", codegen_type(inner), member.0),
        Type::Identifier(spanned_id) => spanned_id.0.clone(),
        Type::Polymorphic(spanned_id, types) => {
            let mut result: String = String::new();
            result.push_str(&format!("{}<", spanned_id.0));
            for (i, ty) in types.iter().enumerate() {
                result.push_str(&codegen_type(ty));
                if i != types.len() - 1 {
                    result.push_str(", ");
                }
            }
            result.push_str(">");
            result
        }
    }
}
