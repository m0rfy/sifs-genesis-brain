# Запуск cargo с путём к Rust (если cargo не в PATH)
# Использование: .\run_cargo.ps1 test   или   .\run_cargo.ps1 build --release
$cargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
if (Test-Path (Join-Path $cargoBin "cargo.exe")) {
    $env:PATH = "$cargoBin;$env:PATH"
}
& cargo $args
