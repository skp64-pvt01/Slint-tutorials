fn main() {
    // Debug info lets the testing backend find elements by their accessible
    // label. The release binary is not affected.
    let config = slint_build::CompilerConfiguration::new().with_debug_info(true);
    slint_build::compile_with_config("ui/main.slint", config).expect("the Slint UI compiles");
}
