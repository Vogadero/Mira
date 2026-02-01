# Mira 发布构建脚本
# 用于创建优化的发布版本并验证大小要求

param(
    [switch]$Clean = $false,
    [switch]$Verbose = $false
)

Write-Host "Mira 发布构建脚本" -ForegroundColor Green
Write-Host "===================" -ForegroundColor Green

# 设置错误处理
$ErrorActionPreference = "Stop"

# 获取项目根目录
$ProjectRoot = Split-Path -Parent $PSScriptRoot
Set-Location $ProjectRoot

Write-Host "项目根目录: $ProjectRoot" -ForegroundColor Cyan

# 清理之前的构建（如果需要）
if ($Clean) {
    Write-Host "清理之前的构建..." -ForegroundColor Yellow
    if (Test-Path "target") {
        Remove-Item -Recurse -Force "target"
    }
}

# 检查 Rust 工具链
Write-Host "检查 Rust 工具链..." -ForegroundColor Cyan
try {
    $rustVersion = rustc --version
    Write-Host "Rust 版本: $rustVersion" -ForegroundColor Green
    
    $cargoVersion = cargo --version
    Write-Host "Cargo 版本: $cargoVersion" -ForegroundColor Green
} catch {
    Write-Error "Rust 工具链未安装或不可用"
    exit 1
}

# 构建发布版本
Write-Host "开始构建发布版本..." -ForegroundColor Cyan
$buildStart = Get-Date

try {
    if ($Verbose) {
        cargo build --release --verbose
    } else {
        cargo build --release
    }
    
    $buildEnd = Get-Date
    $buildTime = $buildEnd - $buildStart
    Write-Host "构建完成，耗时: $($buildTime.TotalSeconds.ToString('F2')) 秒" -ForegroundColor Green
} catch {
    Write-Error "发布构建失败: $_"
    exit 1
}

# 检查二进制文件
$binaryPath = "target\release\mira.exe"
if (-not (Test-Path $binaryPath)) {
    Write-Error "未找到构建的二进制文件: $binaryPath"
    exit 1
}

# 获取文件大小
$fileInfo = Get-Item $binaryPath
$fileSizeMB = [math]::Round($fileInfo.Length / 1MB, 2)

Write-Host "二进制文件信息:" -ForegroundColor Cyan
Write-Host "  路径: $binaryPath" -ForegroundColor White
Write-Host "  大小: $fileSizeMB MB" -ForegroundColor White
Write-Host "  创建时间: $($fileInfo.CreationTime)" -ForegroundColor White

# 验证大小要求
$maxSizeWindows = 20 # MB
if ($fileSizeMB -le $maxSizeWindows) {
    Write-Host "✓ 二进制文件大小符合要求 ($fileSizeMB MB <= $maxSizeWindows MB)" -ForegroundColor Green
} else {
    Write-Warning "⚠ 二进制文件大小超过要求 ($fileSizeMB MB > $maxSizeWindows MB)"
}

# 检查依赖项
Write-Host "检查依赖项..." -ForegroundColor Cyan
try {
    $dependencies = dumpbin /dependents $binaryPath 2>$null
    if ($dependencies) {
        Write-Host "依赖的 DLL 文件:" -ForegroundColor Yellow
        $dependencies | Select-String "\.dll" | ForEach-Object {
            Write-Host "  $($_.Line.Trim())" -ForegroundColor White
        }
    }
} catch {
    Write-Host "无法检查依赖项（需要 Visual Studio 工具）" -ForegroundColor Yellow
}

# 创建分发目录
$distDir = "dist"
if (Test-Path $distDir) {
    Remove-Item -Recurse -Force $distDir
}
New-Item -ItemType Directory -Path $distDir | Out-Null

Write-Host "创建分发包..." -ForegroundColor Cyan

# 复制二进制文件
Copy-Item $binaryPath "$distDir\mira.exe"

# 复制文档文件
$docFiles = @("README.md", "LICENSE")
foreach ($docFile in $docFiles) {
    if (Test-Path $docFile) {
        Copy-Item $docFile $distDir
    }
}

# 创建版本信息文件
$versionInfo = @"
Mira - 桌面摄像精灵
版本: $(cargo pkgid | Split-Path -Leaf)
构建时间: $(Get-Date -Format "yyyy-MM-dd HH:mm:ss")
构建机器: $env:COMPUTERNAME
Rust 版本: $rustVersion
二进制大小: $fileSizeMB MB

