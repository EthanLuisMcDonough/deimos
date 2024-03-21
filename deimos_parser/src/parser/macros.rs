#[macro_export]
macro_rules! next_guard {
    ({ $next:expr } ( $loc_bind:ident ) { $( $( $p:pat )* => $b:expr ),* $(,)? } ) => {
        match $next.into() {
            $($(
                Some(deimos_ast::Located {
                    data: $p,
                    loc: $loc_bind,
                }) => $b,
            )*)*
            Some(t) => return Err(crate::parser::ParseError::UnexpectedToken(t)),
            None => return Err(crate::parser::ParseError::UnexpectedEOF),
        }
    };
    ({ $next:expr } { $( $token:tt )* }) => {
        next_guard!({ $next } (_e) { $( $token )* })
    };
}

#[macro_export]
macro_rules! next_expect {
    ({ $next:expr } ( $loc_bind:ident ) { $p:pat }) => {
        match $next.into() {
            Some(deimos_ast::Located {
                data: $p,
                loc: $loc_bind,
            }) => {}
            Some(t) => return Err(crate::parser::ParseError::UnexpectedToken(t)),
            None => return Err(crate::parser::ParseError::UnexpectedEOF),
        }
    };
    ({ $next:expr } { $( $token:tt )* }) => {
        next_expect!({ $next } (_e) { $( $token )* })
    };
}

#[macro_export]
macro_rules! expect_semicolon {
    ($next:expr) => {
        next_expect!({ $next } { crate::lexer::Lexeme::Semicolon })
    };
}
