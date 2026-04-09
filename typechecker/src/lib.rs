use std::{borrow::Cow, collections::HashMap, ops::Deref, rc::Rc};

use common::{
    environment::Environment,
    error::{NovaError, NovaResult},
    fileposition::FilePosition,
    nodes::{Expr, Statement},
    table::Table,
    ttype::{generate_unique_string, TType},
};

/// The TypeChecker owns the Environment and provides all type-checking
/// and type-resolution methods. The parser delegates type operations to
/// this struct while building the AST.
#[derive(Debug, Clone)]
pub struct TypeChecker {
    pub environment: Environment,
}

pub fn new() -> TypeChecker {
    TypeChecker {
        environment: Environment::new(),
    }
}

pub fn with_environment(env: Environment) -> TypeChecker {
    TypeChecker { environment: env }
}

impl TypeChecker {
    // ---------------------------------------------------------------
    // Error helpers
    // ---------------------------------------------------------------

    pub fn generate_error_with_pos(
        &self,
        msg: impl Into<Cow<'static, str>>,
        note: impl Into<Cow<'static, str>>,
        pos: FilePosition,
    ) -> Box<NovaError> {
        Box::new(NovaError::Parsing {
            msg: msg.into(),
            note: note.into(),
            position: pos,
            extra: None,
        })
    }

    // ---------------------------------------------------------------
    // Core type-checking: check_and_map_types
    // ---------------------------------------------------------------

    pub fn check_and_map_types(
        &self,
        type_list1: &[TType],
        type_list2: &[TType],
        type_map: &mut HashMap<Rc<str>, TType>,
        pos: FilePosition,
    ) -> NovaResult<()> {
        if type_list1.len() != type_list2.len() {
            return Err(self.generate_error_with_pos(
                format!("Incorrect number of arguments: expected {}, got {}", type_list1.len(), type_list2.len()),
                format!(
                    "Found {} argument(s), but expecting {} argument(s).\n  Check the function signature and make sure the number of arguments matches.",
                    type_list2.len(),
                    type_list1.len()
                ),
                pos,
            ));
        }
        for (t1, t2) in type_list1.iter().zip(type_list2.iter()) {
            match (t1, t2) {
                (TType::Any, b) if b != &TType::None => {
                    continue;
                }
                (a, TType::Any) if a != &TType::None => {
                    continue;
                }
                (
                    TType::Tuple {
                        elements: elements1,
                    },
                    TType::Tuple {
                        elements: elements2,
                    },
                ) => {
                    self.check_and_map_types(elements1, elements2, type_map, pos.clone())?;
                }
                (TType::Generic { name: name1 }, _) => {
                    if t2 == &TType::None {
                        return Err(Box::new(NovaError::TypeMismatch {
                            expected: t1.clone(),
                            found: t2.clone(),
                            position: pos.clone(),
                        }));
                    }
                    if t2 == &TType::Void {
                        return Err(Box::new(NovaError::TypeMismatch {
                            expected: t1.clone(),
                            found: t2.clone(),
                            position: pos.clone(),
                        }));
                    }
                    let typemap_clone: HashMap<Rc<str>, TType> = type_map.clone();
                    if let Some(mapped_type) = typemap_clone.get(name1) {
                        if let (TType::Dyn { own, contract }, Some(name)) =
                            (mapped_type, t2.custom_to_string())
                        {
                            let name_rc = Rc::from(name);
                            self.check_contracts(type_map, &pos, t1, t2, own, contract, &name_rc)?;
                        } else if mapped_type != t2 {
                            return Err(Box::new(NovaError::TypeMismatch {
                                expected: mapped_type.clone(),
                                found: t2.clone(),
                                position: pos.clone(),
                            }));
                        }
                    } else {
                        type_map.insert(name1.clone(), t2.clone());
                    }
                }

                (TType::List { inner: inner1 }, TType::List { inner: inner2 }) => {
                    self.check_and_map_types(
                        &[*inner1.clone()],
                        &[*inner2.clone()],
                        type_map,
                        pos.clone(),
                    )?;
                }
                (TType::Option { inner: inner1 }, TType::Option { inner: inner2 }) => {
                    self.check_and_map_types(
                        &[*inner1.clone()],
                        &[*inner2.clone()],
                        type_map,
                        pos.clone(),
                    )?;
                }
                (
                    TType::Function {
                        parameters: params1,
                        return_type: ret1,
                    },
                    TType::Function {
                        parameters: params2,
                        return_type: ret2,
                    },
                ) => {
                    if params1.len() != params2.len() {
                        return Err(Box::new(NovaError::TypeMismatch {
                            expected: t1.clone(),
                            found: t2.clone(),
                            position: pos.clone(),
                        }));
                    }

                    self.check_and_map_types(params1, params2, type_map, pos.clone())?;
                    self.check_and_map_types(
                        &[*ret1.clone()],
                        &[*ret2.clone()],
                        type_map,
                        pos.clone(),
                    )?;
                }
                (
                    TType::Custom {
                        name: custom1,
                        type_params: gen1,
                    },
                    TType::Custom {
                        name: custom2,
                        type_params: gen2,
                    },
                ) => {
                    if custom1 == custom2 {
                        self.check_and_map_types(gen1, gen2, type_map, pos.clone())?;
                    } else {
                        return Err(Box::new(NovaError::TypeMismatch {
                            expected: t1.clone(),
                            found: t2.clone(),
                            position: pos.clone(),
                        }));
                    }
                }
                (TType::Dyn { own, contract: c1 }, TType::Custom { name, .. }) => {
                    self.check_contracts(type_map, &pos, t1, t2, own, c1, name)?;
                }
                (TType::Dyn { own, .. }, TType::Generic { name }) => {
                    if name == own {
                        continue;
                    } else {
                        return Err(Box::new(NovaError::TypeMismatch {
                            expected: TType::Generic { name: own.clone() },
                            found: TType::Generic { name: name.clone() },
                            position: pos.clone(),
                        }));
                    }
                }
                _ if t1 == t2 => continue,
                _ => {
                    return Err(Box::new(NovaError::TypeMismatch {
                        expected: t1.clone(),
                        found: t2.clone(),
                        position: pos.clone(),
                    }));
                }
            }
        }
        Ok(())
    }

