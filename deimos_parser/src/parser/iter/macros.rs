#[macro_export]
macro_rules! next_guard {
    ($iter:ident ( $loc_bind:ident ) { $( $( $p:pat )* => $b:expr ),* $(,)? } ) => {
        match $iter.next() {
            $($(
                Some(deimos_ast::Located {
                    data: $p,
                    loc: $loc_bind,
                }) => $b,
            )*)*
            Some(t) => return Err(crate::parser::ParseError::UnexpectedToken(t)),
            None => return Err($iter.eof_err()),
        }
    };
    ($iter:ident { $( $token:tt )* }) => {
        next_guard!($iter (_e) { $( $token )* })
    };
}
