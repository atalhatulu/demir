use std::collections::HashMap;
use crate::ast::*;
use crate::hir::*;

#[derive(Debug)]
pub struct Diagnostic {
    pub message: String,
}

struct Symbol {
    id: usize,
    ty: Type,
    is_mut: bool,
}

struct FunctionSignature {
    params: Vec<Type>,
    return_type: Type,
}

pub struct Analyzer {
    env: Vec<HashMap<String, Symbol>>,
    functions: HashMap<String, FunctionSignature>,
    pub structs: HashMap<String, Vec<StructFieldInfo>>,
    next_id: usize,
    pub diagnostics: Vec<Diagnostic>,
}

impl Analyzer {
    pub fn new() -> Self {
        let mut analyzer = Self {
            env: vec![HashMap::new()],
            functions: HashMap::new(),
            structs: HashMap::new(),
            next_id: 1,
            diagnostics: Vec::new(),
        };
        
        // Mock standard library
        analyzer.functions.insert("std.io.print".to_string(), FunctionSignature {
            params: vec![Type::Int], // Basic mock for print
            return_type: Type::Unit,
        });

        analyzer
    }

    fn report_error(&mut self, message: String) {
        self.diagnostics.push(Diagnostic { message });
    }

    fn push_scope(&mut self) {
        self.env.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.env.pop();
    }

