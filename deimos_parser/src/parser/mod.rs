use crate::next_guard;

use super::lexer::*;
use deimos_ast::*;

mod error;
mod expr;
mod iter;
pub use error::*;

use self::iter::TokenIter;

pub fn parse(Tokens { lexemes, bank }: Tokens) -> ParseResult<Program> {
    let mut tokens = TokenIter::new(&lexemes);
    let mut definitions = Definitions::new();
    let mut body = None;

    while let Some(token) = tokens.next() {
        let def = match token.data {
            Lexeme::Keyword(Keyword::Fn) => {
                let name = tokens.expect_ident()?;
                let args = parse_fn_params(&mut tokens)?;
                let block = parse_fn_body(&mut tokens)?;
                Definition::Function(Function { name, args, block })
            }
            Lexeme::Keyword(Keyword::Record) => {
                return Err(ParseError::ReservedWord(Keyword::Record))
            }
            Lexeme::Keyword(Keyword::Mem) => {
                let addr = tokens.expect_group(Grouper::Parenthesis, |t| t.expect_int())?;
                let var = parse_typed_ident(&mut tokens)?;
                tokens.expect_semicolon()?;
                Definition::MemVar(MemVar { addr, var })
            }
            Lexeme::Keyword(Keyword::Static) => Definition::Static(parse_fn_varinit(&mut tokens)?),
            Lexeme::Keyword(Keyword::Program) if body.is_some() => {
                return Err(ParseError::BodyRedefinition(token.loc));
            }
            Lexeme::Keyword(Keyword::Program) => {
                body = Some(parse_fn_body(&mut tokens)?);
                continue;
            }
            _ => return Err(ParseError::UnexpectedToken(token)),
        };

        let name = def.name();
        if definitions.insert(name, def).is_some() {
            return Err(ParseError::InvalidRedefinition(Located::new(
                name, token.loc,
            )));
        }
    }

    body.map(|body| Program {
        bank,
        definitions,
        body,
    })
    .ok_or(ParseError::NoBody)
}

fn parse_decl_type(tokens: &mut TokenIter) -> ParseResult<DeclType> {
    let base = expr::parse_param_type(tokens)?;
    if tokens
        .next_if_eq(Lexeme::GroupBegin(Grouper::Bracket))
        .is_some()
    {
        let array_size = tokens.expect_int()?;
        tokens.expect_end(Grouper::Bracket)?;
        Ok(DeclType::Array {
            array_type: base,
            size: array_size,
        })
    } else {
        Ok(DeclType::Param(base))
    }
}

fn parse_fn_varinit(tokens: &mut TokenIter) -> ParseResult<VarDecl> {
    let name = tokens.expect_ident()?;
    let mut init_val = None;
    tokens.expect_colon()?;
    let var_type = parse_decl_type(tokens)?;
    if tokens.next_if_eq(Lexeme::Equals).is_some() {
        init_val = Some(parse_initval(tokens)?);
    }
    Ok(VarDecl {
        variable: var_type,
        name,
        init: init_val,
    })
}

fn parse_initval(tokens: &mut TokenIter) -> ParseResult<InitValue> {
    Ok(next_guard!(tokens (_loc) {
        Lexeme::Integer(i) => InitValue::Primitive(PrimitiveValue::Int(Located::new(i, _loc))),
        Lexeme::Float(f) => InitValue::Primitive(PrimitiveValue::Float(Located::new(f, _loc))),
        Lexeme::String(s) => InitValue::Primitive(PrimitiveValue::String(Located::new(s, _loc))),
        Lexeme::Unsigned(u) => InitValue::Primitive(PrimitiveValue::Unsigned(Located::new(u, _loc))),
        Lexeme::GroupBegin(Grouper::Bracket) => {
            let mut vals = Vec::new();
            loop {
                if tokens.next_if_eq(Lexeme::GroupEnd(Grouper::Bracket)).is_some() {
                    break;
                }
                vals.push(parse_initval(tokens)?);
                next_guard!(tokens {
                    Lexeme::Comma => {},
                    Lexeme::GroupEnd(Grouper::Bracket) => break,
                });
            }
            InitValue::List(vals)
        }
    }))
}