    // ---------------------------------------------------------------
    // Contract checking
    // ---------------------------------------------------------------

    #[allow(clippy::too_many_arguments)]
    pub fn check_contracts(
        &self,
        type_map: &mut HashMap<Rc<str>, TType>,
        pos: &FilePosition,
        t1: &TType,
        t2: &TType,
        own: &Rc<str>,
        c1: &[(Rc<str>, TType)],
        name: &Rc<str>,
    ) -> NovaResult<()> {
        let contract_names = c1.iter().map(|(n, _)| n).collect::<Vec<_>>();
        // Use a prefixed key for the Dyn's own type variable so it never
        // collides with function-level generics (e.g. both named "T").
        let dyn_key: Rc<str> = format!("__dyn_{}", own).into();
        if let Some(fields) = self.environment.custom_types.get(name.as_ref()) {
            if let Some(mapped_type) = type_map.get(&dyn_key) {
                if mapped_type != t2 {
                    return Err(Box::new(NovaError::TypeMismatch {
                        expected: mapped_type.clone(),
                        found: t2.clone(),
                        position: pos.clone(),
                    }));
                }
            } else {
                type_map.insert(dyn_key.clone(), t2.clone());
            }

            if contract_names.len() > fields.len() {
                return Err(Box::new(NovaError::TypeMismatch {
                    expected: t1.clone(),
                    found: t2.clone(),
                    position: pos.clone(),
                }));
            }

            let new_fields = if let Some(generic_params) = self
                .environment
                .generic_type_struct
                .get(t2.custom_to_string().unwrap())
            {
                let TType::Custom { type_params, .. } = t2 else {
                    return Err(self.generate_error_with_pos(
                        "Expected custom type",
                        format!("got {}", t2),
                        pos.clone(),
                    ));
                };
                fields
                    .iter()
                    .map(|(name, ttype)| {
                        let new_ttype =
                            Self::replace_generic_types(ttype, generic_params, type_params);
                        (name.clone(), new_ttype)
                    })
                    .collect()
            } else {
                fields.clone()
            };

            for contract_name in contract_names.iter() {
                if !new_fields
                    .iter()
                    .any(|(field_name, _)| field_name == *contract_name)
                {
                    return Err(Box::new(NovaError::TypeMismatch {
                        expected: t1.clone(),
                        found: t2.clone(),
                        position: pos.clone(),
                    }));
                }

                let field_type = c1
                    .iter()
                    .find(|(n, _)| n == *contract_name)
                    .map(|(_, t)| t)
                    .unwrap();

                let new_field = new_fields
                    .iter()
                    .find(|(n, _)| n == *contract_name)
                    .map(|(_, t)| t)
                    .unwrap();

                // Replace the Dyn's own type variable (e.g. Generic("T")) with the
                // prefixed key (e.g. Generic("__dyn_T")) so it resolves against the
                // dyn-scoped mapping instead of any outer function-level generic
                // with the same name.  This prevents infinite recursion when, e.g.,
                // List::push's generic T is mapped to a Dyn(T = draw: fn($T)).
                let scoped_field_type = Self::replace_generic_types(
                    field_type,
                    &[own.clone()],
                    &[TType::Generic {
                        name: dyn_key.clone(),
                    }],
                );

                self.check_and_map_types(
                    std::slice::from_ref(&scoped_field_type),
                    std::slice::from_ref(new_field),
                    type_map,
                    pos.clone(),
                )?;
            }
        } else {
            return Err(Box::new(NovaError::TypeMismatch {
                expected: t1.clone(),
                found: t2.clone(),
                position: pos.clone(),
            }));
        };
        Ok(())
    }

