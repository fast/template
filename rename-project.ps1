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

$ErrorActionPreference = "Stop"

Write-Host "========================================" -ForegroundColor Blue
Write-Host "  Template Project Batch Renamer        " -ForegroundColor Blue
Write-Host "========================================" -ForegroundColor Blue
Write-Host ""

if (-not (Test-Path "Cargo.toml") -or -not (Test-Path "template" -PathType Container)) {
    Write-Host "ERROR: This script must be run from the project root directory" -ForegroundColor Red
    exit 1
}

Write-Host "Please provide the following information:" -ForegroundColor Yellow
Write-Host ""

$ProjectName = Read-Host "New project name (e.g., my-awesome-project)"
$GitHubUser = Read-Host "GitHub username/org (e.g., myname)"

if ([string]::IsNullOrWhiteSpace($ProjectName)) {
    Write-Host "ERROR: Project name is required" -ForegroundColor Red
    exit 1
}

if ([string]::IsNullOrWhiteSpace($GitHubUser)) {
    Write-Host "ERROR: GitHub username is required" -ForegroundColor Red
    exit 1
}

# Validate project name format (Rust package naming convention)
if ($ProjectName -notmatch '^[a-z][a-z0-9_-]*$') {
    Write-Host "ERROR: Project name must start with a lowercase letter and contain only lowercase letters, numbers, hyphens, and underscores" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "Summary:" -ForegroundColor Blue
Write-Host "  Project name:  $ProjectName" -ForegroundColor Green
Write-Host "  GitHub repo:   $GitHubUser/$ProjectName" -ForegroundColor Green
Write-Host "  Crates.io URL: https://crates.io/crates/$ProjectName" -ForegroundColor Green
Write-Host ""

$Confirm = Read-Host "Continue with renaming? (y/N)"
$ConfirmLower = $Confirm.ToLower()
if ($ConfirmLower -ne "y" -and $ConfirmLower -ne "yes") {
    Write-Host "Cancelled." -ForegroundColor Yellow
    exit 0
}

Write-Host ""
Write-Host "Starting batch rename..." -ForegroundColor Blue
Write-Host ""

function Update-FileContent {
    param(
        [string]$FilePath,
        [string]$OldValue,
        [string]$NewValue
    )
    
    $content = Get-Content $FilePath -Raw
    $content = $content -replace [regex]::Escape($OldValue), $NewValue
    Set-Content $FilePath -Value $content -NoNewline
}

# 1. Update root Cargo.toml
Write-Host "[OK] Updating Cargo.toml..." -ForegroundColor Green
Update-FileContent "Cargo.toml" "https://github.com/fast/template" "https://github.com/$GitHubUser/$ProjectName"
Update-FileContent "Cargo.toml" '"template"' "`"$ProjectName`""

# 2. Update template/Cargo.toml
Write-Host "[OK] Updating template/Cargo.toml..." -ForegroundColor Green
Update-FileContent "template/Cargo.toml" 'name = "template"' "name = `"$ProjectName`""

# 3. Update README.md
Write-Host "[OK] Updating README.md..." -ForegroundColor Green
Update-FileContent "README.md" "crates.io/crates/template" "crates.io/crates/$ProjectName"
Update-FileContent "README.md" "img.shields.io/crates/v/template.svg" "img.shields.io/crates/v/$ProjectName.svg"
Update-FileContent "README.md" "img.shields.io/crates/l/template" "img.shields.io/crates/l/$ProjectName"
Update-FileContent "README.md" "github.com/fast/template" "github.com/$GitHubUser/$ProjectName"
Update-FileContent "README.md" "docs.rs/template" "docs.rs/$ProjectName"

# 4. Update .github/semantic.yml
Write-Host "[OK] Updating .github/semantic.yml..." -ForegroundColor Green
Update-FileContent ".github/semantic.yml" "github.com/fast/template" "github.com/$GitHubUser/$ProjectName"

# 5. Rename template directory
Write-Host "[OK] Renaming template/ directory to $ProjectName/..." -ForegroundColor Green
if (Test-Path "template" -PathType Container) {
    Rename-Item "template" $ProjectName
}

# 6. Update Cargo.lock
Write-Host "[OK] Updating Cargo.lock..." -ForegroundColor Green
Update-FileContent "Cargo.lock" 'name = "template"' "name = `"$ProjectName`""

Write-Host ""
Write-Host "========================================" -ForegroundColor Green
Write-Host "  SUCCESS: Renaming completed!          " -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Blue
Write-Host ""
Write-Host "  1. Review the changes:"
Write-Host "     git diff" -ForegroundColor Yellow
Write-Host ""
Write-Host "  2. Delete the rename scripts (no longer needed):"
Write-Host "     Remove-Item rename-project.sh, rename-project.ps1" -ForegroundColor Yellow
Write-Host ""
Write-Host "  3. Update the project description in README.md"
Write-Host ""
Write-Host "  4. Commit your changes:"
Write-Host "     git add ." -ForegroundColor Yellow
Write-Host "     git commit -m `"chore: initialize project as $ProjectName`"" -ForegroundColor Yellow
Write-Host ""
Write-Host "  5. Push to GitHub:"
Write-Host "     git push" -ForegroundColor Yellow
Write-Host ""
Write-Host "Happy coding!" -ForegroundColor Green

