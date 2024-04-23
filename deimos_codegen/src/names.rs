pub const ARGC_GLOBAL: &'static str = "ARGC_GLOBAL";
pub const ARGV_GLOBAL: &'static str = "ARGV_GLOBAL";

pub const GET_FLOAT_BOOL: &'static str = "internal_get_float_bool";
pub const GET_FLOAT_BOOL_FALSE: &'static str = "internal_get_float_bool_false";

pub const FN_PREFIX: &'static str = "USER_SUB_";
pub const FN_END: &'static str = "_END";

pub const STATIC_PREFIX: &'static str = "USER_STATIC_";
pub const STRING_PREFIX: &'static str = "USER_STRING_";

pub fn get_fn_name(fn_id: usize) -> String {
    format!("{}{}", FN_PREFIX, fn_id)
}

pub fn get_fn_end(fn_id: usize) -> String {
    format!("{}{}{}", FN_PREFIX, fn_id, FN_END)
}

pub fn get_static_name(stat_id: usize) -> String {
    format!("{}{}", STATIC_PREFIX, stat_id)
}

pub fn get_str_name(str_id: usize) -> String {
    format!("{}{}", STRING_PREFIX, str_id)
}