    // ---------------------------------------------------------------
    // Generic output resolution
    // ---------------------------------------------------------------

    pub fn get_output(
        &self,
        output: TType,
        type_map: &mut HashMap<Rc<str>, TType>,
        pos: FilePosition,
    ) -> NovaResult<TType> {
        match output.clone() {
            TType::Tuple { elements } => {
                let mut mapped_elements = Vec::new();
                for element in elements {
                    let mapped_element = self.get_output(element, type_map, pos.clone())?;
                    mapped_elements.push(mapped_element);
                }
                Ok(TType::Tuple {
                    elements: mapped_elements,
                })
            }
            TType::Generic { name } => {
                if let Some(mapped_type) = type_map.get(&name) {
                    Ok(mapped_type.clone())
                } else if self.environment.live_generics.last().unwrap().has(&name) {
                    Ok(TType::Generic { name })
                } else {
                    Err(Box::new(NovaError::SimpleTypeError {
                        msg: format!("Generic type `{}` could not be inferred.\n  Provide explicit type annotations to help the type checker.", name).into(),
                        position: pos,
                    }))
                }
            }
            TType::List { inner } => {
                let mapped_inner = self.get_output(*inner.clone(), type_map, pos)?;
                Ok(TType::List {
                    inner: Box::new(mapped_inner),
                })
            }
            TType::Option { inner } => {
                let mapped_inner = self.get_output(*inner.clone(), type_map, pos)?;
                Ok(TType::Option {
                    inner: Box::new(mapped_inner),
                })
            }
            TType::Function {
                parameters: args,
                return_type,
            } => {
                let mut mapped_args = Vec::new();
                for arg in args {
                    let mapped_arg = self.get_output(arg, type_map, pos.clone())?;
                    mapped_args.push(mapped_arg);
                }

                let mapped_return_type = self.get_output(*return_type.clone(), type_map, pos)?;

                Ok(TType::Function {
                    parameters: mapped_args,
                    return_type: Box::new(mapped_return_type),
                })
            }
            TType::Custom { name, type_params } => {
                let mut mapped_type_params = Vec::new();
                for param in type_params {
                    let mapped_param = self.get_output(param, type_map, pos.clone())?;
                    mapped_type_params.push(mapped_param);
                }

                Ok(TType::Custom {
                    name,
                    type_params: mapped_type_params,
                })
            }
            _ => Ok(output.clone()),
        }
    }

    // ---------------------------------------------------------------
    // Generic type utilities (static)
    // ---------------------------------------------------------------

