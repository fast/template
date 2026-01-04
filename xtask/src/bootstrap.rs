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

use std::error::Error;
use std::path::Path;

use colored::Colorize;
use dialoguer::Confirm;
use dialoguer::Input;
use toml_edit::DocumentMut;

use super::workspace_dir;

pub fn bootstrap(cleanup: bool) {
    if cleanup {
        cleanup_bootstrap();
    } else {
        bootstrap_project();
    }
}

fn bootstrap_project() {
    println!("\n{}", "🚀 Starting project bootstrap...".yellow().bold());

    let project_name = get_valid_input(
        "Enter your project name (e.g., my-awesome-project)",
        parse_project_name,
    );
    let github_username = get_valid_input(
        "Enter your GitHub username (e.g., tisonkun)",
        parse_github_username,
    );

    let confirmation = Confirm::new()
        .with_prompt(
            format!("Bootstrap project '{project_name}' for user '{github_username}'?")
                .blue()
                .to_string(),
        )
        .default(false)
        .interact()
        .unwrap();

    if !confirmation {
        println!("\n{}", "Cancelled.".yellow());
        return;
    }

    println!("\n{}", "Bootstrapping...".cyan());
    execute_bootstrap(&project_name, &github_username);

    println!("\n{}", "🎉 Bootstrap complete!".green().bold());
    println!(
        "   {}: {}",
        "You can now delete this script".dimmed(),
        "cargo x bootstrap --cleanup".cyan().bold(),
    );
}

fn cleanup_bootstrap() {
    println!("\n{}", "🧹 Starting bootstrap cleanup...".yellow().bold());
    remove_ci_workflows();
    override_bootstrap_file();
    cleanup_cargo_toml();
    println!("\n{}", "🧹 Bootstrap cleanup complete!".green().bold());
}

fn remove_ci_workflows() {
    let workflows_dir = workspace_dir().join(".github/workflows/ci-bootstrap.yml");
    if workflows_dir.exists() {
        println!("Removing CI Bootstrap workflows...");
        std::fs::remove_dir_all(workflows_dir).unwrap();
    } else {
        panic!("Broken bootstrap cleanup state: '.github/workflows/ci-bootstrap.yml' not found");
    }
}

fn override_bootstrap_file() {
    let old_bootstrap_file = workspace_dir().join("xtask/src/bootstrap.rs");
    let new_bootstrap_file = workspace_dir().join("xtask/src/bootstrap-done.rs");
    if new_bootstrap_file.exists() {
        println!("Overriding bootstrap file...");
        std::fs::rename(new_bootstrap_file, old_bootstrap_file).unwrap();
    } else {
        panic!("Broken bootstrap cleanup state: 'bootstrap-done.rs' not found");
    }
}

fn cleanup_cargo_toml() {
    let cargo_toml = workspace_dir().join("xtask/Cargo.toml");

    let content = std::fs::read_to_string(&cargo_toml).unwrap();
    let mut doc = content.parse::<DocumentMut>().unwrap();
    if let Some(dependencies) = doc.get_mut("dependencies").and_then(|d| d.as_table_mut()) {
        println!("Removing unnecessary dependencies...");
        dependencies.remove("toml_edit");
        dependencies.remove("colored");
        dependencies.remove("dialoguer");
        std::fs::write(&cargo_toml, doc.to_string()).unwrap();
    } else {
        panic!("Broken bootstrap cleanup state: 'dependencies' section not found");
    }
}

