#[derive(Debug, Clone)]
pub struct Inst;

impl Inst {
    pub const NIL: u8 = 1;
    pub const SELF_VALUE: u8 = 2;
    pub const ASSIGN: u8 = 3;
    pub const FIXNUM: u8 = 4;
    pub const DECIMALNUM: u8 = 5;
    pub const BOOL: u8 = 6;
    pub const STRING: u8 = 7;
    pub const ARRAY: u8 = 8;
    pub const ARRAY_INDEX: u8 = 9;
    pub const SEND: u8 = 10;
    pub const IDENT: u8 = 11;
    pub const TABLE_IDENT: u8 = 12;
    pub const IF: u8 = 13;
    pub const INIT_FUNC: u8 = 14;
    pub const FUNC_DECL: u8 = 15;

    pub const ADD: u8 = 40;
    pub const SUB: u8 = 41;
    pub const MUL: u8 = 42;
    pub const DIV: u8 = 43;
    pub const EQ: u8 = 44;
    pub const NE: u8 = 45;
    pub const GT: u8 = 46;
    pub const GE: u8 = 47;
    pub const LT: u8 = 48;
    pub const LE: u8 = 49;
    pub const LAND: u8 = 50;
    pub const LOR: u8 = 51;

    pub const END: u8 = 255;
}