    pub fn replace_generic_types(
        ttype: &TType,
        x: &[impl AsRef<str>],
        type_params: &[TType],
    ) -> TType {
        match ttype {
            TType::Generic { name: n } => {
                if let Some(index) = x.iter().position(|x| x.as_ref() == n.deref()) {
                    type_params[index].clone()
                } else {
                    ttype.clone()
                }
            }
            TType::None
            | TType::Any
            | TType::Int
            | TType::Float
            | TType::Bool
            | TType::String
            | TType::Char
            | TType::Void
            | TType::Auto => ttype.clone(),
            TType::Custom {
                name,
                type_params: inner_params,
            } => {
                let new_params = inner_params
                    .iter()
                    .map(|param| Self::replace_generic_types(param, x, type_params))
                    .collect();
                TType::Custom {
                    name: name.clone(),
                    type_params: new_params,
                }
            }
            TType::List { inner } => TType::List {
                inner: Box::new(Self::replace_generic_types(inner, x, type_params)),
            },
            TType::Function {
                parameters,
                return_type,
            } => {
                let new_params = parameters
                    .iter()
                    .map(|param| Self::replace_generic_types(param, x, type_params))
                    .collect();
                TType::Function {
                    parameters: new_params,
                    return_type: Box::new(Self::replace_generic_types(return_type, x, type_params)),
                }
            }
            TType::Option { inner } => TType::Option {
                inner: Box::new(Self::replace_generic_types(inner, x, type_params)),
            },
            TType::Tuple { elements } => {
                let new_elements = elements
                    .iter()
                    .map(|element| Self::replace_generic_types(element, x, type_params))
                    .collect();
                TType::Tuple {
                    elements: new_elements,
                }
            }
            TType::Dyn { own, contract } => {
                // The Dyn's `own` parameter is a scoped type variable for the Dyn itself
                // (used for self-referential types like fn($T) -> String in the contract).
                // It must NOT be replaced by external generic substitution.
                // However, contract field types may contain external generics that do need replacing.
                let new_contract = contract
                    .iter()
                    .map(|(name, field_type)| {
                        (
                            name.clone(),
                            Self::replace_generic_types(field_type, x, type_params),
                        )
                    })
                    .collect();
                TType::Dyn {
                    own: own.clone(),
                    contract: new_contract,
                }
            }
        }
    }

    pub fn is_generic(params: &[TType]) -> bool {
        for param in params {
            match param {
                TType::Generic { .. } => return true,
                TType::Function {
                    parameters,
                    return_type,
                } => {
                    if Self::is_generic(parameters) || Self::is_generic(&[*return_type.clone()]) {
                        return true;
                    }
                }
                TType::List { inner } => {
                    if Self::is_generic(&[*inner.clone()]) {
                        return true;
                    }
                }
                TType::Option { inner } => {
                    if Self::is_generic(&[*inner.clone()]) {
                        return true;
                    }
                }
                TType::Custom { type_params, .. } => {
                    if Self::is_generic(type_params) {
                        return true;
                    }
                }
                TType::Tuple { elements } => {
                    if Self::is_generic(elements) {
                        return true;
                    }
                }
                TType::Dyn { .. } => {
                    return true;
                }
                _ => {}
            }
        }
        false
    }

    pub fn collect_generics(input: &[TType]) -> Table<Rc<str>> {
        let mut contracts = Table::new();
        for t in input {
            match t {
                TType::Generic { name: generic } => contracts.insert(generic.clone()),
                TType::Function {
                    parameters: input,
                    return_type: output,
                } => {
                    contracts.extend(Self::collect_generics(input));
                    contracts.extend(Self::collect_generics(&[*output.clone()]))
                }
                TType::List { inner: list } => {
                    contracts.extend(Self::collect_generics(&[*list.clone()]))
                }
                TType::Option { inner: option } => {
                    contracts.extend(Self::collect_generics(&[*option.clone()]))
                }
                TType::Custom { type_params, .. } => {
                    contracts.extend(Self::collect_generics(&type_params.clone()))
                }
                TType::Tuple { elements } => {
                    contracts.extend(Self::collect_generics(&elements.clone()))
                }
                _ => {}
            }
        }
        contracts
    }

    // ---------------------------------------------------------------
    // map_generic_types
    // ---------------------------------------------------------------

