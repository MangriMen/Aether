# env_db.ps1
$root = Get-Location
$dbPath = Join-Path $root "aether.db"
$uriPath = $dbPath.Replace('\', '/')
$env:DATABASE_URL = "sqlite:///$uriPath"

Write-Host "DATABASE_URL set to: $env:DATABASE_URL" -ForegroundColor Cyan