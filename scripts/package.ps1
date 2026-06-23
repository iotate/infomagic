# InfoMagic 一键打包脚本
# 1. 编译 Vue 前端
# 2. 编译 Tauri 后端
# 3. 打包为 ZIP（包含 exe + config + templates + styles + projects）

$ErrorActionPreference = "Stop"

$ROOT_DIR = Split-Path -Parent $PSScriptRoot
$SRC_TAURI_DIR = Join-Path $ROOT_DIR "src-tauri"
$RELEASE_DIR = Join-Path $SRC_TAURI_DIR "target\release"
$BUNDLE_DIR = Join-Path $RELEASE_DIR "bundle"
$OUTPUT_DIR = Join-Path $ROOT_DIR "output"
$VERSION = "0.2.0"

# 设置 MinGW 路径
$env:PATH = "D:\Dev\mingw64\bin;$env:PATH"

Write-Host "🚀 InfoMagic 打包脚本" -ForegroundColor Green
Write-Host "======================"
Write-Host "根目录: $ROOT_DIR"
Write-Host ""

# Step 1: 编译 Vue 前端
Write-Host "📦 Step 1: 编译 Vue 前端..." -ForegroundColor Cyan
Set-Location $ROOT_DIR
npm run build
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Vue 前端编译失败" -ForegroundColor Red
    exit 1
}
Write-Host "✅ Vue 前端编译完成" -ForegroundColor Green

# Step 2: 编译 Tauri 后端
Write-Host ""
Write-Host "🔧 Step 2: 编译 Tauri 后端..." -ForegroundColor Cyan
npm run tauri:build
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Tauri 后端编译失败" -ForegroundColor Red
    exit 1
}
Write-Host "✅ Tauri 后端编译完成" -ForegroundColor Green

# Step 3: 准备输出目录
Write-Host ""
Write-Host "📁 Step 3: 准备输出目录..." -ForegroundColor Cyan

# 清理旧输出
if (Test-Path $OUTPUT_DIR) {
    Remove-Item -Recurse -Force $OUTPUT_DIR
}
New-Item -ItemType Directory -Path $OUTPUT_DIR -Force | Out-Null

# 创建打包目录
$PACKAGE_NAME = "InfoMagic_${VERSION}_win64"
$PACKAGE_DIR = Join-Path $OUTPUT_DIR $PACKAGE_NAME
New-Item -ItemType Directory -Path $PACKAGE_DIR -Force | Out-Null

# Step 4: 复制可执行文件
Write-Host "📋 Step 4: 复制可执行文件..." -ForegroundColor Cyan

# 复制 MSI
$msiPath = Join-Path $BUNDLE_DIR "msi\InfoMagic_${VERSION}_x64_en-US.msi"
if (Test-Path $msiPath) {
    Copy-Item $msiPath $OUTPUT_DIR
    Write-Host "  ✅ 复制 MSI: InfoMagic_${VERSION}_x64_en-US.msi" -ForegroundColor Green
}

# 复制 EXE 安装包
$exePath = Join-Path $BUNDLE_DIR "nsis\InfoMagic_${VERSION}_x64-setup.exe"
if (Test-Path $exePath) {
    Copy-Item $exePath $OUTPUT_DIR
    Write-Host "  ✅ 复制 EXE: InfoMagic_${VERSION}_x64-setup.exe" -ForegroundColor Green
}

# 复制便携版 exe
$portableExe = Join-Path $RELEASE_DIR "infomagic.exe"
if (Test-Path $portableExe) {
    Copy-Item $portableExe (Join-Path $PACKAGE_DIR "InfoMagic.exe")
    Write-Host "  ✅ 复制便携版: InfoMagic.exe" -ForegroundColor Green
}

# Step 5: 复制数据文件夹
Write-Host "📋 Step 5: 复制数据文件夹..." -ForegroundColor Cyan

$dataFolders = @("config.yaml", "templates", "styles", "projects")

foreach ($folder in $dataFolders) {
    $srcPath = Join-Path $ROOT_DIR $folder
    $destPath = Join-Path $PACKAGE_DIR $folder
    
    if (Test-Path $srcPath) {
        Copy-Item -Recurse -Force $srcPath $destPath
        Write-Host "  ✅ 复制: $folder" -ForegroundColor Green
    } else {
        # 创建空目录或默认文件
        if ($folder -match "\.ya?ml$") {
            "# InfoMagic 配置文件" | Out-File -FilePath $destPath -Encoding utf8
        } else {
            New-Item -ItemType Directory -Path $destPath -Force | Out-Null
        }
        Write-Host "  📁 创建: $folder" -ForegroundColor Yellow
    }
}

# Step 6: 创建 ZIP 包
Write-Host ""
Write-Host "📦 Step 6: 创建 ZIP 包..." -ForegroundColor Cyan

$zipPath = Join-Path $OUTPUT_DIR "${PACKAGE_NAME}.zip"
Compress-Archive -Path $PACKAGE_DIR -DestinationPath $zipPath -Force
Write-Host "✅ ZIP 包已创建: $zipPath" -ForegroundColor Green

# 完成
Write-Host ""
Write-Host "🎉 打包完成！" -ForegroundColor Green
Write-Host "📁 输出目录: $OUTPUT_DIR"
Write-Host "📦 安装包目录: $PACKAGE_DIR"
Write-Host "📦 ZIP 文件: $zipPath"

# 打开输出目录
Start-Process "explorer.exe" $OUTPUT_DIR
