use alloc::string::String;
use core::fmt;

use vm_core::Felt;

#[derive(Debug, Clone)]
pub enum DocumentationType {
    Module(String),
    Form(String),
}
impl From<DocumentationType> for String {
    fn from(doc: DocumentationType) -> Self {
        match doc {
            DocumentationType::Module(s) => s,
            DocumentationType::Form(s) => s,
        }
    }
}
impl core::ops::Deref for DocumentationType {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Module(ref s) => s,
            Self::Form(ref s) => s,
        }
    }
}

/// Represents one of the various types of values
/// that have a hex-encoded representation in Miden
/// Assembly source files.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum HexEncodedValue {
    /// A tiny value
    U8(u8),
    /// A small value
    U16(u16),
    /// A u32 constant, typically represents a memory address
    U32(u32),
    /// A single field element, 8 bytes, encoded as 16 hex digits
    Felt(Felt),
    /// A set of 4 field elements, 32 bytes, encoded as a contiguous string of 64 hex digits
    Word([Felt; 4]),
}

#[derive(Debug, Clone)]
pub enum Token<'input> {
    Add,
    Adv,
    InsertHdword,
    InsertHperm,
    InsertMem,
    AdvLoadw,
    AdvPipe,
    AdvPush,
    PushExt2intt,
    PushMapval,
    PushMapvaln,
    PushMtnode,
    PushSig,
    PushSmtpeek,
    PushSmtset,
    PushSmtget,
    PushU64Div,
    And,
    Assert,
    Assertz,
    AssertEq,
    AssertEqw,
    Begin,
    Caller,
    Call,
    Cdrop,
    Cdropw,
    Clk,
    Const,
    Cswap,
    Cswapw,
    Debug,
    Div,
    Drop,
    Dropw,
    Dup,
    Dupw,
    Dynexec,
    Dyncall,
    Else,
    Emit,
    End,
    Eq,
    Eqw,
    Ext2Add,
    Ext2Div,
    Ext2Inv,
    Ext2Mul,
    Ext2Neg,
    Ext2Sub,
    Err,
    Exec,
    Export,
    Exp,
    ExpU,
    FriExt2Fold4,
    Gt,
    Gte,
    Hash,
    Hperm,
    Hmerge,
    If,
    ILog2,
    Inv,
    IsOdd,
    Local,
    Locaddr,
    LocLoad,
    LocLoadw,
    LocStore,
    LocStorew,
    Lt,
    Lte,
    Mem,
    MemLoad,
    MemLoadw,
    MemStore,
    MemStorew,
    MemStream,
    Movdn,
    Movdnw,
    Movup,
    Movupw,
    MtreeGet,
    MtreeMerge,
    MtreeSet,
    MtreeVerify,
    Mul,
    Neg,
    Neq,
    Not,
    Or,
    Padw,
    Pow2,
    Proc,
    Procref,
    Push,
    RCombBase,
    Repeat,
    RpoFalcon512,
    Sdepth,
    Stack,
    Sub,
    Swap,
    Swapw,
    Swapdw,
    Syscall,
    Trace,
    True,
    Use,
    U32And,
    U32Assert,
    U32Assert2,
    U32Assertw,
    U32Cast,
    U32Div,
    U32Divmod,
    U32Gt,
    U32Gte,
    U32Lt,
    U32Lte,
    U32Max,
    U32Min,
    U32Mod,
    U32Not,
    U32Or,
    U32OverflowingAdd,
    U32OverflowingAdd3,
    U32OverflowingMadd,
    U32OverflowingMul,
    U32OverflowingSub,
    U32Popcnt,
    U32Clz,
    U32Ctz,
    U32Clo,
    U32Cto,
    U32Rotl,
    U32Rotr,
    U32Shl,
    U32Shr,
    U32Split,
    U32Test,
    U32Testw,
    U32WrappingAdd,
    U32WrappingAdd3,
    U32WrappingMadd,
    U32WrappingMul,
    U32WrappingSub,
    U32Xor,
    While,
    Xor,
    Bang,
    ColonColon,
    Dot,
    Equal,
    Lparen,
    Minus,
    Plus,
    SlashSlash,
    Slash,
    Star,
    Rparen,
    Rstab,
    DocComment(DocumentationType),
    HexValue(HexEncodedValue),
    Int(u64),
    Ident(&'input str),
    ConstantIdent(&'input str),
    QuotedIdent(&'input str),
    Comment,
    Eof,
}

impl<'input> fmt::Display for Token<'input> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Add => write!(f, "add"),
            Token::Adv => write!(f, "adv"),
            Token::InsertHdword => write!(f, "insert_hdword"),
            Token::InsertHperm => write!(f, "insert_hperm"),
            Token::InsertMem => write!(f, "insert_mem"),
            Token::AdvLoadw => write!(f, "adv_loadw"),
            Token::AdvPipe => write!(f, "adv_pipe"),
            Token::AdvPush => write!(f, "adv_push"),
            Token::PushExt2intt => write!(f, "push_ext2intt"),
            Token::PushMapval => write!(f, "push_mapval"),
            Token::PushMapvaln => write!(f, "push_mapvaln"),
            Token::PushMtnode => write!(f, "push_mtnode"),
            Token::PushSig => write!(f, "push_sig"),
            Token::PushSmtpeek => write!(f, "push_smtpeek"),
            Token::PushSmtset => write!(f, "push_smtset"),
            Token::PushSmtget => write!(f, "push_smtget"),
            Token::PushU64Div => write!(f, "push_u64div"),
            Token::And => write!(f, "and"),
            Token::Assert => write!(f, "assert"),
            Token::Assertz => write!(f, "assertz"),
            Token::AssertEq => write!(f, "assert_eq"),
            Token::AssertEqw => write!(f, "assert_eqw"),
            Token::Begin => write!(f, "begin"),
            Token::Caller => write!(f, "caller"),
            Token::Call => write!(f, "call"),
            Token::Cdrop => write!(f, "cdrop"),
            Token::Cdropw => write!(f, "cdropw"),
            Token::Clk => write!(f, "clk"),
            Token::Const => write!(f, "const"),
            Token::Cswap => write!(f, "cswap"),
            Token::Cswapw => write!(f, "cswapw"),
            Token::Debug => write!(f, "debug"),
            Token::Div => write!(f, "div"),
            Token::Drop => write!(f, "drop"),
            Token::Dropw => write!(f, "dropw"),
            Token::Dup => write!(f, "dup"),
            Token::Dupw => write!(f, "dupw"),
            Token::Dynexec => write!(f, "dynexec"),
            Token::Dyncall => write!(f, "dyncall"),
            Token::Else => write!(f, "else"),
            Token::Emit => write!(f, "emit"),
            Token::End => write!(f, "end"),
            Token::Eq => write!(f, "eq"),
            Token::Eqw => write!(f, "eqw"),
            Token::Ext2Add => write!(f, "ext2add"),
            Token::Ext2Div => write!(f, "ext2div"),
            Token::Ext2Inv => write!(f, "ext2inv"),
            Token::Ext2Mul => write!(f, "ext2mul"),
            Token::Ext2Neg => write!(f, "ext2neg"),
            Token::Ext2Sub => write!(f, "ext2sub"),
            Token::Err => write!(f, "err"),
            Token::Exec => write!(f, "exec"),
            Token::Exp => write!(f, "exp"),
            Token::ExpU => write!(f, "exp.u"),
            Token::Export => write!(f, "export"),
            Token::FriExt2Fold4 => write!(f, "fri_ext2fold4"),
            Token::Gt => write!(f, "gt"),
            Token::Gte => write!(f, "gte"),
            Token::Hash => write!(f, "hash"),
            Token::Hperm => write!(f, "hperm"),
            Token::Hmerge => write!(f, "hmerge"),
            Token::If => write!(f, "if"),
            Token::ILog2 => write!(f, "ilog2"),
            Token::Inv => write!(f, "inv"),
            Token::IsOdd => write!(f, "is_odd"),
            Token::Local => write!(f, "local"),
            Token::Locaddr => write!(f, "locaddr"),
            Token::LocLoad => write!(f, "loc_load"),
            Token::LocLoadw => write!(f, "loc_loadw"),
            Token::LocStore => write!(f, "loc_store"),
            Token::LocStorew => write!(f, "loc_storew"),
            Token::Lt => write!(f, "lt"),
            Token::Lte => write!(f, "lte"),
            Token::Mem => write!(f, "mem"),
            Token::MemLoad => write!(f, "mem_load"),
            Token::MemLoadw => write!(f, "mem_loadw"),
            Token::MemStore => write!(f, "mem_store"),
            Token::MemStorew => write!(f, "mem_storew"),
            Token::MemStream => write!(f, "mem_stream"),
            Token::Movdn => write!(f, "movdn"),
            Token::Movdnw => write!(f, "movdnw"),
            Token::Movup => write!(f, "movup"),
            Token::Movupw => write!(f, "movupw"),
            Token::MtreeGet => write!(f, "mtree_get"),
            Token::MtreeMerge => write!(f, "mtree_merge"),
            Token::MtreeSet => write!(f, "mtree_set"),
            Token::MtreeVerify => write!(f, "mtree_verify"),
            Token::Mul => write!(f, "mul"),
            Token::Neg => write!(f, "neg"),
            Token::Neq => write!(f, "neq"),
            Token::Not => write!(f, "not"),
            Token::Or => write!(f, "or"),
            Token::Padw => write!(f, "padw"),
            Token::Pow2 => write!(f, "pow2"),
            Token::Proc => write!(f, "proc"),
            Token::Procref => write!(f, "procref"),
            Token::Push => write!(f, "push"),
            Token::RCombBase => write!(f, "rcomb_base"),
            Token::Repeat => write!(f, "repeat"),
            Token::RpoFalcon512 => write!(f, "rpo_falcon512"),
            Token::Sdepth => write!(f, "sdepth"),
            Token::Stack => write!(f, "stack"),
            Token::Sub => write!(f, "sub"),
            Token::Swap => write!(f, "swap"),
            Token::Swapw => write!(f, "swapw"),
            Token::Swapdw => write!(f, "swapdw"),
            Token::Syscall => write!(f, "syscall"),
            Token::Trace => write!(f, "trace"),
            Token::True => write!(f, "true"),
            Token::Use => write!(f, "use"),
            Token::U32And => write!(f, "u32and"),
            Token::U32Assert => write!(f, "u32assert"),
            Token::U32Assert2 => write!(f, "u32assert2"),
            Token::U32Assertw => write!(f, "u32assertw"),
            Token::U32Cast => write!(f, "u32cast"),
            Token::U32Div => write!(f, "u32div"),
            Token::U32Divmod => write!(f, "u32divmod"),
            Token::U32Gt => write!(f, "u32gt"),
            Token::U32Gte => write!(f, "u32gte"),
            Token::U32Lt => write!(f, "u32lt"),
            Token::U32Lte => write!(f, "u32lte"),
            Token::U32Max => write!(f, "u32max"),
            Token::U32Min => write!(f, "u32min"),
            Token::U32Mod => write!(f, "u32mod"),
            Token::U32Not => write!(f, "u32not"),
            Token::U32Or => write!(f, "u32or"),
            Token::U32OverflowingAdd => write!(f, "u32overflowing_add"),
            Token::U32OverflowingAdd3 => write!(f, "u32overflowing_add3"),
            Token::U32OverflowingMadd => write!(f, "u32overflowing_madd"),
            Token::U32OverflowingMul => write!(f, "u32overflowing_mul"),
            Token::U32OverflowingSub => write!(f, "u32overflowing_sub"),
            Token::U32Popcnt => write!(f, "u32popcnt"),
            Token::U32Clz => write!(f, "u32clz"),
            Token::U32Ctz => write!(f, "u32ctz"),
            Token::U32Clo => write!(f, "u32clo"),
            Token::U32Cto => write!(f, "u32cto"),
            Token::U32Rotl => write!(f, "u32rotl"),
            Token::U32Rotr => write!(f, "u32rotr"),
            Token::U32Shl => write!(f, "u32shl"),
            Token::U32Shr => write!(f, "u32shr"),
            Token::U32Split => write!(f, "u32split"),
            Token::U32Test => write!(f, "u32test"),
            Token::U32Testw => write!(f, "u32testw"),
            Token::U32WrappingAdd => write!(f, "u32wrapping_add"),
            Token::U32WrappingAdd3 => write!(f, "u32wrapping_add3"),
            Token::U32WrappingMadd => write!(f, "u32wrapping_madd"),
            Token::U32WrappingMul => write!(f, "u32wrapping_mul"),
            Token::U32WrappingSub => write!(f, "u32wrapping_sub"),
            Token::U32Xor => write!(f, "u32xor"),
            Token::While => write!(f, "while"),
            Token::Xor => write!(f, "xor"),
            Token::Bang => write!(f, "!"),
            Token::ColonColon => write!(f, "::"),
            Token::Dot => write!(f, "."),
            Token::Equal => write!(f, "="),
            Token::Lparen => write!(f, "("),
            Token::Minus => write!(f, "-"),
            Token::Plus => write!(f, "+"),
            Token::SlashSlash => write!(f, "//"),
            Token::Slash => write!(f, "/"),
            Token::Star => write!(f, "*"),
            Token::Rparen => write!(f, ")"),
            Token::Rstab => write!(f, "->"),
            Token::DocComment(DocumentationType::Module(_)) => f.write_str("module doc"),
            Token::DocComment(DocumentationType::Form(_)) => f.write_str("doc comment"),
            Token::HexValue(_) => f.write_str("hex-encoded value"),
            Token::Int(_) => f.write_str("integer"),
            Token::Ident(_) => f.write_str("identifier"),
            Token::ConstantIdent(_) => f.write_str("constant identifier"),
            Token::QuotedIdent(_) => f.write_str("quoted identifier"),
            Token::Comment => f.write_str("comment"),
            Token::Eof => write!(f, "end of file"),
        }
    }
}

