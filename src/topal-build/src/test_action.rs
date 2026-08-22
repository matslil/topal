use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let arguments = env::args().skip(1).collect::<Vec<_>>();
    let [identity, output, input] = arguments.as_slice() else {
        eprintln!("expected identity, output, and input");
        std::process::exit(2);
    };
    let source_root = env::var_os("TOPAL_SOURCE_ROOT").expect("source root");
    let build_root = env::var_os("TOPAL_BUILD_ROOT").expect("build root");
    let input_text = fs::read_to_string(Path::new(&source_root).join(input)).expect("input");
    if input_text.contains("fail") {
        std::process::exit(3);
    }
    let output = Path::new(&build_root).join(output);
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent).expect("output parent");
    }
    fs::write(&output, format!("{identity}:{input_text}")).expect("output");
    let log = Path::new(&build_root).join("executed.log");
    let mut contents = fs::read_to_string(&log).unwrap_or_default();
    contents.push_str(identity);
    contents.push('\n');
    fs::write(log, contents).expect("log");
}
