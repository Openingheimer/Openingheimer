fn main() {
    let config = slint_build::CompilerConfiguration::new()
    .with_include_paths(vec!["images".into()])
    .with_style("material-dark".into());

    slint_build::compile_with_config("ui/app.slint", config).unwrap();
}
