#!/usr/bin/env bash
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

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  Template Project Batch Renamer        ${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

if [[ ! -f "Cargo.toml" ]] || [[ ! -d "template" ]]; then
    echo -e "${RED}ERROR: This script must be run from the project root directory${NC}"
    exit 1
fi

echo -e "${YELLOW}Please provide the following information:${NC}"
echo ""

read -p "New project name (e.g., my-awesome-project): " PROJECT_NAME
read -p "GitHub username/org (e.g., myname): " GITHUB_USER

if [[ -z "$PROJECT_NAME" ]]; then
    echo -e "${RED}ERROR: Project name is required${NC}"
    exit 1
fi

if [[ -z "$GITHUB_USER" ]]; then
    echo -e "${RED}ERROR: GitHub username is required${NC}"
    exit 1
fi

# Validate project name format (Rust package naming convention)
if [[ ! "$PROJECT_NAME" =~ ^[a-z][a-z0-9_-]*$ ]]; then
    echo -e "${RED}ERROR: Project name must start with a lowercase letter and contain only lowercase letters, numbers, hyphens, and underscores${NC}"
    exit 1
fi

echo ""
echo -e "${BLUE}Summary:${NC}"
echo -e "  Project name:  ${GREEN}$PROJECT_NAME${NC}"
echo -e "  GitHub repo:   ${GREEN}$GITHUB_USER/$PROJECT_NAME${NC}"
echo -e "  Crates.io URL: ${GREEN}https://crates.io/crates/$PROJECT_NAME${NC}"
echo ""
read -p "Continue with renaming? (y/N): " CONFIRM

CONFIRM_LOWER=$(echo "$CONFIRM" | tr '[:upper:]' '[:lower:]')

if [[ "$CONFIRM_LOWER" != "y" ]] && [[ "$CONFIRM_LOWER" != "yes" ]]; then
    echo -e "${YELLOW}Cancelled.${NC}"
    exit 0
fi

echo ""
echo -e "${BLUE}Starting batch rename...${NC}"
echo ""

update_file() {
    local file=$1
    local old=$2
    local new=$3
    
    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS
        sed -i '' "s|$old|$new|g" "$file"
    else
        # Linux
        sed -i "s|$old|$new|g" "$file"
    fi
}

# 1. Update root Cargo.toml
echo -e "${GREEN}[OK]${NC} Updating Cargo.toml..."
update_file "Cargo.toml" "https://github.com/fast/template" "https://github.com/$GITHUB_USER/$PROJECT_NAME"
update_file "Cargo.toml" '"template"' "\"$PROJECT_NAME\""

# 2. Update template/Cargo.toml
echo -e "${GREEN}[OK]${NC} Updating template/Cargo.toml..."
update_file "template/Cargo.toml" 'name = "template"' "name = \"$PROJECT_NAME\""

# 3. Update README.md
echo -e "${GREEN}[OK]${NC} Updating README.md..."
update_file "README.md" "crates.io/crates/template" "crates.io/crates/$PROJECT_NAME"
update_file "README.md" "img.shields.io/crates/v/template.svg" "img.shields.io/crates/v/$PROJECT_NAME.svg"
update_file "README.md" "img.shields.io/crates/l/template" "img.shields.io/crates/l/$PROJECT_NAME"
update_file "README.md" "github.com/fast/template" "github.com/$GITHUB_USER/$PROJECT_NAME"
update_file "README.md" "docs.rs/template" "docs.rs/$PROJECT_NAME"

# 4. Update .github/semantic.yml
echo -e "${GREEN}[OK]${NC} Updating .github/semantic.yml..."
update_file ".github/semantic.yml" "github.com/fast/template" "github.com/$GITHUB_USER/$PROJECT_NAME"

# 5. Rename template directory
echo -e "${GREEN}[OK]${NC} Renaming template/ directory to $PROJECT_NAME/..."
if [[ -d "template" ]]; then
    mv template "$PROJECT_NAME"
fi

# 6. Update Cargo.lock
echo -e "${GREEN}[OK]${NC} Updating Cargo.lock..."
update_file "Cargo.lock" 'name = "template"' "name = \"$PROJECT_NAME\""

echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}  SUCCESS: Renaming completed!          ${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo -e "${BLUE}Next steps:${NC}"
echo ""
echo -e "  1. Review the changes:"
echo -e "     ${YELLOW}git diff${NC}"
echo ""
echo -e "  2. Delete the rename scripts (no longer needed):"
echo -e "     ${YELLOW}rm rename-project.sh rename-project.ps1${NC}"
echo ""
echo -e "  3. Update the project description in README.md"
echo ""
echo -e "  4. Commit your changes:"
echo -e "     ${YELLOW}git add .${NC}"
echo -e "     ${YELLOW}git commit -m \"chore: initialize project as $PROJECT_NAME\"${NC}"
echo ""
echo -e "  5. Push to GitHub:"
echo -e "     ${YELLOW}git push${NC}"
echo ""
echo -e "${GREEN}Happy coding!${NC}"