    fn declare(&mut self, name: String, ty: Type, is_mut: bool) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        self.env.last_mut().unwrap().insert(name, Symbol { id, ty, is_mut });
        id
    }

    fn resolve(&self, name: &str) -> Option<&Symbol> {
        for scope in self.env.iter().rev() {
            if let Some(symbol) = scope.get(name) {
                return Some(symbol);
            }
        }
        None
    }

    fn map_type(&self, ty_info: &TypeInfo) -> Type {
        match ty_info {
            TypeInfo::Int => Type::Int,
            TypeInfo::Float => Type::Float,
            TypeInfo::String => Type::String,
            TypeInfo::Bool => Type::Bool,
            TypeInfo::Char => Type::Char,
            TypeInfo::Unit => Type::Unit,
            TypeInfo::Ref(inner) => Type::Ref(Box::new(self.map_type(inner))),
            TypeInfo::MutRef(inner) => Type::MutRef(Box::new(self.map_type(inner))),
            TypeInfo::Custom(name) => {
                Type::Struct(name.clone())
            }
        }
    }

    // İlk geçiş: Global fonksiyonların imzalarını kaydet
    fn pre_declare_functions(&mut self, program: &Program) {
        for stmt in &program.statements {
            if let Statement::FunctionDeclaration { name, params, return_type, .. } = stmt {
                let mut param_types = Vec::new();
                for (_, ty_info) in params {
                    param_types.push(self.map_type(ty_info));
                }
                let ret_ty = return_type.as_ref().map(|t| self.map_type(t)).unwrap_or(Type::Unit);
                
                self.functions.insert(name.clone(), FunctionSignature {
                    params: param_types,
                    return_type: ret_ty,
                });
            }
        }
    }

    fn pre_declare_structs(&mut self, program: &Program) {
        for stmt in &program.statements {
            if let Statement::StructDeclaration { name, fields } = stmt {
                let mut hir_fields = Vec::new();
                for (f_name, f_ty_info) in fields {
                    hir_fields.push(StructFieldInfo {
                        name: f_name.clone(),
                        ty: self.map_type(f_ty_info),
                    });
                }
                self.structs.insert(name.clone(), hir_fields);
            }
        }
    }

    pub fn analyze_program(&mut self, program: Program) -> HirProgram {
        self.pre_declare_structs(&program);
        self.pre_declare_functions(&program);

        let mut statements = Vec::new();
        for stmt in program.statements {
            let hir_stmt = self.analyze_statement(stmt);
            statements.push(hir_stmt);
        }
        HirProgram { statements }
    }

    fn analyze_statement(&mut self, stmt: Statement) -> HirStatement {
        match stmt {
            Statement::FunctionDeclaration { name, params, return_type, requires, ensures, body } => {
                let ret_ty = return_type.map(|t| self.map_type(&t)).unwrap_or(Type::Unit);
                let id = self.declare(name.clone(), Type::Unit, false);
                
                self.push_scope();
                
                let mut hir_params = Vec::new();
                for (p_name, p_type_info) in params {
                    let p_ty = self.map_type(&p_type_info);
                    let p_id = self.declare(p_name, p_ty.clone(), false);
                    hir_params.push((p_id, p_ty));
                }

                let mut hir_requires = Vec::new();
                for req in requires {
                    let hir_req = self.analyze_expression(req);
                    if *hir_req.ty() != Type::Bool && *hir_req.ty() != Type::Error {
                        self.report_error(format!("Requires condition must be Bool, found {:?}", hir_req.ty()));
                    }
                    hir_requires.push(hir_req);
                }

                let mut hir_ensures = Vec::new();
                for ens in ensures {
                    let hir_ens_expr = self.analyze_expression(ens);
                    if *hir_ens_expr.ty() != Type::Bool && *hir_ens_expr.ty() != Type::Error {
                        self.report_error(format!("Ensures condition must be Bool, found {:?}", hir_ens_expr.ty()));
                    }
                    hir_ensures.push(hir_ens_expr);
                }

                let hir_body = self.analyze_block(body);
                self.pop_scope();
                
                HirStatement::FunctionDeclaration { 
                    id, 
                    name, 
                    params: hir_params, 
                    return_type: ret_ty,
                    requires: hir_requires,
                    ensures: hir_ensures,
                    body: hir_body 
                }
            }
            Statement::StructDeclaration { name, fields } => {
                let mut hir_fields = Vec::new();
                for (f_name, f_ty_info) in fields {
                    hir_fields.push(StructFieldInfo {
                        name: f_name,
                        ty: self.map_type(&f_ty_info),
                    });
                }
                HirStatement::StructDeclaration { name, fields: hir_fields }
            }
            Statement::LetDeclaration { name, ty, value } => {
                let expr = self.analyze_expression(value);
                
                if let Some(explicit_ty_info) = ty {
                    let explicit_ty = self.map_type(&explicit_ty_info);
                    if *expr.ty() != explicit_ty && *expr.ty() != Type::Error {
                        self.report_error(format!("Type mismatch in let '{}'. Expected {:?}, found {:?}", name, explicit_ty, expr.ty()));
                    }
                }

                let id = self.declare(name.clone(), expr.ty().clone(), false); 
                HirStatement::LetDeclaration { id, name, value: expr }
            }
            Statement::VarDeclaration { name, ty, value } => {
                let expr = self.analyze_expression(value);
                
                if let Some(explicit_ty_info) = ty {
                    let explicit_ty = self.map_type(&explicit_ty_info);
                    if *expr.ty() != explicit_ty && *expr.ty() != Type::Error {
                        self.report_error(format!("Type mismatch in var '{}'. Expected {:?}, found {:?}", name, explicit_ty, expr.ty()));
                    }
                }

                let id = self.declare(name.clone(), expr.ty().clone(), true);
                HirStatement::VarDeclaration { id, name, value: expr }
            }
            Statement::Assignment { target, value } => {
                let val_expr = self.analyze_expression(value);
                
                if let Expression::Dereference(inner) = target {
                    // Deref-store: *ptr = value
                    if let Expression::Identifier(name) = *inner {
                        if let Some(symbol) = self.resolve(&name) {
                            let sym_ty = symbol.ty.clone();
                            let ptr_id = symbol.id;
                            // Yalnızca mutable reference üzerinden yazılabilir.
                            if let Type::MutRef(pointee) = &sym_ty {
                                if *val_expr.ty() != **pointee && *val_expr.ty() != Type::Error {
                                    self.report_error(format!(
                                        "Type mismatch in deref-store: cannot assign {:?} to {:?}",
                                        val_expr.ty(),
                                        **pointee
                                    ));
                                }
                                return HirStatement::DerefStore { ptr_id, value: val_expr };
                            } else {
                                self.report_error(format!(
                                    "Cannot assign through `{}`: deref-store requires a mutable reference (`&mut`).",
                                    name
                                ));
                            }
                        } else {
                            self.report_error(format!("Undefined variable: {}", name));
                        }
                    } else {
                        self.report_error("Deref-store target must be an identifier".to_string());
                    }
                    return HirStatement::ExpressionStatement(val_expr);
                }

                if let Expression::Identifier(name) = target {
                    let mut found_id = None;
                    let mut is_mut = false;
                    let mut sym_ty = Type::Error;
                    
                    if let Some(symbol) = self.resolve(&name) {
                        found_id = Some(symbol.id);
                        is_mut = symbol.is_mut;
                        sym_ty = symbol.ty.clone();
                    }

                    if let Some(id) = found_id {
                        if !is_mut {
                            self.report_error(format!("Cannot assign to immutable variable '{}'.", name));
                        } else if sym_ty != *val_expr.ty() && *val_expr.ty() != Type::Error {
                            self.report_error(format!("Type mismatch in assignment. Cannot assign {:?} to {:?}", val_expr.ty(), sym_ty));
                        }
                        return HirStatement::Assignment { target_id: id, value: val_expr };
                    } else {
                        self.report_error(format!("Undefined variable: {}", name));
                    }
                } else {
                    self.report_error("Invalid assignment target".to_string());
                }
                
                HirStatement::ExpressionStatement(val_expr)
            }
            Statement::ExpressionStatement(expr) => {
                HirStatement::ExpressionStatement(self.analyze_expression(expr))
            }
            Statement::IfStatement { condition, then_branch, else_branch } => {
                let cond_expr = self.analyze_expression(condition);
                if *cond_expr.ty() != Type::Bool && *cond_expr.ty() != Type::Error {
                    self.report_error(format!("If condition must be Bool, found {:?}", cond_expr.ty()));
                }
                
                self.push_scope();
                let then_b = self.analyze_block(then_branch);
                self.pop_scope();

                let else_b = match else_branch {
                    Some(b) => {
                        self.push_scope();
                        let analyzed = self.analyze_block(b);
                        self.pop_scope();
                        Some(analyzed)
                    }
                    None => None,
                };

                HirStatement::IfStatement { condition: cond_expr, then_branch: then_b, else_branch: else_b }
            }
            Statement::WhileStatement { condition, body } => {
                let cond_expr = self.analyze_expression(condition);
                if *cond_expr.ty() != Type::Bool && *cond_expr.ty() != Type::Error {
                    self.report_error(format!("While condition must be Bool, found {:?}", cond_expr.ty()));
                }
                
                self.push_scope();
                let b = self.analyze_block(body);
                self.pop_scope();

                HirStatement::WhileStatement { condition: cond_expr, body: b }
            },
            Statement::Import(_) => panic!("Imports should be resolved before analysis"),
            Statement::ReturnStatement(value_opt) => {
                let val = value_opt.map(|e| self.analyze_expression(e));
                HirStatement::ReturnStatement(val)
            }
        }
    }

    fn analyze_block(&mut self, block: Block) -> HirBlock {
        let mut statements = Vec::new();
        for stmt in block.statements {
            statements.push(self.analyze_statement(stmt));
        }
        HirBlock { statements, ty: Type::Unit }
    }

    fn analyze_expression(&mut self, expr: Expression) -> HirExpression {
        match expr {
            Expression::Integer(val) => HirExpression::Integer(val, Type::Int),
            Expression::Float(val) => HirExpression::Float(val, Type::Float),
            Expression::StringLiteral(val) => HirExpression::StringLiteral(val, Type::String),
            Expression::Boolean(val) => HirExpression::Boolean(val, Type::Bool),
            Expression::Identifier(name) => {
                if let Some(symbol) = self.resolve(&name) {
                    HirExpression::Variable(symbol.id, symbol.ty.clone())
                } else {
                    self.report_error(format!("Undefined variable: {}", name));
                    HirExpression::Variable(0, Type::Error)
                }
            }
            Expression::Binary { left, operator, right } => {
                let lhs = self.analyze_expression(*left);
                let rhs = self.analyze_expression(*right);
                
                if lhs.ty() != rhs.ty() && *lhs.ty() != Type::Error && *rhs.ty() != Type::Error {
                    self.report_error(format!("Type mismatch in binary operation: {:?} {} {:?}", lhs.ty(), operator, rhs.ty()));
                }

                let ty = match operator.as_str() {
                    "==" | "!=" | "<" | ">" | "<=" | ">=" => Type::Bool,
                    "+" | "-" | "*" | "/" | "%" => lhs.ty().clone(),
                    "&&" | "||" => {
                        if *lhs.ty() != Type::Bool && *lhs.ty() != Type::Error {
                            self.report_error("Logical operators require Bool".to_string());
                        }
                        Type::Bool
                    },
                    _ => {
                        self.report_error(format!("Unknown operator: {}", operator));
                        Type::Error
                    }
                };

                HirExpression::Binary {
                    left: Box::new(lhs),
                    operator,
                    right: Box::new(rhs),
                    ty,
                }
            }
            Expression::Call { callee, arguments } => {
                let mut func_name = String::new();
                
                fn extract_func_name(expr: &Expression) -> Option<String> {
                    match expr {
                        Expression::Identifier(name) => Some(name.clone()),
                        Expression::MemberAccess { object, member } => {
                            if let Some(obj_name) = extract_func_name(object) {
                                Some(format!("{}.{}", obj_name, member))
                            } else {
                                None
                            }
                        }
                        _ => None,
                    }
                }
                
                if let Some(name) = extract_func_name(&callee) {
                    func_name = name;
                }

                let mut args = Vec::new();
                for arg in arguments {
                    args.push(self.analyze_expression(arg.clone()));
                }

                if func_name == "std.io.print" {
                    let mut actual_func_name = func_name;
                    if !args.is_empty() && *args[0].ty() == Type::String {
                        actual_func_name = "std.io.print_str".to_string();
                    }
                    return HirExpression::Call {
                        callee: Box::new(HirExpression::BuiltinFunction(actual_func_name, Type::Unit)),
                        arguments: args,
                        ty: Type::Unit,
                    };
                }

                if let Some(sig) = self.functions.get(&func_name) {
                    let expected_len = sig.params.len();
                    let return_ty = sig.return_type.clone();
                    let mut expected_types = Vec::new();
                    for p in &sig.params {
                        expected_types.push(p.clone());
                    }

                    if args.len() != expected_len {
                        self.report_error(format!("Function '{}' expects {} arguments, got {}", func_name, expected_len, args.len()));
                    } else {
                        for (i, arg) in args.iter().enumerate() {
                            if arg.ty() != &expected_types[i] && *arg.ty() != Type::Error {
                                self.report_error(format!("Argument {} of '{}' expected {:?}, got {:?}", i+1, func_name, expected_types[i], arg.ty()));
                            }
                        }
                    }
                    // Analyzer ignores checking 'callee' as a variable if it's a known function
                    let callee_expr = HirExpression::BuiltinFunction(func_name, return_ty.clone());
                    return HirExpression::Call { callee: Box::new(callee_expr), arguments: args, ty: return_ty };
                } else {
                    self.report_error(format!("Call to undefined function '{}'", func_name));
                    let callee_expr = self.analyze_expression(*callee);
                    return HirExpression::Call { callee: Box::new(callee_expr), arguments: args, ty: Type::Error };
                }
            }
            Expression::MemberAccess { object, member } => {
                let obj_expr = self.analyze_expression(*object.clone());
                
                if let Type::Struct(struct_name) = obj_expr.ty() {
                    if let Some(fields) = self.structs.get(struct_name) {
                        for field in fields {
                            if field.name == member {
                                return HirExpression::MemberAccess {
                                    object: Box::new(obj_expr),
                                    member,
                                    ty: field.ty.clone(),
                                };
                            }
                        }
                        self.report_error(format!("Struct '{}' has no field '{}'", struct_name, member));
                    }
                }
                
                // For V0.1 we only support std.io specifically inside Call resolution
                HirExpression::MemberAccess { object: Box::new(obj_expr), member, ty: Type::Error }
            }
            Expression::StructInstantiation { name, fields } => {
                if !self.structs.contains_key(&name) {
                    self.report_error(format!("Undefined struct: {}", name));
                    return HirExpression::StructInstantiation {
                        name,
                        fields: vec![],
                        ty: Type::Error,
                    };
                }
                let struct_def = self.structs.get(&name).unwrap().clone();
                let mut hir_fields = Vec::new();

                for (f_name, f_expr) in fields {
                    let mut found = false;
                    for def_field in &struct_def {
                        if def_field.name == f_name {
                            found = true;
                            let expr = self.analyze_expression(f_expr);
                            if *expr.ty() != def_field.ty && *expr.ty() != Type::Error {
                                self.report_error(format!("Type mismatch in struct '{}' field '{}'. Expected {:?}, got {:?}", name, f_name, def_field.ty, expr.ty()));
                            }
                            hir_fields.push((f_name.clone(), expr));
                            break;
                        }
                    }
                    if !found {
                        self.report_error(format!("Struct '{}' has no field named '{}'", name, f_name));
                    }
                }
                
                HirExpression::StructInstantiation {
                    name: name.clone(),
                    fields: hir_fields,
                    ty: Type::Struct(name),
                }
            }
            Expression::Borrow(expr) => {
                let hir_expr = self.analyze_expression(*expr);
                let ty = Type::Ref(Box::new(hir_expr.ty().clone()));
                HirExpression::Borrow(Box::new(hir_expr), ty)
            }
            Expression::MutBorrow(expr) => {
                let hir_expr = self.analyze_expression(*expr);
                let ty = Type::MutRef(Box::new(hir_expr.ty().clone()));
                HirExpression::MutBorrow(Box::new(hir_expr), ty)
            }
            Expression::Dereference(expr) => {
                let hir_expr = self.analyze_expression(*expr);
                let ty = match hir_expr.ty() {
                    Type::Ref(inner) | Type::MutRef(inner) => *inner.clone(),
                    Type::Error => Type::Error,
                    _ => {
                        self.report_error(format!("Cannot dereference non-pointer type {:?}", hir_expr.ty()));
                        Type::Error
                    }
                };
                HirExpression::Dereference(Box::new(hir_expr), ty)
            }
        }
    }
}
