# Stop execution on any PowerShell command errors
$ErrorActionPreference = "Stop"

$root = Get-Location

# 1. Load environment
$envScript = Join-Path $root "aether_db_env.ps1"
if (Test-Path $envScript) {
    . $envScript
} else {
    Write-Error "Environment script not found at $envScript"
}

$corePath = Join-Path $root "packages/core/aether-core"
$desktopPath = Join-Path $root "apps/desktop/src-tauri"
$migrationsLinksDir = Join-Path $desktopPath "migrations_links"

# Helper to check exit codes of external commands (cargo, sqlx, etc.)
function Check-LastExit($CommandName) {
    if ($LASTEXITCODE -ne 0) {
        Write-Host "`nError: $CommandName failed with exit code $LASTEXITCODE" -ForegroundColor Red
        exit $LASTEXITCODE
    }
}

Write-Host "`n--- Rebuilding Migrations ---" -ForegroundColor Yellow

# 2. Create hard links
if (Test-Path $migrationsLinksDir) { Remove-Item -Force -Recurse $migrationsLinksDir }
New-Item -ItemType Directory -Path $migrationsLinksDir | Out-Null

function New-MigrationLink($sourcePath, $targetDir) {
    if (Test-Path $sourcePath) {
        Get-ChildItem $sourcePath -Filter *.sql | ForEach-Object {
            $linkPath = Join-Path $targetDir $_.Name
            if (-not (Test-Path $linkPath)) {
                # Create hard links to avoid duplication and keep migrations in sync
                New-Item -ItemType HardLink -Path $linkPath -Target $_.FullName | Out-Null
            }
        }
    }
}

New-MigrationLink (Join-Path $corePath "migrations") $migrationsLinksDir
New-MigrationLink (Join-Path $desktopPath "migrations") $migrationsLinksDir

# 3. Update Database
sqlx db create
Check-LastExit "sqlx db create"

sqlx migrate run --source $migrationsLinksDir
Check-LastExit "sqlx migrate run"

Write-Host "--- Updating SQLx Metadata ---" -ForegroundColor Yellow

# 4. Prepare sqlx metadata
# Run preparation for core package
Push-Location $corePath
    cargo sqlx prepare -- --all-targets
    $exitCode = $LASTEXITCODE
Pop-Location
if ($exitCode -ne 0) { $LASTEXITCODE = $exitCode; Check-LastExit "cargo sqlx prepare (core)" }

# Run preparation for desktop app
Push-Location $desktopPath
    cargo sqlx prepare -- --all-targets
    $exitCode = $LASTEXITCODE
Pop-Location
if ($exitCode -ne 0) { $LASTEXITCODE = $exitCode; Check-LastExit "cargo sqlx prepare (desktop)" }

Write-Host "`nAll systems go!" -ForegroundColor Green