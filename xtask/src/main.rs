// Copyright 2025 FastLabs Developers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! An xtask binary for managing workspace tasks.

use std::io::Write;
use std::io::stdin;
use std::io::stdout;
use std::path::Path;
use std::process::Command as StdCommand;

use clap::Parser;
use clap::Subcommand;

mod colors {
    pub const RED: &str = "\x1b[31m";
    pub const GREEN: &str = "\x1b[32m";
    pub const YELLOW: &str = "\x1b[33m";
    pub const BLUE: &str = "\x1b[34m";
    pub const RESET: &str = "\x1b[0m";
}

#[derive(Parser)]
struct Command {
    #[clap(subcommand)]
    sub: SubCommand,
}

impl Command {
    fn run(self) {
        match self.sub {
            SubCommand::Build(cmd) => cmd.run(),
            SubCommand::Bootstrap(cmd) => cmd.run(),
            SubCommand::Lint(cmd) => cmd.run(),
            SubCommand::Test(cmd) => cmd.run(),
        }
    }
}

#[derive(Subcommand)]
enum SubCommand {
    #[clap(about = "Compile workspace packages.")]
    Build(CommandBuild),
    #[clap(about = "Bootstrap a new project from this template.")]
    Bootstrap(CommandBootstrap),
    #[clap(about = "Run format and clippy checks.")]
    Lint(CommandLint),
    #[clap(about = "Run unit tests.")]
    Test(CommandTest),
}

#[derive(Parser)]
struct CommandBuild {
    #[arg(long, help = "Assert that `Cargo.lock` will remain unchanged.")]
    locked: bool,
}

impl CommandBuild {
    fn run(self) {
        run_command(make_build_cmd(self.locked));
    }
}

#[derive(Parser)]
struct CommandBootstrap {
    #[arg(long, value_parser=parse_project_name, help = "Name of the new project (e.g., my-awesome-project).")]
    project_name: Option<String>,

    #[arg(long, value_parser=parse_github_account, help = "GitHub username or organization (e.g., rust-lang).")]
    github_account: Option<String>,
}

impl CommandBootstrap {
    fn run(self) {
        bootstrap_project(self.project_name, self.github_account);
    }
}

#[derive(Parser)]
struct CommandTest {
    #[arg(long, help = "Run tests serially and do not capture output.")]
    no_capture: bool,
}

impl CommandTest {
    fn run(self) {
        run_command(make_test_cmd(self.no_capture, true, &[]));
    }
}

#[derive(Parser)]
#[clap(name = "lint")]
struct CommandLint {
    #[arg(long, help = "Automatically apply lint suggestions.")]
    fix: bool,
}

impl CommandLint {
    fn run(self) {
        run_command(make_clippy_cmd(self.fix));
        run_command(make_format_cmd(self.fix));
        run_command(make_taplo_cmd(self.fix));
        run_command(make_typos_cmd());
        run_command(make_hawkeye_cmd(self.fix));
    }
}

fn find_command(cmd: &str) -> StdCommand {
    match which::which(cmd) {
        Ok(exe) => {
            let mut cmd = StdCommand::new(exe);
            cmd.current_dir(env!("CARGO_WORKSPACE_DIR"));
            cmd
        }
        Err(err) => {
            panic!("{cmd} not found: {err}");
        }
    }
}

fn ensure_installed(bin: &str, crate_name: &str) {
    if which::which(bin).is_err() {
        let mut cmd = find_command("cargo");
        cmd.args(["install", crate_name]);
        run_command(cmd);
    }
}

fn run_command(mut cmd: StdCommand) {
    println!("{cmd:?}");
    let status = cmd.status().expect("failed to execute process");
    assert!(status.success(), "command failed: {status}");
}

fn make_build_cmd(locked: bool) -> StdCommand {
    let mut cmd = find_command("cargo");
    cmd.args([
        "build",
        "--workspace",
        "--all-features",
        "--tests",
        "--examples",
        "--benches",
        "--bins",
    ]);
    if locked {
        cmd.arg("--locked");
    }
    cmd
}

