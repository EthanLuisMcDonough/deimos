#[macro_export]
macro_rules! keyword_map {
    ($name:ident { $( $field:ident -> $s:expr ),* $(,)* }) => {
        #[derive(Clone, Debug, PartialEq, Copy)]
        pub enum $name {
            $( $field ),*
        }

        impl $name {
            pub fn str(&self) -> &'static str {
                use self::$name::*;
                match self {
                    $( $field => $s ),*
                }
            }

            pub fn from_str(s: &str) -> Option<Self> {
                use self::$name::*;
                match s {
                    $( $s => Some($field), )*
                    _ => None,
                }
            }
        }
    };
}
