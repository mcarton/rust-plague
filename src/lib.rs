#![feature(plugin_registrar)]
#![feature(quote)]
#![feature(rustc_private)]
#![feature(slice_patterns)]

extern crate rustc_plugin;
extern crate syntax;

use rustc_plugin::Registry;
use syntax::abi::Abi;
use syntax::ast::{
    Constness, Expr, ExprKind, FnDecl, FunctionRetTy, Generics, Ident, ItemKind, Lifetime, Path,
    Unsafety
};
use syntax::codemap::{self, DUMMY_SP, Span, Spanned};
use syntax::ext::base::{DummyResult, ExtCtxt, MacEager, MacResult};
use syntax::ext::build::AstBuilder;
use syntax::parse::PResult;
use syntax::parse::common::SeqSep;
use syntax::parse::parser::{Parser, PathStyle};
use syntax::parse::token::keywords;
use syntax::parse::token::{DelimToken, Token};
use syntax::ptr::P;
use syntax::tokenstream::TokenTree;
use syntax::util::small_vector::SmallVector;

type ItemInfo = (Ident, ItemKind);

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
        fn_: ItemKind,
        span: Span,
    },
    Path(Path)
}

fn parse_plague<'a>(parser: &mut Parser<'a>) -> PResult<'a, (Spanned<Vec<Param>>, FnKind, bool)> {
    try!(parser.expect_keyword(keywords::For));

    let params = try!(parser.parse_seq(
        &Token::OpenDelim(DelimToken::Bracket),
        &Token::CloseDelim(DelimToken::Bracket),
        SeqSep::trailing_allowed(Token::Comma),
        parse_param
    ));

    try!(expect_keyword(parser, "test"));

    let should_panic = parser.eat(&Token::Not);

    let result = if parser.look_ahead(1, Token::is_ident) {
        try!(parse_fn_decl(parser))
    }
    else {
        try!(parse_fn_use(parser))
    };

    if !parser.eat(&Token::Eof) {
        parser.span_err(parser.span, &format!("expected end of macro, got `{}`", parser. this_token_to_string()));
    }

    Ok((params, result, should_panic))
}

type Param = (Option<Lifetime>, P<Expr>, Option<P<Expr>>);

fn parse_param<'a>(parser: &mut Parser<'a>) -> PResult<'a, Param> {
    let name = try!(parser.parse_opt_lifetime());
    let expr = try!(parser.parse_expr());

    let ret = if parser.eat(&Token::RArrow) {
        Some(try!(parser.parse_expr()))
    }
    else {
        None
    };

    Ok((name, expr, ret))
}

fn parse_fn_use<'a>(parser: &mut Parser<'a>) -> PResult<'a, FnKind> {
    let path = try!(parser.parse_path(PathStyle::Expr));

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
    let fn_ = ItemKind::Fn(decl, unsafety, constness, abi, generics, body);

    span.hi = parser.prev_span.hi;

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
    params: Spanned<Vec<Param>>,
    fn_: FnKind,
    should_panic: bool
) -> PResult<'a, Box<MacResult + 'cx>> {
    if params.node.is_empty() {
        cx.span_err(params.span, "empty parametrized tests are useless");
    }

    let mut fns = Vec::with_capacity(params.node.len());

    let (ident, fn_) = match fn_ {
        FnKind::Decl { name, fn_, span } => {
            let unused = quote_meta_item!(cx, allow(unused));
            let unused = cx.attribute(span, unused);
            fns.push(cx.item(span, name, vec![unused], fn_));

            (name, cx.expr_ident(span, name))
        }
        FnKind::Path(path) => {
            let name = path.segments.iter().last().unwrap().identifier;
            (name, cx.expr_path(path))
        }
    };

    let attributes = if should_panic {
        vec![
            cx.attribute(DUMMY_SP, quote_meta_item!(cx, test)),
            cx.attribute(DUMMY_SP, quote_meta_item!(cx, should_panic)),
        ]
    }
    else {
        vec![cx.attribute(DUMMY_SP, quote_meta_item!(cx, test))]
    };

    let span = params.span;
    for (i, param) in params.node.iter().enumerate() {
        let params = try!(make_params(parser, &param.1));
        let fn_ = make_test_fn(cx, span, fn_.clone(), params, &param.2);

        let name = if let Some(name) = param.0 {
            format!("{}{}", ident.name, name.name)
        }
        else {
            format!("{}#{}", ident.name, i)
        };

        fns.push(cx.item(
            span,
            //cx.ident_of(&name),
            Ident {
                name: cx.name_of(&name),
                ctxt: ident.ctxt,
            },
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
    params: Vec<P<Expr>>,
    ret: &Option<P<Expr>>,
) -> ItemKind {
    let call = cx.expr_call(span, fn_, params);
    let call = if let Some(ref exp_ret) = *ret {
        quote_expr!(cx, {
            let exp = $exp_ret;
            let got = $call;

            if !(got == exp) {
                panic!("test failed: got `{:?}`, expected `{:?}`", got, exp);
            }
        })
    }
    else {
        call
    };

    let block = cx.block_expr(call);

    ItemKind::Fn(
        P(FnDecl { inputs: vec![], output: FunctionRetTy::Default(DUMMY_SP), variadic: false }),
        Unsafety::Normal,
        codemap::respan(span, Constness::NotConst),
        Abi::Rust,
        Generics::default(),
        block
    )
}

fn make_params<'a>(parser: &mut Parser<'a>, params: &P<Expr>) -> PResult<'a, Vec<P<Expr>>> {
    if let ExprKind::Tup(ref params) = params.node {
        Ok(params.clone())
    }
    else {
        Err(parser.span_fatal(params.span, "expected tuple literal"))
    }
}