fn make_test_cmd(no_capture: bool, default_features: bool, features: &[&str]) -> StdCommand {
    let mut cmd = find_command("cargo");
    cmd.args(["test", "--workspace"]);
    if !default_features {
        cmd.arg("--no-default-features");
    }
    if !features.is_empty() {
        cmd.args(["--features", features.join(",").as_str()]);
    }
    if no_capture {
        cmd.args(["--", "--nocapture"]);
    }
    cmd
}

fn make_format_cmd(fix: bool) -> StdCommand {
    let mut cmd = find_command("cargo");
    cmd.args(["fmt", "--all"]);
    if !fix {
        cmd.arg("--check");
    }
    cmd
}

fn make_clippy_cmd(fix: bool) -> StdCommand {
    let mut cmd = find_command("cargo");
    cmd.args([
        "clippy",
        "--tests",
        "--all-features",
        "--all-targets",
        "--workspace",
    ]);
    if fix {
        cmd.args(["--allow-staged", "--allow-dirty", "--fix"]);
    } else {
        cmd.args(["--", "-D", "warnings"]);
    }
    cmd
}

fn make_hawkeye_cmd(fix: bool) -> StdCommand {
    ensure_installed("hawkeye", "hawkeye");
    let mut cmd = find_command("hawkeye");
    if fix {
        cmd.args(["format", "--fail-if-updated=false"]);
    } else {
        cmd.args(["check"]);
    }
    cmd
}

fn make_typos_cmd() -> StdCommand {
    ensure_installed("typos", "typos-cli");
    find_command("typos")
}

fn make_taplo_cmd(fix: bool) -> StdCommand {
    ensure_installed("taplo", "taplo-cli");
    let mut cmd = find_command("taplo");
    if fix {
        cmd.args(["format"]);
    } else {
        cmd.args(["format", "--check"]);
    }
    cmd
}

/// Validates a project name according to Cargo's naming conventions.
///
/// Adapted from Cargo's [`restricted_names`] validation.
///
/// [`restricted_names`]: https://github.com/rust-lang/cargo/blob/master/crates/cargo-util-schemas/src/restricted_names.rs
///
/// See also: <https://doc.rust-lang.org/cargo/reference/manifest.html#the-name-field>
fn parse_project_name(name: &str) -> Result<String, String> {
    let name = name.trim();

    if name.is_empty() {
        return Err("project name cannot be empty".into());
    }

    let mut chars = name.chars();
    if let Some(ch) = chars.next() {
        if ch.is_ascii_digit() {
            return Err(format!("the name cannot start with a digit: '{}'", ch));
        }
        if !(ch.is_ascii_alphabetic() || ch == '_') {
            return Err(format!(
                "the first character must be a letter or `_`, found: '{}'",
                ch
            ));
        }
    }

    for ch in chars {
        if !(ch.is_ascii_alphanumeric() || ch == '-' || ch == '_') {
            return Err(format!(
                "invalid character '{}': only letters, numbers, `-`, or `_` are allowed",
                ch
            ));
        }
    }

    Ok(name.to_owned())
}

fn parse_github_account(account_name: &str) -> Result<String, String> {
    let account_name = account_name.trim();
    if account_name.is_empty() {
        return Err("GitHub account name cannot be empty".into());
    }
    Ok(account_name.to_owned())
}

fn check_project_root() -> Result<(), String> {
    if !Path::new("Cargo.toml").exists() || !Path::new("xtask").is_dir() {
        return Err("This command must be run from the project root directory".into());
    }

    if !Path::new("template").is_dir() {
        return Err("The 'template' directory not found. \
             This project may have already been bootstrapped."
            .into());
    }
    Ok(())
}

fn prompt_input(prompt: &str) -> String {
    print!("{}: ", prompt);
    stdout().flush().unwrap();
    let mut input = String::new();
    stdin().read_line(&mut input).unwrap();
    input.trim().to_owned()
}

