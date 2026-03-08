# Установка окружения (Windows, RTX 4090)

Всё необходимое для сборки **SIFS Brain** (Genesis CPU) и **genesis-agi с CUDA** (наша копия в `genesis-agi/`).

---

## 1. Rust

**Нужно:** `rustc`, `cargo` в PATH (обычно через rustup).

**Установка (один из способов):**

- **winget (рекомендуется):**
  ```powershell
  winget install Rustlang.Rustup
  ```
  После установки **перезапусти терминал** или выполни: `$env:Path = [System.Environment]::GetEnvironmentVariable("Path","Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path","User")`

- **Или вручную:** скачай [rustup-init.exe](https://win.rustup.rs/x86_64) и запусти. Выбери установку по умолчанию, затем добавь в PATH: `%USERPROFILE%\.cargo\bin`.

**Проверка:**
```powershell
cargo --version
rustc --version
```

---

## 2. Visual Studio Build Tools (C++)

**Нужно для:** сборки genesis-compute (CUDA на Windows использует MSVC как host compiler).

**Установка:**

- Скачай [Build Tools for Visual Studio 2022](https://visualstudio.microsoft.com/visual-cpp-build-tools/) (бесплатно).
- В установщике выбери workload **«Разработка классических приложений на C++»** (Desktop development with C++). Включи компоненты:
  - MSVC v143 (или новее)
  - Windows 10/11 SDK
  - C++ CMake tools (по желанию)

**Проверка:** в **Developer Command Prompt** или после запуска `VsDevCmd.bat` должна быть в PATH команда `cl.exe`:
```powershell
& "${env:ProgramFiles(x86)}\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat"
cl
```

Для сборки из обычного PowerShell лучше собирать из **Developer PowerShell for VS 2022** (в меню Пуск) или один раз выполнить `vcvars64.bat` в cmd и из той же консоли запускать `cargo build`.

---

## 3. CUDA Toolkit (для RTX 4090)

**Нужно:** `nvcc`, библиотеки CUDA Runtime. RTX 4090 — архитектура Ada (compute capability **8.9**, в build используется `sm_89`).

**Установка:**

- Официальная инструкция: [CUDA Installation Guide for Microsoft Windows](https://docs.nvidia.com/cuda/cuda-installation-guide-microsoft-windows/index.html).
- Скачай [CUDA Toolkit 12.x для Windows](https://developer.nvidia.com/cuda-downloads?target_os=Windows&target_arch=x86_64) (например 12.4 или 12.6). Выбери версию Windows и установщик (exe локальный или network). Для RTX 4090 подойдёт CUDA 12.x; нужен компилятор MSVC 2017/2019/2022 ([NVIDIA docs](https://docs.nvidia.com/cuda/archive/12.4.0/cuda-installation-guide-microsoft-windows/index.html)).
- Установи. По умолчанию путь: `C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v12.x`.
- Убедись, что в PATH добавлено (установщик обычно делает сам):
  - `C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v12.x\bin`
  - `C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v12.x\libnvvp` (опционально)

**Проверка:**
```powershell
nvidia-smi
nvcc --version
```

Для RTX 4090 build автоматически определит `sm_89` через `nvidia-smi --query-gpu=compute_cap`. Если нужно задать вручную:
```powershell
$env:CUDA_ARCH = "sm_89"
cargo build --release -p genesis-compute
```

**Драйвер:** для CUDA 12 нужен драйвер NVIDIA **525.60.13 или новее**. Проверь в «Диспетчер устройств» или `nvidia-smi`.

---

## 4. Остальное (уже в Cargo)

- **genesis-agi (наша копия):** все Rust-зависимости подтягиваются через `cargo build` (rayon, serde, bytemuck, libc, anyhow и т.д.).
- **Genesis CPU-гибрид:** только Rust + rayon + serde_json; CUDA не нужен.
- **Python (для CartPole, скриптов):** если ещё нет — установи [Python 3.11+](https://www.python.org/downloads/) и добавь в PATH. Для демо: `pip install gymnasium`.

---

## Быстрая проверка всего

Из папки **Genesis** (для CPU-мозга):

```powershell
.\run_cargo.ps1 test
.\run_cargo.ps1 run --release
```

Из папки **genesis-agi** (с CUDA):

Сначала открой **Developer PowerShell for VS 2022** (или выполни `vcvars64.bat` в cmd), затем:

```powershell
cd c:\Users\m0rfy\Projects\genesis-agi
$env:CUDA_ARCH = "sm_89"
cargo build --release -p genesis-core
cargo build --release -p genesis-compute
```

Если `genesis-compute` собирается без `mock-gpu` — CUDA и компилятор настроены верно. Если ошибки линковки/компиляции — проверь PATH (nvcc, cl.exe) и снова запусти из Developer PowerShell.

---

## Краткий чеклист

| Компонент | Команда проверки |
|-----------|-------------------|
| Rust | `cargo --version` |
| MSVC (C++) | Запуск из Developer PowerShell или `cl` в PATH после vcvars64 |
| Драйвер NVIDIA | `nvidia-smi` |
| CUDA Toolkit | `nvcc --version` |
| Genesis CPU | `cd Genesis; .\run_cargo.ps1 test` |
| genesis-agi (CUDA) | `cd genesis-agi; cargo build --release -p genesis-compute` |
