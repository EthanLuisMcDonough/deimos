pub const ARGC_GLOBAL: &'static str = "ARGC_GLOBAL";
pub const ARGV_GLOBAL: &'static str = "ARGV_GLOBAL";

pub const GET_FLOAT_BOOL: &'static str = "internal_get_float_bool";
pub const GET_FLOAT_BOOL_FALSE: &'static str = "internal_get_float_bool_false";

pub const GET_FLOAT_BOOL_INV: &'static str = "internal_get_float_bool_inv";
pub const GET_FLOAT_BOOL_INV_FALSE: &'static str = "internal_get_float_bool_inv_false";

pub const FN_PREFIX: &'static str = "USER_SUB_";
pub const FN_END: &'static str = "_END";

pub const STATIC_PREFIX: &'static str = "USER_STATIC_";
pub const STRING_PREFIX: &'static str = "USER_STRING_";

pub const IF_BLOCK_PREFIX: &'static str = "IF_BRANCH_";
pub const ELIF_MODIFIER: &'static str = "_ELIF_";
pub const ELSE_MODIFIER: &'static str = "_ELSE";
pub const IF_BLOCK_END_SUFFIX: &'static str = "_END";

pub const WHILE_BLOCK_PREFIX: &'static str = "WHILE_BLOCK_";
pub const WHILE_BLOCK_SUFFIX: &'static str = "_END";

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

pub fn get_if_lbl(construct_id: usize) -> String {
    format!("{}{}", IF_BLOCK_PREFIX, construct_id)
}

pub fn get_elif_lbl(construct_id: usize, elif_ind: usize) -> String {
    format!(
        "{}{}{}{}",
        IF_BLOCK_PREFIX, construct_id, ELIF_MODIFIER, elif_ind
    )
}

pub fn get_if_else(construct_id: usize) -> String {
    format!("{}{}{}", IF_BLOCK_PREFIX, construct_id, ELSE_MODIFIER)
}

pub fn get_if_end(construct_id: usize) -> String {
    format!("{}{}{}", IF_BLOCK_PREFIX, construct_id, IF_BLOCK_END_SUFFIX)
}

pub fn get_while_lbl(while_id: usize) -> String {
    format!("{}{}", WHILE_BLOCK_PREFIX, while_id)
}

pub fn get_while_end(while_id: usize) -> String {
    format!("{}{}{}", WHILE_BLOCK_PREFIX, while_id, WHILE_BLOCK_SUFFIX)
}
