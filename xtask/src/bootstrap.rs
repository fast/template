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
use std::path::PathBuf;

use colored::Colorize;
use dialoguer::Confirm;
use dialoguer::Input;

fn workspace_dir() -> &'static Path {
    Path::new(env!("CARGO_WORKSPACE_DIR"))
}

fn workspace_path(relative: &str) -> PathBuf {
    workspace_dir().join(relative)
}

pub fn bootstrap_project() {
    println!("\n{}", "🚀 Starting project bootstrap...".yellow().bold());

    let project_name = get_valid_input(
        "Enter your project name (e.g., my-awesome-project)",
        parse_project_name,
    );
    let github_username = get_valid_input(
        "Enter your GitHub username (e.g., torvalds)",
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

pub fn cleanup_bootstrap() {
    println!("\n{}", "🧹 Starting bootstrap cleanup...".yellow().bold());
    let bootstrap_file = workspace_path("xtask/src/bootstrap.rs");
    let cargo_toml = workspace_path("xtask/Cargo.toml");
    remove_bootstrap_file(&bootstrap_file);
    cleanup_cargo_toml(&cargo_toml).unwrap();
    println!("\n{}", "🧹 Bootstrap cleanup complete!".green().bold());
}

fn remove_bootstrap_file(bootstrap_file: &Path) {
    if bootstrap_file.exists() {
        println!("Deleting bootstrap.rs...");
        std::fs::remove_file(bootstrap_file).unwrap();
    } else {
        println!("{}", "bootstrap.rs already deleted".dimmed());
    }
}

fn cleanup_cargo_toml(cargo_toml_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    use toml_edit::DocumentMut;

    let content = std::fs::read_to_string(cargo_toml_path)?;
    let mut doc = content.parse::<DocumentMut>()?;

    disable_bootstrap_feature(&mut doc);
    remove_bootstrap_dependencies(&mut doc);

    std::fs::write(cargo_toml_path, doc.to_string())?;
    Ok(())
}

fn disable_bootstrap_feature(doc: &mut toml_edit::DocumentMut) {
    if let Some(features) = doc.get_mut("features").and_then(|f| f.as_table_mut()) {
        println!("Disabling bootstrap feature...");
        if let Some(default) = features.get_mut("default").and_then(|d| d.as_array_mut()) {
            let index_to_remove = default
                .iter()
                .position(|feature| feature.as_str() == Some("bootstrap"));
            if let Some(idx) = index_to_remove {
                default.remove(idx);
            }
        }
    }
}

fn remove_bootstrap_dependencies(doc: &mut toml_edit::DocumentMut) {
    if let Some(dependencies) = doc.get_mut("dependencies").and_then(|d| d.as_table_mut()) {
        println!("Removing unnecessary dependencies...");
        dependencies.remove("toml_edit");
        dependencies.remove("colored");
        dependencies.remove("dialoguer");
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

fn replace_in_file(file: &std::path::Path, old: &str, new: &str) -> Result<(), Box<dyn Error>> {
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
    let file = workspace_path("README.md");
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
    let file = workspace_path("Cargo.toml");
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
    let file = workspace_path("template/Cargo.toml");
    print_task(format!("Updating {}...", file.display()));
    let result = replace_in_file(&file, "template", project_name);
    print_update_result(result);
}

fn update_semantic_yml(project_name: &str, github_username: &str) {
    let file = workspace_path(".github/semantic.yml");
    print_task(format!("Updating {}...", file.display()));
    let result = replace_in_file(
        &file,
        "fast/template",
        &format!("{}/{}", github_username, project_name),
    );
    print_update_result(result);
}

fn update_cargo_lock(project_name: &str) {
    let file = workspace_path("Cargo.lock");
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
        assert_eq!(parse_github_username("myuser"), Ok("myuser".into()));
        assert_eq!(parse_github_username("my-org"), Ok("my-org".into()));
        assert_eq!(parse_github_username("  myuser  "), Ok("myuser".into()));

        // invalid accounts
        assert!(parse_github_username("").is_err());
        assert!(parse_github_username("   ").is_err());
    }
}