/// Validates a project name according to Cargo's naming conventions.
///
/// Adapted from Cargo's [`restricted_names`] validation.
///
/// [`restricted_names`]: https://github.com/rust-lang/cargo/blob/master/crates/cargo-util-schemas/src/restricted_names.rs
///
/// See also: <https://doc.rust-lang.org/cargo/reference/manifest.html#the-name-field>
pub fn parse_project_name(name: &str) -> Result<String, String> {
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

pub fn parse_github_username(account_name: &str) -> Result<String, String> {
    let account_name = account_name.trim();
    if account_name.is_empty() {
        return Err("GitHub account name cannot be empty".into());
    }
    Ok(account_name.to_owned())
}

fn get_valid_input<F>(prompt: &str, validator: F) -> String
where
    F: Fn(&str) -> Result<String, String>,
{
    loop {
        let input: String = Input::new().with_prompt(prompt).interact_text().unwrap();
        match validator(&input) {
            Ok(value) => return value,
            Err(e) => eprintln!("{}", format!("ERROR: {e}").red()),
        }
    }
}

fn execute_bootstrap(project_name: &str, github_username: &str) {
    update_readme(project_name, github_username);
    update_root_cargo_toml(project_name, github_username);
    update_template_cargo_toml(project_name);
    update_semantic_yml(project_name, github_username);
    update_cargo_lock(project_name);
    update_project_dir(project_name);
}

fn replace_in_file(file: &Path, old: &str, new: &str) -> Result<(), Box<dyn Error>> {
    let content = std::fs::read_to_string(file)?;

    if !content.contains(old) {
        return Ok(());
    }
    let content = content.replace(old, new);

    std::fs::write(file, content)?;
    Ok(())
}

fn print_task(task: impl AsRef<str>) {
    print!("{:.<60}", task.as_ref());
}

fn print_update_result(result: Result<(), Box<dyn Error>>) {
    match result {
        Ok(_) => println!("{}", "[OK]".green()),
        Err(e) => println!("{}", format!("[ERROR] {e}").red()),
    }
}

fn update_readme(project_name: &str, github_username: &str) {
    let file = workspace_dir().join("README.md");
    print_task(format!("Updating {}...", file.display()));
    let result = replace_in_file(
        &file,
        "fast/template",
        &format!("{}/{}", github_username, project_name),
    )
    .and_then(|_| replace_in_file(&file, "${projectName}", project_name));
    print_update_result(result);
}

fn update_root_cargo_toml(project_name: &str, github_username: &str) {
    let file = workspace_dir().join("Cargo.toml");
    print_task(format!("Updating {}...", file.display()));
    let result = replace_in_file(
        &file,
        "fast/template",
        &format!("{}/{}", github_username, project_name),
    )
    .and_then(|_| replace_in_file(&file, "template", project_name));

    print_update_result(result);
}

fn update_template_cargo_toml(project_name: &str) {
    let file = workspace_dir().join("template/Cargo.toml");
    print_task(format!("Updating {}...", file.display()));
    let result = replace_in_file(&file, "template", project_name);
    print_update_result(result);
}

fn update_semantic_yml(project_name: &str, github_username: &str) {
    let file = workspace_dir().join(".github/semantic.yml");
    print_task(format!("Updating {}...", file.display()));
    let result = replace_in_file(
        &file,
        "fast/template",
        &format!("{}/{}", github_username, project_name),
    );
    print_update_result(result);
}

fn update_cargo_lock(project_name: &str) {
    let file = workspace_dir().join("Cargo.lock");
    print_task(format!("Updating {}...", file.display()));
    let result = replace_in_file(&file, "template", project_name);
    print_update_result(result);
}

fn update_project_dir(project_name: &str) {
    print_task(format!(
        "Renaming directory \"template\" to \"{project_name}\" ..."
    ));
    let template_dir = Path::new(env!("CARGO_WORKSPACE_DIR")).join("template");
    let target_dir = Path::new(env!("CARGO_WORKSPACE_DIR")).join(project_name);
    let result = if target_dir.exists() {
        Err(format!("Directory '{project_name}' already exists").into())
    } else {
        std::fs::rename(template_dir, target_dir).map_err(|e| e.into())
    };
    print_update_result(result);
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
    fn test_parse_github_username() {
        // valid accounts
        assert_eq!(parse_github_username("my-user"), Ok("my-user".into()));
        assert_eq!(parse_github_username("my-org"), Ok("my-org".into()));
        assert_eq!(parse_github_username("  my-user  "), Ok("my-user".into()));

        // invalid accounts
        assert!(parse_github_username("").is_err());
        assert!(parse_github_username("   ").is_err());
    }
}