    pub fn map_generic_types(
        &mut self,
        parameters: &[TType],
        argument_types: &[TType],
        type_map: &mut HashMap<Rc<str>, TType>,
        pos: FilePosition,
    ) -> NovaResult<()> {
        for (param_type, arg_type) in parameters.iter().zip(argument_types.iter()) {
            if let (
                TType::Custom {
                    name: param_name, ..
                },
                TType::Custom { name: arg_name, .. },
            ) = (param_type, arg_type)
            {
                if let Some(internal_type) =
                    self.environment.generic_type_map.get(arg_name.as_ref())
                {
                    if internal_type == param_name {
                        if let Some(param_list) = self.environment.get_type(param_name) {
                            let s = self.clone();
                            if let Some(arg_list) = self.environment.get_type(arg_name) {
                                // Use the cloned checker for check_and_map_types
                                // to avoid borrow issues
                                s.check_and_map_types(
                                    &[param_list],
                                    &[arg_list],
                                    type_map,
                                    pos.clone(),
                                )?;
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    // ---------------------------------------------------------------
    // Varargs resolution
    // ---------------------------------------------------------------

    pub fn varargs(
        &mut self,
        identifier: &str,
        argument_types: &mut Vec<TType>,
        arguments: &mut Vec<Expr>,
    ) {
        let mut type_flag: TType = TType::Any;
        let mut has_varargs = false;
        let mut element = 0;

        if self
            .environment
            .get_function_type(identifier, argument_types)
            .is_none()
        {
            for i in 0..=argument_types.len() {
                let (left, right) = argument_types.split_at(argument_types.len() - i);
                if let Some(first) = right.first() {
                    type_flag = first.clone();
                    let mut check = true;
                    for ttype in right.iter() {
                        if ttype != first {
                            check = false;
                            break;
                        }
                    }
                    if check {
                        let mut new_right = left.to_vec();
                        new_right.push(TType::List {
                            inner: Box::new(first.clone()),
                        });
                        if self
                            .environment
                            .get(&generate_unique_string(identifier, &new_right))
                            .is_some()
                        {
                            *argument_types = new_right;
                            element = i;
                            has_varargs = true;
                            break;
                        }
                    }
                }
            }
        }

        if has_varargs {
            arguments.reverse();
            let (leftexpr, rightexpr) = arguments.split_at(element);
            let mut leftexpr = leftexpr.to_vec();
            let mut rightexpr = rightexpr.to_vec();
            *arguments = vec![];
            rightexpr.reverse();
            arguments.append(&mut rightexpr);
            leftexpr.reverse();
            arguments.push(Expr::ListConstructor {
                ttype: type_flag.clone(),
                elements: leftexpr.clone(),
            });
        }
    }

    // ---------------------------------------------------------------
    // Return validation (will_return)
    // ---------------------------------------------------------------

    pub fn will_return(
        &self,
        statements: &[Statement],
        return_type: TType,
        pos: FilePosition,
    ) -> NovaResult<bool> {
        for statement in statements.iter() {
            match statement {
                Statement::Return { ttype, .. } => {
                    match self.check_and_map_types(
                        std::slice::from_ref(ttype),
                        std::slice::from_ref(&return_type),
                        &mut HashMap::default(),
                        pos.clone(),
                    ) {
                        Ok(_) => {}
                        _ => {
                            return Err(self.generate_error_with_pos(
                                format!("Cannot return `{}` from function expecting `{}`", ttype, return_type),
                                format!("The function's return type is `{}` but `return` expression has type `{}`.", return_type, ttype),
                                pos.clone(),
                            ));
                        }
                    }
                    return Ok(true);
                }
                Statement::If {
                    body, alternative, ..
                } => {
                    let body_return = self.will_return(body, return_type.clone(), pos.clone())?;
                    if let Some(alt) = alternative {
                        let alt_return = self.will_return(alt, return_type.clone(), pos.clone())?;
                        if body_return && alt_return {
                            return Ok(true);
                        }
                    }
                }
                Statement::Expression { expr, .. } => {
                    if let Expr::Return { expr, ttype: _ } = expr {
                        match self.check_and_map_types(
                            &[expr.get_type()],
                            std::slice::from_ref(&return_type),
                            &mut HashMap::default(),
                            pos.clone(),
                        ) {
                            Ok(_) => {}
                            _ => {
                                return Err(self.generate_error_with_pos(
                                    format!("Cannot return `{}` from function expecting `{}`", expr.get_type(), return_type),
                                    format!("The function's return type is `{}` but `return` expression has type `{}`.", return_type, expr.get_type()),
                                    pos.clone(),
                                ));
                            }
                        }
                        return Ok(true);
                    }
                }
                Statement::Pass => {
                    return Ok(true);
                }
                Statement::IfLet {
                    expr,
                    body,
                    alternative,
                    ..
                } => {
                    let body_return = self.will_return(body, return_type.clone(), pos.clone())?;
                    if let Some(alt) = alternative {
                        let alt_return = self.will_return(alt, return_type.clone(), pos.clone())?;
                        if body_return && alt_return {
                            return Ok(true);
                        }
                    }
                    self.will_return(
                        &[Statement::Expression {
                            ttype: expr.get_type(),
                            expr: expr.clone(),
                        }],
                        return_type.clone(),
                        pos.clone(),
                    )?;
                }
                Statement::While { test, body } => {
                    self.will_return(
                        &[Statement::Expression {
                            ttype: test.get_type(),
                            expr: test.clone(),
                        }],
                        return_type.clone(),
                        pos.clone(),
                    )?;
                    self.will_return(body, return_type.clone(), pos.clone())?;
                }
                Statement::For {
                    init,
                    test,
                    inc,
                    body,
                } => {
                    self.will_return(
                        &[Statement::Expression {
                            ttype: init.get_type(),
                            expr: init.clone(),
                        }],
                        return_type.clone(),
                        pos.clone(),
                    )?;
                    self.will_return(
                        &[Statement::Expression {
                            ttype: test.get_type(),
                            expr: test.clone(),
                        }],
                        return_type.clone(),
                        pos.clone(),
                    )?;
                    self.will_return(
                        &[Statement::Expression {
                            ttype: inc.get_type(),
                            expr: inc.clone(),
                        }],
                        return_type.clone(),
                        pos.clone(),
                    )?;
                    self.will_return(body, return_type.clone(), pos.clone())?;
                }
                Statement::Foreach { expr, body, .. } => {
                    self.will_return(
                        &[Statement::Expression {
                            ttype: expr.get_type(),
                            expr: expr.clone(),
                        }],
                        return_type.clone(),
                        pos.clone(),
                    )?;
                    self.will_return(body, return_type.clone(), pos.clone())?;
                }
                Statement::ForRange {
                    start,
                    end,
                    step,
                    body,
                    ..
                } => {
                    self.will_return(
                        &[Statement::Expression {
                            ttype: start.get_type(),
                            expr: start.clone(),
                        }],
                        return_type.clone(),
                        pos.clone(),
                    )?;
                    self.will_return(
                        &[Statement::Expression {
                            ttype: end.get_type(),
                            expr: end.clone(),
                        }],
                        return_type.clone(),
                        pos.clone(),
                    )?;
                    if let Some(step) = step {
                        self.will_return(
                            &[Statement::Expression {
                                ttype: step.get_type(),
                                expr: step.clone(),
                            }],
                            return_type.clone(),
                            pos.clone(),
                        )?;
                    }
                    self.will_return(body, return_type.clone(), pos.clone())?;
                }
                Statement::Block { body, .. } => {
                    self.will_return(body, return_type.clone(), pos.clone())?;
                }
                Statement::Match {
                    expr,
                    arms,
                    default,
                    ..
                } => {
                    self.will_return(
                        &[Statement::Expression {
                            ttype: expr.get_type(),
                            expr: expr.clone(),
                        }],
                        return_type.clone(),
                        pos.clone(),
                    )?;
                    let mut arms_return = vec![];
                    for arm in arms.iter() {
                        for statement in arm.2.iter() {
                            arms_return.push(self.will_return(
                                std::slice::from_ref(statement),
                                return_type.clone(),
                                pos.clone(),
                            )?);
                        }
                    }

                    if let Some(default) = default {
                        for statement in default.iter() {
                            arms_return.push(self.will_return(
                                std::slice::from_ref(statement),
                                return_type.clone(),
                                pos.clone(),
                            )?);
                        }
                    }

                    if arms_return.iter().all(|x| *x) {
                        return Ok(true);
                    }
                }
                Statement::Enum { .. } => {}
                Statement::Struct { .. } => {}
                Statement::Function { .. } => {}
                Statement::ForwardDec { .. } => {}
                Statement::Continue => {}
                Statement::Break => {}
                Statement::Unwrap { .. } => {}
                Statement::WhileLet { expr, body, .. } => {
                    self.will_return(
                        &[Statement::Expression {
                            ttype: expr.get_type(),
                            expr: expr.clone(),
                        }],
                        return_type.clone(),
                        pos.clone(),
                    )?;
                    self.will_return(body, return_type.clone(), pos.clone())?;
                }
            }
        }

        Ok(false)
    }

    // ---------------------------------------------------------------
    // Type error creation helper
    // ---------------------------------------------------------------

    pub fn create_type_error(
        &self,
        left_expr: Expr,
        right_expr: Expr,
        operation: common::tokens::Operator,
        pos: FilePosition,
    ) -> Box<NovaError> {
        Box::new(NovaError::TypeError {
            expected: left_expr.get_type().to_string().into(),
            found: right_expr.get_type().to_string().into(),
            position: pos,
            msg: format!(
                "Cannot apply `{operation:?}` to `{}` and `{}`. Both operands must have compatible types, or define a dunder method (e.g. `__add__`, `__mul__`) for these types.",
                left_expr.get_type(),
                right_expr.get_type(),
            )
            .into(),
        })
    }
}
