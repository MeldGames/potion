pub fn write_obj() -> io::Result<()> {}

pub fn convex_decomposition(file: Path) -> Result<()> {
    if cfg!(target_os = "windows") {
        Command::new("TestVHACD")
            .args([])
            .output()
            .expect("failed to execute process")
    } else {
        Command::new("sh")
            .arg("-c")
            .arg("echo hello")
            .output()
            .expect("failed to execute process")
    };
}
