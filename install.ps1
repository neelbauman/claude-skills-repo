#Requires -Version 5.1
Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

# ── Configuration ────────────────────────────────────────────────
$Repo       = "neelbauman/claude-skills-repo"              # TODO: Replace with actual GitHub owner/repo
$BinName    = "claude-registry"
$InstallDir = Join-Path $env:USERPROFILE ".local\bin"

# ── Architecture detection ───────────────────────────────────────
function Get-Arch {
    switch ($env:PROCESSOR_ARCHITECTURE) {
        "AMD64" { return "x86_64" }
        "ARM64" { return "aarch64" }
        default {
            Write-Error "Unsupported architecture: $env:PROCESSOR_ARCHITECTURE"
            exit 1
        }
    }
}

# ── Install from GitHub Releases ─────────────────────────────────
function Install-FromRelease {
    param([string]$Arch)

    $url = "https://github.com/$Repo/releases/latest/download/$BinName-windows-$Arch.zip"
    $tmpDir = Join-Path ([System.IO.Path]::GetTempPath()) ([System.IO.Path]::GetRandomFileName())
    New-Item -ItemType Directory -Path $tmpDir -Force | Out-Null

    try {
        $zipPath = Join-Path $tmpDir "$BinName.zip"

        Write-Host "Downloading $BinName from GitHub Releases..."
        Write-Host "  URL: $url"

        [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
        Invoke-WebRequest -Uri $url -OutFile $zipPath -UseBasicParsing

        Expand-Archive -Path $zipPath -DestinationPath $tmpDir -Force

        if (-not (Test-Path $InstallDir)) {
            New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
        }

        $exeName = "$BinName.exe"
        Copy-Item (Join-Path $tmpDir $exeName) (Join-Path $InstallDir $exeName) -Force

        Write-Host "Installed $BinName to $InstallDir\$exeName"
        return $true
    }
    catch {
        Write-Host "Download failed: $_" -ForegroundColor Yellow
        return $false
    }
    finally {
        Remove-Item -Recurse -Force $tmpDir -ErrorAction SilentlyContinue
    }
}

# ── Fallback: build from source ──────────────────────────────────
function Install-FromSource {
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        Write-Error "cargo is not installed. Install Rust first: https://rustup.rs"
        exit 1
    }
    if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
        Write-Error "git is not installed."
        exit 1
    }

    $tmpDir = Join-Path ([System.IO.Path]::GetTempPath()) ([System.IO.Path]::GetRandomFileName())
    New-Item -ItemType Directory -Path $tmpDir -Force | Out-Null

    try {
        Write-Host "Building $BinName from source..."

        git clone "https://github.com/$Repo.git" (Join-Path $tmpDir "repo")
        Push-Location (Join-Path $tmpDir "repo" "cli")
        try {
            cargo build --release
        }
        finally {
            Pop-Location
        }

        if (-not (Test-Path $InstallDir)) {
            New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
        }

        $exeName = "$BinName.exe"
        $builtExe = Join-Path $tmpDir "repo" "cli" "target" "release" $exeName
        Copy-Item $builtExe (Join-Path $InstallDir $exeName) -Force

        Write-Host "Installed $BinName to $InstallDir\$exeName"
    }
    finally {
        Remove-Item -Recurse -Force $tmpDir -ErrorAction SilentlyContinue
    }
}

# ── Main ─────────────────────────────────────────────────────────
function Main {
    Write-Host "Installing $BinName..."
    Write-Host ""

    $arch = Get-Arch
    Write-Host "Detected architecture: $arch"

    $installed = Install-FromRelease -Arch $arch

    if (-not $installed) {
        Write-Host ""
        Write-Host "Pre-built binary not available. Falling back to source build..."
        Install-FromSource
    }

    Write-Host ""

    # PATH check
    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($userPath -notlike "*$InstallDir*") {
        Write-Host "NOTE: $InstallDir is not in your PATH." -ForegroundColor Yellow
        Write-Host "Run the following to add it permanently:"
        Write-Host ""
        Write-Host "  [Environment]::SetEnvironmentVariable('Path', `"$InstallDir;`$env:Path`", 'User')"
        Write-Host ""
    }

    Write-Host "Done! Run '$BinName --help' to get started."
}

Main
