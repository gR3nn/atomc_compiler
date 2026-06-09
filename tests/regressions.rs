use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn run_source(source: &str) -> std::process::Output {
    let mut path = PathBuf::from(env::temp_dir());
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    path.push(format!("atomc_regression_{unique}.c"));

    fs::write(&path, source).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_atomc_compiler"))
        .arg(&path)
        .output()
        .unwrap();
    let _ = fs::remove_file(&path);
    output
}

#[test]
fn accepts_grouped_expressions() {
    let output = run_source("void main(){int a; a=(1+2);}");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn rejects_unsized_array_variables() {
    let output = run_source("void main(){int v[];}");
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("A vector variable must have a specified dimension")
    );
}

#[test]
fn accepts_sized_array_parameters() {
    let output = run_source("void f(int v[10]){} void main(){}");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn resolves_struct_types_even_if_name_is_shadowed() {
    let output = run_source("struct S{int x;}; void main(){int S; struct S a; a.x=1;}");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn accepts_zero_with_exponent_as_real_constant() {
    let output = run_source("void main(){double x; x=0e1;}");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn rejects_hex_constant_without_digits() {
    let output = run_source("void main(){int x; x=0x;}");
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("Expected hexadecimal digits after 0x")
    );
}
