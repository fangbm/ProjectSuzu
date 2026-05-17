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
$rootDocFiles = @(
    "README.md",
    "README.zh-CN.md",
    "CONTRIBUTING.md",
    "CONTRIBUTING.zh-CN.md",
    "SECURITY.md",
    "SECURITY.zh-CN.md",
    "LEGAL.md",
    "LEGAL.zh-CN.md",
    "LICENSE-MIT",
    "LICENSE-APACHE",
    "THIRD_PARTY_LICENSES.md",
    "THIRD_PARTY_LICENSES.zh-CN.md",
    "CHANGELOG.md",
    "CHANGELOG.zh-CN.md"
)
$brandingFiles = @(
    "assets/branding/Suzu_icon.png",
    "assets/branding/README.md",
    "assets/branding/README.zh-CN.md"
)
$docFiles = @(
    "docs/framework-guide.md",
    "docs/framework-guide.zh-CN.md",
    "docs/getting-started.md",
    "docs/getting-started.zh-CN.md",
    "docs/implementation-checklist.md",
    "docs/implementation-checklist.zh-CN.md",
    "docs/user-guide.md",
    "docs/user-guide.zh-CN.md",
    "docs/scripting-reference.md",
    "docs/scripting-reference.zh-CN.md",
    "docs/short-vn-demo.md",
    "docs/short-vn-demo.zh-CN.md",
    "docs/xp3-support.md",
    "docs/xp3-support.zh-CN.md",
    "docs/xp3-plugin-interface.md",
    "docs/xp3-plugin-interface.zh-CN.md",
    "docs/api-stability.md",
    "docs/api-stability.zh-CN.md",
    "docs/visual-script-editor-development-plan.md",
    "docs/visual-script-editor-development-plan.zh-CN.md",
    "docs/project-plan.md",
    "docs/project-plan.zh-CN.md",
    "docs/release-packaging.md",
    "docs/release-packaging.zh-CN.md",
    "docs/developer-checks.md",
    "docs/developer-checks.zh-CN.md",
    "docs/release-checklist.md",
    "docs/release-checklist.zh-CN.md"
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
    "suzu-ui-save-load-demo",
    "suzu-short-vn-demo"
)

Push-Location $repo
try {
    if ($Check) {
        foreach ($path in ($rootDocFiles + $brandingFiles + @("templates/minimal-vn", "examples/short-vn-demo", $AssetRoot) + $docFiles)) {
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
    New-Item -ItemType Directory -Force (Join-Path $dist "templates") | Out-Null

    foreach ($binary in $binaries) {
        Copy-Item (Join-Path $targetDir "$binary$exeSuffix") (Join-Path $dist "$binary$exeSuffix")
    }

    foreach ($rootDocFile in $rootDocFiles) {
        Copy-Item $rootDocFile (Join-Path $dist $rootDocFile)
    }
    foreach ($brandingFile in $brandingFiles) {
        Copy-Item $brandingFile (Join-Path $dist $brandingFile)
    }
    foreach ($docFile in $docFiles) {
        Copy-Item $docFile (Join-Path $dist $docFile)
    }
    Copy-Item "templates/minimal-vn" (Join-Path $dist "templates/minimal-vn") -Recurse

    cargo run -p suzu-packer -- $AssetRoot --pack (Join-Path $dist "assets/hello-world.suzupack")
    cargo run -p suzu-packer -- $AssetRoot --output (Join-Path $dist "assets/hello-world-assets.json")
    cargo run -p suzu-packer -- examples/short-vn-demo --pack (Join-Path $dist "assets/short-vn-demo.suzupack")
    cargo run -p suzu-packer -- examples/short-vn-demo --output (Join-Path $dist "assets/short-vn-demo-assets.json")

    Compress-Archive -Path (Join-Path $dist "*") -DestinationPath "$dist.zip" -Force
    Write-Host "Packaged desktop release at $dist.zip"
}
finally {
    Pop-Location
}
