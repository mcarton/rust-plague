#![feature(plugin_registrar)]
#![feature(rustc_private)]
#![feature(slice_patterns)]

extern crate rustc_plugin;
extern crate syntax;

use rustc_plugin::Registry;
use syntax::abi::Abi;
use syntax::ast::Expr_::ExprTup;
use syntax::ast::Item_::ItemFn;
use syntax::ast::{Constness, Expr, FnDecl, FunctionRetTy, Generics, Ident, Item_, Path, TokenTree, Unsafety};
use syntax::codemap::{DUMMY_SP, Span, Spanned};
use syntax::ext::base::{DummyResult, ExtCtxt, MacEager, MacResult};
use syntax::ext::build::AstBuilder;
use syntax::parse::PResult;
use syntax::parse::common::seq_sep_trailing_allowed;
use syntax::parse::parser::{Parser, PathParsingMode};
use syntax::parse::token::get_ident_interner;
use syntax::parse::token::keywords::Keyword;
use syntax::parse::token::{DelimToken, Token};
use syntax::ptr::P;
use syntax::util::small_vector::SmallVector;

type ItemInfo = (Ident, Item_);

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_macro("plague", plague_macro);
}

pub fn plague_macro<'cx>(cx: &'cx mut ExtCtxt, span: Span, tts: &[TokenTree]) -> Box<MacResult + 'cx> {
    let mut parser = cx.new_parser_from_tts(tts);

    match parse_plague(&mut parser) {
        Ok((params, fn_, should_panic)) => {
            match make_plague(cx, &mut parser, params, fn_, should_panic) {
                Ok(r) => r,
                Err(mut err) => {
                    err.emit();
                    DummyResult::any(span)
                }
            }
        }
        Err(mut err) => {
            err.emit();
            DummyResult::any(span)
        }
    }
}

enum FnKind {
    Decl {
        name: Ident,
        fn_: Item_,
        span: Span,
    },
    Path(Path)
}

fn parse_plague<'a>(parser: &mut Parser<'a>) -> PResult<'a, (Spanned<Vec<P<Expr>>>, FnKind, bool)> {
    try!(parser.expect_keyword(Keyword::For));

    let params = try!(parser.parse_seq(
        &Token::OpenDelim(DelimToken::Bracket),
        &Token::CloseDelim(DelimToken::Bracket),
        seq_sep_trailing_allowed(Token::Comma),
        |parser| parser.parse_expr()
    ));

    try!(expect_keyword(parser, "test"));

    let should_panic = try!(parser.eat(&Token::Not));

    if parser.look_ahead(1, |t| { if let Token::Ident(..) = *t { true } else { false } }) {
        Ok((
            params,
            try!(parse_fn_decl(parser)),
            should_panic
        ))
    }
    else {
        Ok((
            params,
            try!(parse_fn_use(parser)),
            should_panic
        ))
    }
}

fn parse_fn_use<'a>(parser: &mut Parser<'a>) -> PResult<'a, FnKind> {
    let path = try!(parser.parse_path(PathParsingMode::NoTypesAllowed));

    Ok(FnKind::Path(path))
}

fn parse_fn_decl<'a>(parser: &mut Parser<'a>) -> PResult<'a, FnKind> {
    let mut span = parser.span;
    let (constness, unsafety, abi) = try!(parser.parse_fn_front_matter());

    // Yep, copy-pasted pieces from libsyntax
    let ident = try!(parser.parse_ident());
    let mut generics = try!(parser.parse_generics());
    let decl = try!(parser.parse_fn_decl(false));
    generics.where_clause = try!(parser.parse_where_clause());
    let body = try!(parser.parse_block());
    let fn_ = ItemFn(decl, unsafety, constness, abi, generics, body);

    span.hi = parser.last_span.hi;

    Ok(FnKind::Decl{ name: ident, fn_: fn_, span: span })
}

fn expect_keyword<'a>(parser: &mut Parser<'a>, kw: &str) -> PResult<'a, ()> {
    match parser.parse_ident() {
        Ok(test_ident) if test_ident.name.as_str() == kw => Ok(()),
        Ok(ident) => {
            Err(parser.fatal(&format!("expected `test`, found `{}`", ident)))
        }
        Err(mut err) => {
            err.cancel();
            Err(parser.fatal(&format!("expected `test`, found `{}`", parser.this_token_to_string())))
        }
    }
}

fn make_plague<'cx, 'a>(
    cx: &'cx mut ExtCtxt,
    parser: &mut Parser<'a>,
    params: Spanned<Vec<P<Expr>>>,
    fn_: FnKind,
    should_panic: bool
) -> PResult<'a, Box<MacResult + 'cx>> {
    if params.node.is_empty() {
        cx.span_err(params.span, "empty parametrized tests are useless");
    }

    let interner = get_ident_interner();

    let mut fns = Vec::with_capacity(params.node.len());

    let (ident, fn_, unpack_tuple) = match fn_ {
        FnKind::Decl { name, fn_, span } => {
            let unpack_tuple = if let &ItemFn(ref decl, _, _, _, _, _) = &fn_ {
                decl.inputs.len() > 1
            }
            else {
                panic!();
            };

            let unused = cx.attribute(span, cx.meta_list(
                    span,
                    interner.intern("allow").as_str(),
                    vec![
                        cx.meta_word(span, interner.intern("unused").as_str())
                    ]
            ));
            fns.push(cx.item(span, name, vec![unused], fn_));

            (name, cx.expr_ident(span, name), unpack_tuple)
        }
        FnKind::Path(path) => {
            let name = path.segments.iter().last().unwrap().identifier;
            // TODO: find a way to resolve the function
            (name, cx.expr_path(path), true)
        }
    };

    let attributes = {
        let make_attr = |name| {
            cx.attribute(DUMMY_SP, cx.meta_word(DUMMY_SP, interner.intern(name).as_str()))
        };

        if should_panic {
            vec![make_attr("test"), make_attr("should_panic")]
        }
        else {
            vec![make_attr("test")]
        }
    };

    let span = params.span;
    for (i, param) in params.node.iter().enumerate() {
        let params = try!(make_params(parser, &param, unpack_tuple));
        let fn_ = make_test_fn(cx, span, fn_.clone(), params);

        fns.push(cx.item(
            span,
            Ident::new(interner.intern(&format!("{}_{}", ident.name, i)), ident.ctxt),
            attributes.clone(),
            fn_
        ));
    }

    Ok(MacEager::items(SmallVector::many(fns)))
}

fn make_test_fn<'cx>(
    cx: &'cx mut ExtCtxt,
    span: Span,
    fn_: P<Expr>,
    params: Vec<P<Expr>>
) -> Item_ {
    let call = cx.expr_call(span, fn_, params);
    let block = cx.block_expr(call);

    Item_::ItemFn(
        P(FnDecl { inputs: vec![], output: FunctionRetTy::DefaultReturn(DUMMY_SP), variadic: false }),
        Unsafety::Normal,
        Constness::NotConst,
        Abi::Rust,
        Generics::default(),
        block
    )
}

fn make_params<'a>(
    parser: &mut Parser<'a>,
    params: &P<Expr>,
    unpack_tuple: bool
) -> PResult<'a, Vec<P<Expr>>> {
    if !unpack_tuple {
        Ok(vec![params.clone()])
    }
    else if let ExprTup(ref params) = params.node {
        Ok(params.clone())
    }
    else {
        Err(parser.span_fatal(params.span, "expected tuple, the test function has several arguments"))
    }
}