fn get_valid_input<F>(prompt: &str, validator: F) -> String
where
    F: Fn(&str) -> Result<String, String>,
{
    loop {
        let input = prompt_input(prompt);
        match validator(&input) {
            Ok(value) => return value,
            Err(e) => eprintln!("{}ERROR: {e}{}", colors::RED, colors::RESET),
        }
    }
}

fn bootstrap_project(project_name: Option<String>, github_account: Option<String>) {
    if let Err(e) = check_project_root() {
        eprintln!("{}ERROR: {e}{}", colors::RED, colors::RESET);
        return;
    }
    print_bootstrap_title();
    let Some((project_name, github_account)) = prepare_inputs(project_name, github_account) else {
        return;
    };
    if preview_and_confirm(&project_name, &github_account).is_none() {
        return;
    };
    execute_bootstrap(&project_name, &github_account);
    print_bootstrap_complete(&project_name);
}

fn prepare_inputs(
    project_name: Option<String>,
    github_account: Option<String>,
) -> Option<(String, String)> {
    let project_name = project_name
        .unwrap_or_else(|| get_valid_input("Enter the new project name", parse_project_name));
    let github_account = github_account
        .unwrap_or_else(|| get_valid_input("Enter the GitHub username/org", parse_github_account));
    Some((project_name, github_account))
}

fn preview_and_confirm(project_name: &str, github_account: &str) -> Option<()> {
    print_bootstrap_preview(project_name, github_account);
    confirm()
        .then(|| {
            println!(
                "\n{}Starting batch rename...{}\n",
                colors::BLUE,
                colors::RESET
            )
        })
        .or_else(|| {
            println!("{}Cancelled.{}", colors::YELLOW, colors::RESET);
            None
        })
}

fn execute_bootstrap(project_name: &str, github_account: &str) {
    update_root_cargo_toml(project_name, github_account);
    update_project_cargo_toml(project_name);
    update_readme(project_name, github_account);
    update_semantic_yml(project_name, github_account);
    update_cargo_lock(project_name);
    update_project_dir(project_name);
}

fn print_bootstrap_preview(project_name: &str, github_account: &str) {
    println!(
        "\n\
{blue}Preview:{reset}
  Project name:  {green}{project_name}{reset}
  GitHub repo:   {green}{github_account}/{project_name}{reset}
  Crates.io URL: {green}https://crates.io/crates/{project_name}{reset}
",
        blue = colors::BLUE,
        green = colors::GREEN,
        reset = colors::RESET,
        project_name = project_name,
        github_account = github_account,
    );
}

fn confirm() -> bool {
    print!("Continue? (y/N): ");
    stdout().flush().unwrap();

    let mut input = String::new();
    stdin().read_line(&mut input).unwrap();
    matches!(input.trim().to_lowercase().as_str(), "y" | "yes")
}

fn replace_in_file(file: &std::path::Path, old: &str, new: &str) -> Result<(), String> {
    let content = std::fs::read_to_string(file).map_err(|e| e.to_string())?;

    if !content.contains(old) {
        return Ok(());
    }
    let content = content.replace(old, new);

    std::fs::write(file, content).map_err(|e| e.to_string())
}

fn print_task(task: impl AsRef<str>) {
    print!("{:.<50}", task.as_ref());
}

fn print_update_result(result: Result<(), String>) {
    match result {
        Ok(_) => println!("{}[OK]{}", colors::GREEN, colors::RESET),
        Err(e) => eprintln!("{}[ERROR] {}{}", colors::RED, e, colors::RESET),
    }
}

fn update_root_cargo_toml(project_name: &str, github_account: &str) {
    let file = Path::new("Cargo.toml");
    print_task(format!("Updating {}...", file.display()));
    let result = replace_in_file(file, "/fast", &format!("/{}", github_account))
        .and_then(|_| replace_in_file(file, "template", project_name));

    print_update_result(result);
}

