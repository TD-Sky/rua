use indoc::indoc;
use once_cell::sync::Lazy;
use tracing_subscriber::EnvFilter;

use crate::rua;

static LOG: Lazy<()> = Lazy::new(|| {
    tracing_subscriber::fmt()
        .without_time()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
});

fn init_log() {
    Lazy::force(&LOG);
}

#[test]
fn test_print() {
    init_log();
    let source = indoc! {"
        print(nil)
        print(false)
        print(123)
        print(123456)
        print(123456.0)
    "};
    rua(source).unwrap();
}

#[test]
fn test_local_var() {
    init_log();
    let source = indoc! {r#"
        local a = "hello, local!"
        local b = a
        print(b)
        print(print)
        local print = print
        print "I'm local-print!"
    "#};
    rua(source).unwrap();
}

#[test]
fn test_assignment() {
    init_log();
    let source = indoc! {"
        local a = 456
        a = 123
        print(a)
        a = a
        print(a)
        a = g
        print(a)
        g = 123
        print(g)
        g = a
        print(g)
        g = g2
        print(g)
    "};
    rua(source).unwrap();
}