系统要求:
- Windows 10 或更高版本
- 支持 DirectX 11 的显卡
- 至少一个摄像头设备

使用说明:
1. 运行 mira.exe 启动应用程序
2. 左键拖拽移动窗口
3. 鼠标滚轮缩放窗口
4. Ctrl + 鼠标滚轮旋转窗口
5. F1-F5 切换形状
6. Tab 切换摄像头设备
"@

$versionInfo | Out-File -FilePath "$distDir\VERSION.txt" -Encoding UTF8

# 计算分发包总大小
$distSize = (Get-ChildItem $distDir -Recurse | Measure-Object -Property Length -Sum).Sum
$distSizeMB = [math]::Round($distSize / 1MB, 2)

Write-Host "分发包信息:" -ForegroundColor Cyan
Write-Host "  目录: $distDir" -ForegroundColor White
Write-Host "  总大小: $distSizeMB MB" -ForegroundColor White
Write-Host "  文件数量: $((Get-ChildItem $distDir -Recurse -File).Count)" -ForegroundColor White

# 验证分发包大小
$maxDistSize = 50 # MB
if ($distSizeMB -le $maxDistSize) {
    Write-Host "✓ 分发包大小符合要求 ($distSizeMB MB <= $maxDistSize MB)" -ForegroundColor Green
} else {
    Write-Warning "⚠ 分发包大小超过要求 ($distSizeMB MB > $maxDistSize MB)"
}

# 创建 ZIP 压缩包
$zipPath = "mira-windows-x64.zip"
if (Test-Path $zipPath) {
    Remove-Item $zipPath
}

Write-Host "创建 ZIP 压缩包..." -ForegroundColor Cyan
try {
    Compress-Archive -Path "$distDir\*" -DestinationPath $zipPath -CompressionLevel Optimal
    
    $zipInfo = Get-Item $zipPath
    $zipSizeMB = [math]::Round($zipInfo.Length / 1MB, 2)
    
    Write-Host "✓ ZIP 压缩包创建成功:" -ForegroundColor Green
    Write-Host "  文件: $zipPath" -ForegroundColor White
    Write-Host "  大小: $zipSizeMB MB" -ForegroundColor White
    
    # 验证压缩包大小（应该比原始二进制文件小）
    if ($zipSizeMB -le $maxSizeWindows) {
        Write-Host "✓ 压缩包大小符合下载要求 ($zipSizeMB MB <= $maxSizeWindows MB)" -ForegroundColor Green
    } else {
        Write-Warning "⚠ 压缩包大小可能影响下载体验 ($zipSizeMB MB > $maxSizeWindows MB)"
    }
} catch {
    Write-Error "创建 ZIP 压缩包失败: $_"
    exit 1
}

# 运行基本测试（如果可能）
Write-Host "运行基本验证..." -ForegroundColor Cyan
try {
    # 尝试运行应用程序的版本检查
    $versionOutput = & $binaryPath --version 2>$null
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✓ 应用程序可以正常启动" -ForegroundColor Green
        Write-Host "  版本信息: $versionOutput" -ForegroundColor White
    } else {
        Write-Warning "⚠ 应用程序启动测试失败（可能需要图形环境）"
    }
} catch {
    Write-Warning "⚠ 无法运行基本验证测试: $_"
}

Write-Host ""
Write-Host "构建摘要:" -ForegroundColor Green
Write-Host "=========" -ForegroundColor Green
Write-Host "✓ 发布构建完成" -ForegroundColor Green
Write-Host "✓ 二进制文件: $fileSizeMB MB" -ForegroundColor Green
Write-Host "✓ 分发包: $distSizeMB MB" -ForegroundColor Green
Write-Host "✓ 压缩包: $zipSizeMB MB" -ForegroundColor Green
Write-Host ""
Write-Host "文件位置:" -ForegroundColor Cyan
Write-Host "  二进制: $binaryPath" -ForegroundColor White
Write-Host "  分发目录: $distDir" -ForegroundColor White
Write-Host "  压缩包: $zipPath" -ForegroundColor White
Write-Host ""
Write-Host "发布构建完成！" -ForegroundColor Green