fn parse_typed_ident(tokens: &mut TokenIter) -> ParseResult<TypedIdent> {
    let name = tokens.expect_ident()?;
    tokens.expect_colon()?;
    let field_type = expr::parse_param_type(tokens)?;
    Ok(TypedIdent { name, field_type })
}

fn parse_fn_params(tokens: &mut TokenIter) -> ParseResult<FunctionArgs> {
    let mut args = Vec::new();
    tokens.expect_begin(Grouper::Parenthesis)?;
    loop {
        next_guard!(tokens(_loc) {
            Lexeme::GroupEnd(Grouper::Parenthesis) => break,
            Lexeme::Identifier(ident) => {
                tokens.expect_colon()?;
                let field_type = expr::parse_param_type(tokens)?;
                args.push(TypedIdent {
                    name: Located::new(ident, _loc),
                    field_type
                });
                next_guard!(tokens {
                    Lexeme::Comma => {},
                    Lexeme::GroupEnd(Grouper::Parenthesis) => break,
                })
            },
        });
    }
    Ok(args)
}

fn parse_regmap(tokens: &mut TokenIter) -> ParseResult<RegisterMap> {
    tokens.expect_begin(Grouper::Parenthesis)?;
    let mut map = RegisterMap::new();
    loop {
        next_guard!(tokens(_loc) {
            Lexeme::GroupEnd(Grouper::Parenthesis) => break,
            Lexeme::Register(reg) => {
                tokens.expect_colon()?;
                let mapped_ident = tokens.expect_ident()?;
                if map.insert(reg, mapped_ident).is_some() {
                    return Err(ParseError::DuplicateRegister(Located::new(reg, _loc)));
                }
                next_guard!(tokens {
                    Lexeme::Comma => {},
                    Lexeme::GroupEnd(Grouper::Parenthesis) => break,
                })
            },
        });
    }
    Ok(map)
}

fn parse_inout_regmaps(tokens: &mut TokenIter) -> ParseResult<RegVars> {
    let mut vars = RegVars::default();
    if tokens.next_if_key(Keyword::In).is_some() {
        tokens.expect_colon()?;
        vars.in_values = parse_regmap(tokens)?;
        tokens.expect_semicolon()?;
    }
    if tokens.next_if_key(Keyword::Out).is_some() {
        tokens.expect_colon()?;
        vars.out_values = parse_regmap(tokens)?;
        tokens.expect_semicolon()?;
    }
    Ok(vars)
}

fn parse_syscall(tokens: &mut TokenIter) -> ParseResult<Syscall> {
    let syscall_id = tokens.expect_group(Grouper::Parenthesis, |t| t.expect_int())?;

    let map = next_guard!(tokens {
        Lexeme::GroupBegin(Grouper::Brace) => {
            let v = parse_inout_regmaps(tokens)?;
            tokens.expect_end(Grouper::Brace)?;
            v
        },
        Lexeme::Semicolon => RegVars::default(),
    });

    Ok(Syscall { syscall_id, map })
}

fn parse_asm(tokens: &mut TokenIter) -> ParseResult<AsmBlock> {
    tokens.expect_group(Grouper::Brace, |tokens| {
        let mut asm_strings = vec![tokens.expect_string()?];
        loop {
            next_guard!(tokens(_loc) {
                Lexeme::String(s) => asm_strings.push(Located::new(s, _loc)),
                Lexeme::Semicolon => break,
            });
        }
        let map = parse_inout_regmaps(tokens)?;
        Ok(AsmBlock { asm_strings, map })
    })
}

fn parse_assignment(tokens: &mut TokenIter) -> ParseResult<Assignment> {
    let rtokens = tokens.until_level(|t| *t == Lexeme::Equals || *t == Lexeme::Semicolon)?;
    if let Some(Located {
        data: Lexeme::Semicolon,
        loc,
    }) = rtokens.get_end()
    {
        return Err(ParseError::NakedExpression(*loc));
    }

    let rvalue = expr::parse_rvalue(rtokens)?;
    let lvalue = tokens
        .until_level_eq(Lexeme::Semicolon)
        .and_then(expr::parse_expression)?;

    Ok(Assignment { rvalue, lvalue })
}