impl<'input> Token<'input> {
    pub fn is_instruction(&self) -> bool {
        matches!(
            self,
            Token::Add
                | Token::Adv
                | Token::InsertHdword
                | Token::InsertHperm
                | Token::InsertMem
                | Token::AdvLoadw
                | Token::AdvPipe
                | Token::AdvPush
                | Token::PushExt2intt
                | Token::PushMapval
                | Token::PushMapvaln
                | Token::PushMtnode
                | Token::PushSig
                | Token::PushSmtpeek
                | Token::PushSmtset
                | Token::PushSmtget
                | Token::PushU64Div
                | Token::And
                | Token::Assert
                | Token::Assertz
                | Token::AssertEq
                | Token::AssertEqw
                | Token::Caller
                | Token::Call
                | Token::Cdrop
                | Token::Cdropw
                | Token::Clk
                | Token::Cswap
                | Token::Cswapw
                | Token::Debug
                | Token::Div
                | Token::Drop
                | Token::Dropw
                | Token::Dup
                | Token::Dupw
                | Token::Dynexec
                | Token::Dyncall
                | Token::Emit
                | Token::Eq
                | Token::Eqw
                | Token::Ext2Add
                | Token::Ext2Div
                | Token::Ext2Inv
                | Token::Ext2Mul
                | Token::Ext2Neg
                | Token::Ext2Sub
                | Token::Exec
                | Token::Exp
                | Token::ExpU
                | Token::FriExt2Fold4
                | Token::Gt
                | Token::Gte
                | Token::Hash
                | Token::Hperm
                | Token::Hmerge
                | Token::ILog2
                | Token::Inv
                | Token::IsOdd
                | Token::Local
                | Token::Locaddr
                | Token::LocLoad
                | Token::LocLoadw
                | Token::LocStore
                | Token::LocStorew
                | Token::Lt
                | Token::Lte
                | Token::Mem
                | Token::MemLoad
                | Token::MemLoadw
                | Token::MemStore
                | Token::MemStorew
                | Token::MemStream
                | Token::Movdn
                | Token::Movdnw
                | Token::Movup
                | Token::Movupw
                | Token::MtreeGet
                | Token::MtreeMerge
                | Token::MtreeSet
                | Token::MtreeVerify
                | Token::Mul
                | Token::Neg
                | Token::Neq
                | Token::Not
                | Token::Or
                | Token::Padw
                | Token::Pow2
                | Token::Procref
                | Token::Push
                | Token::RCombBase
                | Token::Repeat
                | Token::Sdepth
                | Token::Stack
                | Token::Sub
                | Token::Swap
                | Token::Swapw
                | Token::Swapdw
                | Token::Syscall
                | Token::Trace
                | Token::U32And
                | Token::U32Assert
                | Token::U32Assert2
                | Token::U32Assertw
                | Token::U32Cast
                | Token::U32Div
                | Token::U32Divmod
                | Token::U32Gt
                | Token::U32Gte
                | Token::U32Lt
                | Token::U32Lte
                | Token::U32Max
                | Token::U32Min
                | Token::U32Mod
                | Token::U32Not
                | Token::U32Or
                | Token::U32OverflowingAdd
                | Token::U32OverflowingAdd3
                | Token::U32OverflowingMadd
                | Token::U32OverflowingMul
                | Token::U32OverflowingSub
                | Token::U32Popcnt
                | Token::U32Clz
                | Token::U32Ctz
                | Token::U32Clo
                | Token::U32Cto
                | Token::U32Rotl
                | Token::U32Rotr
                | Token::U32Shl
                | Token::U32Shr
                | Token::U32Split
                | Token::U32Test
                | Token::U32Testw
                | Token::U32WrappingAdd
                | Token::U32WrappingAdd3
                | Token::U32WrappingMadd
                | Token::U32WrappingMul
                | Token::U32WrappingSub
                | Token::U32Xor
                | Token::Xor
        )
    }

