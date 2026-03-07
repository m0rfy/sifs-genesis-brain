# Быстрая проверка: тесты + сборка бенчмарков (без запуска bench).
# Запуск: из корня репо, .\scripts\ci_check.ps1
$ErrorActionPreference = "Stop"
$root = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
Push-Location $root
try {
    $cargo = Join-Path $env:USERPROFILE ".cargo\bin\cargo.exe"
    if (-not (Test-Path $cargo)) { $cargo = "cargo" }
    & $cargo test -p sifs-genesis-core
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    & $cargo bench -p sifs-genesis-core --no-run
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    Write-Host "OK: tests and bench build passed."
} finally {
    Pop-Location
}
