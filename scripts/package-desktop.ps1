param(
    [string]$Configuration = "release",
    [string]$Output = "dist/project-suzu-desktop",
    [string]$AssetRoot = "examples/hello-world",
    [switch]$Check
)

$ErrorActionPreference = "Stop"

$repo = Split-Path -Parent $PSScriptRoot
$targetDir = Join-Path $repo "target/$Configuration"
$dist = Join-Path $repo $Output
$distParent = Split-Path -Parent $dist
$exeSuffix = if ($IsWindows -or $env:OS -eq "Windows_NT") { ".exe" } else { "" }
$docFiles = @(
    "docs/implementation-checklist.md",
    "docs/user-guide.md",
    "docs/scripting-reference.md",
    "docs/visual-script-editor-development-plan.md",
    "docs/release-packaging.md",
    "docs/developer-checks.md",
    "docs/release-checklist.md"
)
$binaries = @(
    "suzu-compiler",
    "suzu-packer",
    "suzu-bench",
    "suzu-editor",
    "suzu-launcher",
    "suzu-xp3-viewer",
    "suzu-hello-world",
    "suzu-branching-story",
    "suzu-ui-save-load-demo"
)

Push-Location $repo
try {
    if ($Check) {
        foreach ($path in @("README.md", "README.zh-CN.md", "CONTRIBUTING.md", "SECURITY.md", "LICENSE-MIT", "LICENSE-APACHE", "CHANGELOG.md", "assets/branding/Suzu_icon.png", $AssetRoot) + $docFiles) {
            if (-not (Test-Path $path)) {
                throw "Missing package input: $path"
            }
        }
        Write-Host "Package inputs are present."
        return
    }

    cargo build --workspace --profile $Configuration

    $resolvedRepo = (Resolve-Path $repo).Path
    if (-not (Test-Path $distParent)) {
        New-Item -ItemType Directory -Force $distParent | Out-Null
    }
    $resolvedParent = (Resolve-Path $distParent).Path
    if (-not $resolvedParent.StartsWith($resolvedRepo)) {
        throw "Output must stay inside the repository: $dist"
    }

    if (Test-Path $dist) {
        Remove-Item -Recurse -Force $dist
    }
    New-Item -ItemType Directory -Force $dist | Out-Null
    New-Item -ItemType Directory -Force (Join-Path $dist "assets") | Out-Null
    New-Item -ItemType Directory -Force (Join-Path $dist "assets/branding") | Out-Null
    New-Item -ItemType Directory -Force (Join-Path $dist "docs") | Out-Null

    foreach ($binary in $binaries) {
        Copy-Item (Join-Path $targetDir "$binary$exeSuffix") (Join-Path $dist "$binary$exeSuffix")
    }

    Copy-Item "README.md" (Join-Path $dist "README.md")
    Copy-Item "README.zh-CN.md" (Join-Path $dist "README.zh-CN.md")
    Copy-Item "CONTRIBUTING.md" (Join-Path $dist "CONTRIBUTING.md")
    Copy-Item "SECURITY.md" (Join-Path $dist "SECURITY.md")
    Copy-Item "LICENSE-MIT" (Join-Path $dist "LICENSE-MIT")
    Copy-Item "LICENSE-APACHE" (Join-Path $dist "LICENSE-APACHE")
    Copy-Item "CHANGELOG.md" (Join-Path $dist "CHANGELOG.md")
    Copy-Item "assets/branding/Suzu_icon.png" (Join-Path $dist "assets/branding/Suzu_icon.png")
    foreach ($docFile in $docFiles) {
        Copy-Item $docFile (Join-Path $dist $docFile)
    }

    cargo run -p suzu-packer -- $AssetRoot --pack (Join-Path $dist "assets/hello-world.suzupack")
    cargo run -p suzu-packer -- $AssetRoot --output (Join-Path $dist "assets/hello-world-assets.json")

    Compress-Archive -Path (Join-Path $dist "*") -DestinationPath "$dist.zip" -Force
    Write-Host "Packaged desktop release at $dist.zip"
}
finally {
    Pop-Location
}
