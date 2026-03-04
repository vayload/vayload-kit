#[cfg(test)]
mod tests {
    const BIN: &str = "target/debug/vk";
    use assert_cmd::prelude::*;
    use predicates::prelude::*;
    use std::process::Command;

    #[test]
    fn test_help_command() {
        let mut cmd = Command::new(BIN);
        cmd.arg("--help");
        cmd.assert().success().stdout(predicate::str::contains("Vayload Kit (vk)"));
    }

    #[test]
    fn test_version_command() {
        let mut cmd = Command::new(BIN);
        cmd.arg("--version");
        cmd.assert().success().stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
    }

    #[test]
    fn test_install_without_manifest_fails() {
        let mut cmd = Command::new(BIN);
        cmd.arg("install");
        cmd.assert().failure();
    }
}
