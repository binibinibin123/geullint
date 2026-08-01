use geullint_core::{Engine, LintConfig, SourceKind};
use std::{env, error::Error, fs, hint::black_box, path::PathBuf, time::Instant};

struct Arguments {
    fixture: PathBuf,
    source_kind: SourceKind,
    warmup: usize,
    iterations: usize,
}

fn next_value(arguments: &mut impl Iterator<Item = String>, name: &str) -> Result<String, String> {
    arguments
        .next()
        .ok_or_else(|| format!("{name} requires a value"))
}

fn positive_integer(value: &str, name: &str) -> Result<usize, String> {
    value
        .parse::<usize>()
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(|| format!("{name} must be a positive integer"))
}

fn parse_source_kind(value: &str) -> Result<SourceKind, String> {
    match value {
        "plain_text" => Ok(SourceKind::PlainText),
        "markdown" => Ok(SourceKind::Markdown),
        "typescript" => Ok(SourceKind::TypeScript),
        _ => Err("source-kind must be plain_text, markdown, or typescript".to_owned()),
    }
}

fn parse_arguments() -> Result<Arguments, String> {
    let mut fixture = None;
    let mut source_kind = None;
    let mut warmup = None;
    let mut iterations = None;
    let mut arguments = env::args().skip(1);

    while let Some(name) = arguments.next() {
        let value = next_value(&mut arguments, &name)?;
        match name.as_str() {
            "--fixture" => fixture = Some(PathBuf::from(value)),
            "--source-kind" => source_kind = Some(parse_source_kind(&value)?),
            "--warmup" => warmup = Some(positive_integer(&value, "warmup")?),
            "--iterations" => iterations = Some(positive_integer(&value, "iterations")?),
            _ => return Err(format!("unknown option: {name}")),
        }
    }

    Ok(Arguments {
        fixture: fixture.ok_or_else(|| "--fixture is required".to_owned())?,
        source_kind: source_kind.ok_or_else(|| "--source-kind is required".to_owned())?,
        warmup: warmup.ok_or_else(|| "--warmup is required".to_owned())?,
        iterations: iterations.ok_or_else(|| "--iterations is required".to_owned())?,
    })
}

fn main() -> Result<(), Box<dyn Error>> {
    let arguments = parse_arguments().map_err(|message| format!("GeulLint probe: {message}"))?;
    let text = fs::read_to_string(arguments.fixture)?;
    let engine = Engine::new(LintConfig::default());

    let first_start = Instant::now();
    let first_diagnostics = engine.check(&text, arguments.source_kind);
    let first_check_ms = first_start.elapsed().as_secs_f64() * 1_000.0;
    black_box(first_diagnostics.len());

    for _ in 0..arguments.warmup {
        black_box(engine.check(&text, arguments.source_kind).len());
    }

    let mut samples = Vec::with_capacity(arguments.iterations);
    let mut diagnostics = 0;
    for _ in 0..arguments.iterations {
        let start = Instant::now();
        diagnostics = engine.check(&text, arguments.source_kind).len();
        samples.push(start.elapsed().as_secs_f64() * 1_000.0);
        black_box(diagnostics);
    }

    let samples = samples
        .iter()
        .map(|sample| format!("{sample:.6}"))
        .collect::<Vec<_>>()
        .join(",");
    println!(
        "{{\"firstCheckMs\":{first_check_ms:.6},\"diagnostics\":{diagnostics},\"samplesMs\":[{samples}]}}"
    );
    Ok(())
}
