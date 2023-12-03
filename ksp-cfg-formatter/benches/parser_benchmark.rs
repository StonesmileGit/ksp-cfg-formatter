use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ksp_cfg_formatter::parser::{ASTParse, State};
use nom_locate::LocatedSpan;

fn sock_parse_only(c: &mut Criterion) {
    let mut group = c.benchmark_group("larger-samle-size");
    group.measurement_time(Duration::from_secs(10));
    let path = if cfg!(windows) {
        "C:\\github\\ksp\\ksp-cfg-formatter\\ksp-cfg-formatter\\tests\\SOCK.cfg"
    } else {
        "/mnt/c/github/ksp/ksp-cfg-formatter/ksp-cfg-formatter/tests/SOCK.cfg"
    };
    let text = std::fs::read_to_string(path).unwrap();
    // let text = "node {}";
    group.bench_function("SOCK parse only", |b| {
        b.iter(|| ksp_cfg_formatter::parser::parse(black_box(&text)))
    });
    group.finish();
}

fn sock_lint_only(c: &mut Criterion) {
    let path = if cfg!(windows) {
        "C:\\github\\ksp\\ksp-cfg-formatter\\ksp-cfg-formatter\\tests\\SOCK.cfg"
    } else {
        "/mnt/c/github/ksp/ksp-cfg-formatter/ksp-cfg-formatter/tests/SOCK.cfg"
    };
    let text = std::fs::read_to_string(path).unwrap();
    let (document, _errs) = ksp_cfg_formatter::parser::parse(&text);
    c.bench_function("SOCK lint only", |b| {
        b.iter(|| ksp_cfg_formatter::linter::lint_ast(black_box(&document), None))
    });
}

fn parse_parts(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse parts");
    // group.measurement_time(Duration::from_secs(20));
    group.sample_size(10000);

    let comment = "//";
    group.bench_function("comment", |b| {
        b.iter(|| {
            ksp_cfg_formatter::parser::Comment::parse(LocatedSpan::new_extra(
                black_box(comment),
                State::default(),
            ))
        })
    });
    group.finish();
}

criterion_group!(benches, sock_parse_only, sock_lint_only, parse_parts);
// criterion_group!(benches, parse_parts);
criterion_main!(benches);
