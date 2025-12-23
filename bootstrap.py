#!/usr/bin/env python3

# Copyright 2025 FastLabs Developers
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

import os
import sys

def main():
    print("Welcome to the project bootstrap script!")

    # 1. Get user input
    try:
        project_name = input("Enter your project name (e.g., my-awesome-project): ").strip()
        if not project_name:
            print("Error: Project name cannot be empty.")
            sys.exit(1)

        github_username = input("Enter your GitHub username (e.g., torvalds): ").strip()
        if not github_username:
            print("Error: GitHub username cannot be empty.")
            sys.exit(1)
    except KeyboardInterrupt:
        print("\nOperation cancelled.")
        sys.exit(0)

    print(f"\nBootstrapping project '{project_name}' for user '{github_username}'...\n")

    # 2. Update README.md
    # Replaces:
    # - fast/template -> username/project_name
    # - ${projectName} -> project_name
    readme_path = "README.md"
    if os.path.exists(readme_path):
        with open(readme_path, "r", encoding="utf-8") as f:
            content = f.read()

        new_content = content.replace("fast/template", f"{github_username}/{project_name}")
        new_content = new_content.replace("${projectName}", project_name)

        if content != new_content:
            with open(readme_path, "w", encoding="utf-8") as f:
                f.write(new_content)
            print(f"✅ Updated {readme_path}")
        else:
            print(f"ℹ️  No changes needed in {readme_path}")
    else:
        print(f"⚠️  Warning: {readme_path} not found.")

    # 3. Update Cargo.toml (Workspace Root)
    # Replaces:
    # - fast/template -> username/project_name
    # - "template" (in members) -> "project_name"
    root_cargo_path = "Cargo.toml"
    if os.path.exists(root_cargo_path):
        with open(root_cargo_path, "r", encoding="utf-8") as f:
            content = f.read()

        new_content = content.replace("fast/template", f"{github_username}/{project_name}")
        # Identify workspace member "template" specifically to avoid false positives
        new_content = new_content.replace('"template"', f'"{project_name}"')

        if content != new_content:
            with open(root_cargo_path, "w", encoding="utf-8") as f:
                f.write(new_content)
            print(f"✅ Updated {root_cargo_path}")
        else:
            print(f"ℹ️  No changes needed in {root_cargo_path}")
    else:
        print(f"⚠️  Warning: {root_cargo_path} not found.")

    # 4. Update template/Cargo.toml (Package Name)
    # Replaces:
    # - name = "template" -> name = "project_name"
    # Note: We edit the file inside the directory *before* renaming the directory
    template_cargo_path = "template/Cargo.toml"
    if os.path.exists(template_cargo_path):
        with open(template_cargo_path, "r", encoding="utf-8") as f:
            content = f.read()

        new_content = content.replace('name = "template"', f'name = "{project_name}"')

        if content != new_content:
            with open(template_cargo_path, "w", encoding="utf-8") as f:
                f.write(new_content)
            print(f"✅ Updated {template_cargo_path}")
        else:
            print(f"ℹ️  No changes needed in {template_cargo_path}")
    else:
        # If the directory was already renamed in a previous run, we might want to check the new name
        # but for a simple bootstrap script, assuming standard state is fine.
        print(f"⚠️  Warning: {template_cargo_path} not found (Did you already run this script?)")

    # 5. Rename template directory
    if os.path.exists("template"):
        os.rename("template", project_name)
        print(f"✅ Renamed directory 'template' to '{project_name}'")
    else:
        if os.path.exists(project_name):
             print(f"ℹ️  Directory '{project_name}' already exists.")
        else:
             print("⚠️  Warning: Directory 'template' not found.")

    print("\n🎉 Bootstrap complete!")
    print(f"You can now delete this script: rm {os.path.basename(__file__)}")

if __name__ == "__main__":
    main()
