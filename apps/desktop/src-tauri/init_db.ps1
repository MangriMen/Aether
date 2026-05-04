# Prepare path
$dbPath = Join-Path (Get-Location) "dev.db"

$uriPath = $dbPath.Replace('\', '/')

$env:DATABASE_URL = "sqlite:///$uriPath"

$migrations_links_dir = "migrations_links";

Remove-Item -Force -Recurse $migrations_links_dir

# Sync symlinks
if (-not (Test-Path $migrations_links_dir)) { 
  New-Item -ItemType Directory -Path $migrations_links_dir 
}

function New-MigrationLink($sourcePath) {
    Get-ChildItem $sourcePath | ForEach-Object {
        $linkPath = Join-Path $migrations_links_dir $_.Name
        if (-not (Test-Path $linkPath)) {
            New-Item -ItemType HardLink -Path $linkPath -Target $_.FullName
        }
    }
}

New-MigrationLink "./migrations/*.sql"
New-MigrationLink "../../../packages/core/aether-core/migrations/*.sql"

sqlx db create
sqlx migrate run --source migrations_links