fn parse_print(tokens: &mut TokenIter) -> ParseResult<Print> {
    let mut args = Vec::new();
    loop {
        let tokens = tokens.until_level(|t| *t == Lexeme::Comma || *t == Lexeme::Semicolon)?;
        let end = tokens.get_end().cloned();
        let expr = expr::parse_expression(tokens)?;
        args.push(expr);
        match end {
            Some(Located {
                data: Lexeme::Comma,
                ..
            }) => {}
            Some(Located {
                data: Lexeme::Semicolon,
                ..
            }) => break,
            _ => unreachable!(),
        }
    }
    Ok(Print { args })
}

fn parse_block_until_end(tokens: &mut TokenIter) -> ParseResult<Block> {
    let mut block = Block::new();
    loop {
        let token = tokens.next().ok_or(tokens.eof_err())?;
        let stmt = match token.data {
            Lexeme::GroupEnd(Grouper::Brace) => break,
            Lexeme::Keyword(Keyword::Call) => {
                let function = tokens.expect_ident()?;
                tokens.expect_next_eq(Lexeme::GroupBegin(Grouper::Parenthesis))?;
                let args = tokens
                    .level_split_comma(Grouper::Parenthesis)?
                    .into_iter()
                    .map(expr::parse_expression)
                    .collect::<ParseResult<Vec<Expression>>>()?;
                tokens.expect_semicolon()?;
                Statement::Call(Invocation { function, args })
            }
            Lexeme::Keyword(Keyword::Syscall) => Statement::Syscall(parse_syscall(tokens)?),
            Lexeme::Keyword(Keyword::If) => {
                let if_block = parse_condition_body(tokens)?;
                let mut elifs = Vec::new();
                let mut else_block = None;
                while tokens.next_if_key(Keyword::Elif).is_some() {
                    elifs.push(parse_condition_body(tokens)?);
                }
                if tokens.next_if_key(Keyword::Else).is_some() {
                    else_block = Some(parse_block(tokens)?);
                }
                Statement::LogicChain(LogicChain {
                    if_block,
                    elifs,
                    else_block,
                })
            }
            Lexeme::Keyword(Keyword::While) => Statement::While(parse_condition_body(tokens)?),
            Lexeme::Keyword(k @ Keyword::Break | k @ Keyword::Continue | k @ Keyword::Return) => {
                tokens.expect_semicolon()?;
                Statement::ControlBreak(match k {
                    Keyword::Break => ControlBreak::Break,
                    Keyword::Return => ControlBreak::Return,
                    Keyword::Continue => ControlBreak::Continue,
                    _ => unreachable!(),
                })
            }
            Lexeme::Keyword(Keyword::Asm) => Statement::Asm(parse_asm(tokens)?),
            Lexeme::Keyword(Keyword::Print) => Statement::Print(parse_print(tokens)?),
            _ => {
                tokens.prev();
                Statement::Assignment(parse_assignment(tokens)?)
            }
        };
        block.push(Located::new(stmt, token.loc));
    }
    Ok(block)
}

fn parse_condition_body(tokens: &mut TokenIter) -> ParseResult<ConditionBody> {
    tokens.expect_begin(Grouper::Parenthesis)?;
    let condition = tokens
        .take_group(Grouper::Parenthesis)
        .and_then(expr::parse_expression)?;
    let body = parse_block(tokens)?;
    Ok(ConditionBody { condition, body })
}

fn parse_block(tokens: &mut TokenIter) -> ParseResult<Block> {
    tokens.expect_begin(Grouper::Brace)?;
    parse_block_until_end(tokens)
}

fn parse_fn_body(tokens: &mut TokenIter) -> ParseResult<FunctionBlock> {
    tokens.expect_begin(Grouper::Brace)?;

    let mut vars = Vec::new();
    if tokens.next_if_key(Keyword::Let).is_some() {
        loop {
            let decl = parse_fn_varinit(tokens)?;
            vars.push(decl);
            next_guard!(tokens {
                Lexeme::Semicolon => break,
                Lexeme::Comma => {}
            });
        }
    }

    let block = parse_block_until_end(tokens)?;
    Ok(FunctionBlock { vars, block })
}