    const KEYWORDS: &'static [(&'static str, Token<'static>)] = &[
        ("add", Token::Add),
        ("adv", Token::Adv),
        ("insert_hdword", Token::InsertHdword),
        ("insert_hperm", Token::InsertHperm),
        ("insert_mem", Token::InsertMem),
        ("adv_loadw", Token::AdvLoadw),
        ("adv_pipe", Token::AdvPipe),
        ("adv_push", Token::AdvPush),
        ("push_ext2intt", Token::PushExt2intt),
        ("push_mapval", Token::PushMapval),
        ("push_mapvaln", Token::PushMapvaln),
        ("push_mtnode", Token::PushMtnode),
        ("push_sig", Token::PushSig),
        ("push_smtpeek", Token::PushSmtpeek),
        ("push_smtset", Token::PushSmtset),
        ("push_smtget", Token::PushSmtget),
        ("push_u64div", Token::PushU64Div),
        ("and", Token::And),
        ("assert", Token::Assert),
        ("assertz", Token::Assertz),
        ("assert_eq", Token::AssertEq),
        ("assert_eqw", Token::AssertEqw),
        ("begin", Token::Begin),
        ("caller", Token::Caller),
        ("call", Token::Call),
        ("cdrop", Token::Cdrop),
        ("cdropw", Token::Cdropw),
        ("clk", Token::Clk),
        ("const", Token::Const),
        ("cswap", Token::Cswap),
        ("cswapw", Token::Cswapw),
        ("debug", Token::Debug),
        ("div", Token::Div),
        ("drop", Token::Drop),
        ("dropw", Token::Dropw),
        ("dup", Token::Dup),
        ("dupw", Token::Dupw),
        ("dynexec", Token::Dynexec),
        ("dyncall", Token::Dyncall),
        ("else", Token::Else),
        ("emit", Token::Emit),
        ("end", Token::End),
        ("eq", Token::Eq),
        ("eqw", Token::Eqw),
        ("ext2add", Token::Ext2Add),
        ("ext2div", Token::Ext2Div),
        ("ext2inv", Token::Ext2Inv),
        ("ext2mul", Token::Ext2Mul),
        ("ext2neg", Token::Ext2Neg),
        ("ext2sub", Token::Ext2Sub),
        ("err", Token::Err),
        ("exec", Token::Exec),
        ("exp", Token::Exp),
        ("exp.u", Token::ExpU),
        ("export", Token::Export),
        ("fri_ext2fold4", Token::FriExt2Fold4),
        ("gt", Token::Gt),
        ("gte", Token::Gte),
        ("hash", Token::Hash),
        ("hperm", Token::Hperm),
        ("hmerge", Token::Hmerge),
        ("if", Token::If),
        ("ilog2", Token::ILog2),
        ("inv", Token::Inv),
        ("is_odd", Token::IsOdd),
        ("local", Token::Local),
        ("locaddr", Token::Locaddr),
        ("loc_load", Token::LocLoad),
        ("loc_loadw", Token::LocLoadw),
        ("loc_store", Token::LocStore),
        ("loc_storew", Token::LocStorew),
        ("lt", Token::Lt),
        ("lte", Token::Lte),
        ("mem", Token::Mem),
        ("mem_load", Token::MemLoad),
        ("mem_loadw", Token::MemLoadw),
        ("mem_store", Token::MemStore),
        ("mem_storew", Token::MemStorew),
        ("mem_stream", Token::MemStream),
        ("movdn", Token::Movdn),
        ("movdnw", Token::Movdnw),
        ("movup", Token::Movup),
        ("movupw", Token::Movupw),
        ("mtree_get", Token::MtreeGet),
        ("mtree_merge", Token::MtreeMerge),
        ("mtree_set", Token::MtreeSet),
        ("mtree_verify", Token::MtreeVerify),
        ("mul", Token::Mul),
        ("neg", Token::Neg),
        ("neq", Token::Neq),
        ("not", Token::Not),
        ("or", Token::Or),
        ("padw", Token::Padw),
        ("pow2", Token::Pow2),
        ("proc", Token::Proc),
        ("procref", Token::Procref),
        ("push", Token::Push),
        ("rcomb_base", Token::RCombBase),
        ("repeat", Token::Repeat),
        ("rpo_falcon512", Token::RpoFalcon512),
        ("sdepth", Token::Sdepth),
        ("stack", Token::Stack),
        ("sub", Token::Sub),
        ("swap", Token::Swap),
        ("swapw", Token::Swapw),
        ("swapdw", Token::Swapdw),
        ("syscall", Token::Syscall),
        ("trace", Token::Trace),
        ("true", Token::True),
        ("use", Token::Use),
        ("u32and", Token::U32And),
        ("u32assert", Token::U32Assert),
        ("u32assert2", Token::U32Assert2),
        ("u32assertw", Token::U32Assertw),
        ("u32cast", Token::U32Cast),
        ("u32div", Token::U32Div),
        ("u32divmod", Token::U32Divmod),
        ("u32gt", Token::U32Gt),
        ("u32gte", Token::U32Gte),
        ("u32lt", Token::U32Lt),
        ("u32lte", Token::U32Lte),
        ("u32max", Token::U32Max),
        ("u32min", Token::U32Min),
        ("u32mod", Token::U32Mod),
        ("u32not", Token::U32Not),
        ("u32or", Token::U32Or),
        ("u32overflowing_add", Token::U32OverflowingAdd),
        ("u32overflowing_add3", Token::U32OverflowingAdd3),
        ("u32overflowing_madd", Token::U32OverflowingMadd),
        ("u32overflowing_mul", Token::U32OverflowingMul),
        ("u32overflowing_sub", Token::U32OverflowingSub),
        ("u32popcnt", Token::U32Popcnt),
        ("u32clz", Token::U32Clz),
        ("u32ctz", Token::U32Ctz),
        ("u32clo", Token::U32Clo),
        ("u32cto", Token::U32Cto),
        ("u32rotl", Token::U32Rotl),
        ("u32rotr", Token::U32Rotr),
        ("u32shl", Token::U32Shl),
        ("u32shr", Token::U32Shr),
        ("u32split", Token::U32Split),
        ("u32test", Token::U32Test),
        ("u32testw", Token::U32Testw),
        ("u32wrapping_add", Token::U32WrappingAdd),
        ("u32wrapping_add3", Token::U32WrappingAdd3),
        ("u32wrapping_madd", Token::U32WrappingMadd),
        ("u32wrapping_mul", Token::U32WrappingMul),
        ("u32wrapping_sub", Token::U32WrappingSub),
        ("u32xor", Token::U32Xor),
        ("while", Token::While),
        ("xor", Token::Xor),
    ];

    pub fn keyword_searcher() -> aho_corasick::AhoCorasick {
        use aho_corasick::AhoCorasick;

        // Execute a search for any of the keywords above,
        // matching longest first, and requiring the match
        // to cover the entire input.
        AhoCorasick::builder()
            .match_kind(aho_corasick::MatchKind::LeftmostLongest)
            .start_kind(aho_corasick::StartKind::Anchored)
            .build(Self::KEYWORDS.iter().map(|(kw, _)| kw).copied())
            .expect("unable to build aho-corasick searcher for token")
    }

    pub fn from_keyword_or_ident(s: &'input str) -> Self {
        let searcher = Self::keyword_searcher();
        Self::from_keyword_or_ident_with_searcher(s, &searcher)
    }

    pub fn from_keyword_or_ident_with_searcher(
        s: &'input str,
        searcher: &aho_corasick::AhoCorasick,
    ) -> Self {
        let input = aho_corasick::Input::new(s).anchored(aho_corasick::Anchored::Yes);
        match searcher.find(input) {
            // No match, it's an ident
            None => Token::Ident(s),
            // If the match is not exact, it's an ident
            Some(matched) if matched.len() != s.as_bytes().len() => Token::Ident(s),
            // Otherwise clone the Token corresponding to the keyword that was matched
            Some(matched) => Self::KEYWORDS[matched.pattern().as_usize()].1.clone(),
        }
    }

    pub fn parse(s: &'input str) -> Result<Token<'input>, ()> {
        match Token::from_keyword_or_ident(s) {
            Token::Ident(_) => {
                // Nope, try again
                match s {
                    "!" => Ok(Token::Bang),
                    "::" => Ok(Token::ColonColon),
                    "." => Ok(Token::Dot),
                    "=" => Ok(Token::Equal),
                    "(" => Ok(Token::Lparen),
                    "-" => Ok(Token::Minus),
                    "+" => Ok(Token::Plus),
                    "//" => Ok(Token::SlashSlash),
                    "/" => Ok(Token::Slash),
                    "*" => Ok(Token::Star),
                    ")" => Ok(Token::Rparen),
                    "->" => Ok(Token::Rstab),
                    "end of file" => Ok(Token::Eof),
                    "module doc" => Ok(Token::DocComment(DocumentationType::Module(String::new()))),
                    "doc comment" => Ok(Token::DocComment(DocumentationType::Form(String::new()))),
                    "comment" => Ok(Token::Comment),
                    "hex-encoded value" => Ok(Token::HexValue(HexEncodedValue::U8(0))),
                    "integer" => Ok(Token::Int(0)),
                    "identifier" => Ok(Token::Ident("")),
                    "constant identifier" => Ok(Token::ConstantIdent("")),
                    "quoted identifier" => Ok(Token::QuotedIdent("")),
                    _ => Err(()),
                }
            }
            // We matched a keyword
            token => Ok(token),
        }
    }
}
