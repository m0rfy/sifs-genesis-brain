# Установка требований для SIFS Brain + genesis-agi (Windows, RTX 4090)
# Запуск: PowerShell (при необходимости от администратора для winget).

$ErrorActionPreference = "Stop"

Write-Host "=== Проверка требований (Rust, CUDA, VS Build Tools) ===" -ForegroundColor Cyan

# 1. Rust
$cargoPath = "$env:USERPROFILE\.cargo\bin\cargo.exe"
if (Get-Command cargo -ErrorAction SilentlyContinue) {
    Write-Host 'OK: Rust (cargo) в PATH' -ForegroundColor Green
    cargo --version
} elseif (Test-Path $cargoPath) {
    Write-Host ('OK: Rust: ' + $cargoPath) -ForegroundColor Green
    & $cargoPath --version
} else {
    Write-Host '?: Rust не найден. Установка: winget install Rustlang.Rustup' -ForegroundColor Yellow
    Write-Host "    Или: https://rustup.rs" -ForegroundColor Gray
}

# 2. nvidia-smi
if (Get-Command nvidia-smi -ErrorAction SilentlyContinue) {
    Write-Host 'OK: nvidia-smi (драйвер)' -ForegroundColor Green
    nvidia-smi --query-gpu=name,driver_version,compute_cap --format=csv,noheader
} else {
    Write-Host 'FAIL: nvidia-smi не найден. Драйвер: https://www.nvidia.com/Download/index.aspx' -ForegroundColor Red
}

# 3. nvcc (CUDA Toolkit)
if (Get-Command nvcc -ErrorAction SilentlyContinue) {
    Write-Host 'OK: CUDA Toolkit (nvcc)' -ForegroundColor Green
    nvcc --version
} else {
    Write-Host 'FAIL: nvcc не в PATH. CUDA Toolkit: https://developer.nvidia.com/cuda-downloads' -ForegroundColor Red
    Write-Host "    Windows x86_64, 12.x. Для RTX 4090 драйвер 525+." -ForegroundColor Gray
}

# 4. Visual Studio (vswhere)
$vsWherePath = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
$hasVs = Test-Path $vsWherePath
if ($hasVs) {
    $vsPath = & $vsWherePath -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
    if ($vsPath) {
        Write-Host ('[OK] Visual Studio (C++): ' + $vsPath) -ForegroundColor Green
    } else {
        Write-Host '[FAIL] Нет workload C++. Установи Build Tools for VS 2022 с C++.' -ForegroundColor Red
    }
} else {
    Write-Host '[FAIL] Visual Studio не найдена. Build Tools: https://visualstudio.microsoft.com/visual-cpp-build-tools/' -ForegroundColor Red
}

Write-Host ""
Write-Host "RTX 4090: при сборке genesis-compute задай  `$env:CUDA_ARCH = 'sm_89'" -ForegroundColor Cyan
Write-Host "Сборка: из Developer PowerShell for VS 2022." -ForegroundColor Cyan
Write-Host "Подробнее: SETUP_WINDOWS.md" -ForegroundColor Gray
