#![allow(dead_code)]
#![feature(linked_list_cursors)]
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ksp_cfg_formatter::token_formatter::{
    format_blocks, indentation, remove_leading_and_trailing_newlines, remove_leading_whitespace,
    remove_trailing_whitespace, Formatter, Indentation, LineReturn, Token,
};
use logos::Logos;
use std::{
    collections::LinkedList,
    fs,
    path::{Path, PathBuf},
};

fn read_local_path(path: &str) -> String {
    let base_path = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(base_path).join(PathBuf::from(path));
    fs::read_to_string(path).expect("Failed to read path provided")
}

fn bench_tokenizer(bench: &mut Criterion) {
    let text = read_local_path("tests/sock.cfg");
    bench.bench_function("Tokenizer", |b| {
        b.iter(|| {
            let mut _token_list = Token::lexer(black_box(&text)).collect::<LinkedList<Token>>();
        })
    });
}

fn bench_sock(bench: &mut Criterion) {
    let text = read_local_path("tests/sock.cfg");
    bench.bench_function("sock", |b| {
        b.iter(|| {
            let formatter = Formatter::new(Indentation::Tabs, false, LineReturn::Identify);
            let _ = formatter.format_text(black_box(&text));
        })
    });
}

fn bench_leading_whitespace(bench: &mut Criterion) {
    let text = read_local_path("tests/sock.cfg");
    let token_list = Token::lexer(&text).collect::<LinkedList<Token>>();
    bench.bench_function("leading whitespace", |b| {
        b.iter_batched(
            || token_list.clone(),
            |mut token_list| {
                remove_leading_whitespace(&mut token_list.cursor_front_mut());
            },
            criterion::BatchSize::LargeInput,
        )
    });
}

fn bench_block(bench: &mut Criterion) {
    let text = read_local_path("tests/sock.cfg");
    let token_list = Token::lexer(&text).collect::<LinkedList<Token>>();
    bench.bench_function("block", |b| {
        b.iter_batched(
            || token_list.clone(),
            |mut token_list| {
                format_blocks(&mut token_list.cursor_front_mut(), &true, &false);
            },
            criterion::BatchSize::LargeInput,
        )
    });
}

fn bench_trailing_whitespace(bench: &mut Criterion) {
    let text = read_local_path("tests/sock.cfg");
    let token_list = Token::lexer(&text).collect::<LinkedList<Token>>();
    bench.bench_function("trailing whitespace", |b| {
        b.iter_batched(
            || token_list.clone(),
            |mut token_list| {
                remove_trailing_whitespace(&mut token_list.cursor_front_mut());
            },
            criterion::BatchSize::LargeInput,
        )
    });
}

fn bench_whitelines(bench: &mut Criterion) {
    let text = read_local_path("tests/sock.cfg");
    let token_list = Token::lexer(&text).collect::<LinkedList<Token>>();
    bench.bench_function("whitelines", |b| {
        b.iter_batched(
            || token_list.clone(),
            |mut token_list| {
                remove_leading_and_trailing_newlines(&mut token_list.cursor_front_mut());
            },
            criterion::BatchSize::LargeInput,
        )
    });
}

fn bench_indentation(bench: &mut Criterion) {
    let text = read_local_path("tests/sock.cfg");
    let token_list = Token::lexer(&text).collect::<LinkedList<Token>>();
    bench.bench_function("indentation", |b| {
        b.iter_batched(
            || token_list.clone(),
            |mut token_list| {
                indentation(&mut token_list.cursor_front_mut(), Indentation::Tabs);
            },
            criterion::BatchSize::LargeInput,
        )
    });
}

criterion_group!(
    benches,
    bench_tokenizer,
    bench_sock,
    bench_leading_whitespace,
    bench_block,
    bench_trailing_whitespace,
    bench_whitelines,
    bench_indentation
);
criterion_main!(benches);
