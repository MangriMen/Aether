$dbPath = Join-Path (Get-Location) "dev.db"

$uriPath = $dbPath.Replace('\', '/')

$env:DATABASE_URL = "sqlite:///$uriPath"

sqlx db setup