fn update_project_cargo_toml(project_name: &str) {
    let file = Path::new("template/Cargo.toml");
    print_task(format!("Updating {}...", file.display()));
    let result = replace_in_file(file, "template", project_name);
    print_update_result(result);
}

fn update_readme(project_name: &str, github_account: &str) {
    let file = Path::new("README.md");
    print_task(format!("Updating {}...", file.display()));
    let result = replace_in_file(file, "/fast", &format!("/{}", github_account))
        .and_then(|_| replace_in_file(file, "/template", &format!("/{}", project_name)));
    print_update_result(result);
}

fn update_semantic_yml(project_name: &str, github_account: &str) {
    let file = Path::new(".github/semantic.yml");
    print_task(format!("Updating {}...", file.display()));
    let result = replace_in_file(
        file,
        "/fast/template",
        &format!("/{}/{}", github_account, project_name),
    );
    print_update_result(result);
}

fn update_cargo_lock(project_name: &str) {
    let file = Path::new("Cargo.lock");
    print_task(format!("Updating {}...", file.display()));
    let result = replace_in_file(file, "template", project_name);
    print_update_result(result);
}

fn update_project_dir(project_name: &str) {
    print_task(format!(
        "Renaming \"template/\" directory to \"{}/\" ...",
        project_name
    ));
    let result =
        std::fs::rename(Path::new("template"), Path::new(project_name)).map_err(|e| e.to_string());
    print_update_result(result);
}

fn print_bootstrap_title() {
    println!(
        "\n\
{blue}========================================{reset}
{blue}     Template Project Bootstrapper      {reset}
{blue}========================================{reset}
",
        blue = colors::BLUE,
        reset = colors::RESET,
    );
}

fn print_bootstrap_complete(project_name: &str) {
    println!(
        "\n\
{green}========================================{reset}
{green}           Bootstrap completed!         {reset}
{green}========================================{reset}

{blue}Next steps:{reset}

1. Review the changes:
    {yellow}git diff{reset}

2. Update the project description in README.md

3. Commit your changes:
    {yellow}git add .{reset}
    {yellow}git commit -m \"chore: initialize project as {project_name}\"{reset}

4. Push to GitHub:
    {yellow}git push{reset}

{green}Happy coding!{reset}
",
        green = colors::GREEN,
        blue = colors::BLUE,
        yellow = colors::YELLOW,
        reset = colors::RESET,
        project_name = project_name
    );
}

fn main() {
    let cmd = Command::parse();
    cmd.run()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_project_name() {
        // valid names
        assert_eq!(parse_project_name("myproject"), Ok("myproject".into()));
        assert_eq!(parse_project_name("my-project"), Ok("my-project".into()));
        assert_eq!(parse_project_name("my_project"), Ok("my_project".into()));
        assert_eq!(parse_project_name("project123"), Ok("project123".into()));
        assert_eq!(parse_project_name("_private"), Ok("_private".into()));
        assert_eq!(parse_project_name("MyProject"), Ok("MyProject".into()));
        assert_eq!(parse_project_name("  myproject  "), Ok("myproject".into()));

        // invalid names
        assert!(parse_project_name("").is_err());
        assert!(parse_project_name("   ").is_err());
        assert!(parse_project_name("123project").is_err());
        assert!(parse_project_name("-project").is_err());
        assert!(parse_project_name("my@project").is_err());
        assert!(parse_project_name("my project").is_err());
        assert!(parse_project_name("my.project").is_err());
    }

    #[test]
    fn test_parse_github_account() {
        // valid accounts
        assert_eq!(parse_github_account("myuser"), Ok("myuser".into()));
        assert_eq!(parse_github_account("my-org"), Ok("my-org".into()));
        assert_eq!(parse_github_account("  myuser  "), Ok("myuser".into()));

        // invalid accounts
        assert!(parse_github_account("").is_err());
        assert!(parse_github_account("   ").is_err());
    }
}
