// Copyright Kani Contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use rustc_ast::token::TokenKind::{self, Literal};
use rustc_ast::token::{Delimiter, Lit, Token};
use rustc_ast::tokenstream::{DelimSpan, TokenStream, TokenTree};
use rustc_ast::*;
use rustc_expand::base::*;
use rustc_span::edition::Edition;
use rustc_span::{Span, Symbol, sym};
use tracing::debug;

pub(crate) fn new_expand_panic<'cx>(
    cx: &'cx mut ExtCtxt<'_>,
    sp: Span,
    tts: TokenStream,
) -> MacroExpanderResult<'cx> {
    // panic!("expand panic attempt on {sp:?}");
    let mac = sym::panic_2021;
    let sp = cx.with_call_site_ctxt(sp);

    let res = cx.expr(
        sp,
        ExprKind::MacCall(Box::new(MacCall {
            path: Path {
                span: sp,
                segments: cx
                    .std_path(&[sym::panic, mac])
                    .into_iter()
                    .map(|ident| PathSegment::from_ident(ident))
                    .collect(),
                tokens: None,
            },
            args: Box::new(DelimArgs {
                dspan: DelimSpan::from_single(sp),
                delim: Delimiter::Parenthesis,
                tokens: TokenStream::default(),
            }),
        })),
    );

    eprintln!("new expand panic on {:?}, res {:?}", sp, res);

    ExpandResult::Ready(MacEager::expr(res))
}
