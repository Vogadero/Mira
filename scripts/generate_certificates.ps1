# Mira 代码签名证书生成脚本 (Windows)
# 用于生成自签名证书进行代码签名

param(
    [string]$CertName = "Mira Development Certificate",
    [string]$Publisher = "Mira Team",
    [int]$ValidDays = 365,
    [switch]$Install = $false
)

Write-Host "Mira 证书生成脚本" -ForegroundColor Green
Write-Host "==================" -ForegroundColor Green

# 检查是否以管理员身份运行
$isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole] "Administrator")

if (-not $isAdmin -and $Install) {
    Write-Warning "安装证书需要管理员权限。请以管理员身份运行此脚本，或使用 -Install:$false 仅生成证书文件。"
    exit 1
}

# 创建证书目录
$certDir = "certificates"
if (-not (Test-Path $certDir)) {
    New-Item -ItemType Directory -Path $certDir | Out-Null
    Write-Host "创建证书目录: $certDir" -ForegroundColor Cyan
}

# 生成证书参数
$certParams = @{
    Subject = "CN=$Publisher, O=$Publisher, C=US"
    FriendlyName = $CertName
    NotAfter = (Get-Date).AddDays($ValidDays)
    CertStoreLocation = "Cert:\CurrentUser\My"
    KeyUsage = "DigitalSignature"
    KeyAlgorithm = "RSA"
    KeyLength = 2048
    Provider = "Microsoft Enhanced RSA and AES Cryptographic Provider"
    Type = "CodeSigningCert"
}

try {
    Write-Host "生成自签名证书..." -ForegroundColor Cyan
    Write-Host "  证书名称: $CertName" -ForegroundColor White
    Write-Host "  发布者: $Publisher" -ForegroundColor White
    Write-Host "  有效期: $ValidDays 天" -ForegroundColor White
    
    # 生成证书
    $cert = New-SelfSignedCertificate @certParams
    
    Write-Host "✓ 证书生成成功" -ForegroundColor Green
    Write-Host "  指纹: $($cert.Thumbprint)" -ForegroundColor White
    Write-Host "  序列号: $($cert.SerialNumber)" -ForegroundColor White
    
    # 导出证书到文件
    $certPath = Join-Path $certDir "mira-cert.pfx"
    $cerPath = Join-Path $certDir "mira-cert.cer"
    
    # 生成随机密码
    $password = -join ((65..90) + (97..122) + (48..57) | Get-Random -Count 12 | ForEach-Object {[char]$_})
    $securePassword = ConvertTo-SecureString -String $password -Force -AsPlainText
    
    # 导出 PFX 文件（包含私钥）
    Export-PfxCertificate -Cert $cert -FilePath $certPath -Password $securePassword | Out-Null
    Write-Host "✓ PFX 证书已导出: $certPath" -ForegroundColor Green
    
    # 导出 CER 文件（仅公钥）
    Export-Certificate -Cert $cert -FilePath $cerPath | Out-Null
    Write-Host "✓ CER 证书已导出: $cerPath" -ForegroundColor Green
    
    # 保存密码到文件
    $passwordFile = Join-Path $certDir "certificate-password.txt"
    $password | Out-File -FilePath $passwordFile -Encoding UTF8
    Write-Host "✓ 证书密码已保存: $passwordFile" -ForegroundColor Green
    
    # 创建证书信息文件
    $infoContent = @"
Mira 代码签名证书信息
====================

证书名称: $CertName
发布者: $Publisher
生成时间: $(Get-Date -Format "yyyy-MM-dd HH:mm:ss")
有效期: $ValidDays 天
到期时间: $($cert.NotAfter.ToString("yyyy-MM-dd HH:mm:ss"))

证书指纹: $($cert.Thumbprint)
序列号: $($cert.SerialNumber)

文件说明:
- mira-cert.pfx: 包含私钥的证书文件，用于代码签名
- mira-cert.cer: 仅包含公钥的证书文件，用于验证签名
- certificate-password.txt: PFX 证书的密码

使用方法:
1. 代码签名:
   signtool sign /f mira-cert.pfx /p [密码] /t http://timestamp.digicert.com mira.exe

2. 验证签名:
   signtool verify /pa mira.exe

注意事项:
- 这是自签名证书，仅用于开发和测试
- 生产环境请使用受信任的 CA 颁发的证书
- 请妥善保管 PFX 文件和密码
- 不要将私钥文件提交到版本控制系统
"@
    
    $infoFile = Join-Path $certDir "certificate-info.txt"
    $infoContent | Out-File -FilePath $infoFile -Encoding UTF8
    Write-Host "✓ 证书信息已保存: $infoFile" -ForegroundColor Green
    
    # 安装证书到受信任的根证书颁发机构（如果请求）
    if ($Install) {
        Write-Host "安装证书到受信任的根证书颁发机构..." -ForegroundColor Cyan
        
        try {
            Import-Certificate -FilePath $cerPath -CertStoreLocation "Cert:\LocalMachine\Root" | Out-Null
            Write-Host "✓ 证书已安装到受信任的根证书颁发机构" -ForegroundColor Green
            Write-Host "  现在可以使用此证书进行代码签名而不会出现安全警告" -ForegroundColor White
        } catch {
            Write-Error "安装证书失败: $_"
        }
    } else {
        Write-Host "提示: 使用 -Install 参数可以将证书安装到受信任的根证书颁发机构" -ForegroundColor Yellow
    }
    
    # 创建签名脚本
    $signScript = @"
# Mira 代码签名脚本
# 使用生成的证书对 Mira 可执行文件进行签名

param(
    [Parameter(Mandatory=`$true)]
    [string]`$FilePath,
    [string]`$CertPath = "certificates\mira-cert.pfx",
    [string]`$PasswordFile = "certificates\certificate-password.txt",
    [string]`$TimestampUrl = "http://timestamp.digicert.com"
)

# 检查文件是否存在
if (-not (Test-Path `$FilePath)) {
    Write-Error "文件不存在: `$FilePath"
    exit 1
}

if (-not (Test-Path `$CertPath)) {
    Write-Error "证书文件不存在: `$CertPath"
    exit 1
}

if (-not (Test-Path `$PasswordFile)) {
    Write-Error "密码文件不存在: `$PasswordFile"
    exit 1
}

# 读取密码
`$password = Get-Content `$PasswordFile -Raw
`$password = `$password.Trim()

Write-Host "对文件进行代码签名: `$FilePath" -ForegroundColor Cyan

try {
    # 执行签名
    signtool sign /f "`$CertPath" /p "`$password" /t "`$TimestampUrl" /v "`$FilePath"
    
    if (`$LASTEXITCODE -eq 0) {
        Write-Host "✓ 代码签名成功" -ForegroundColor Green
        
        # 验证签名
        Write-Host "验证签名..." -ForegroundColor Cyan
        signtool verify /pa /v "`$FilePath"
        
        if (`$LASTEXITCODE -eq 0) {
            Write-Host "✓ 签名验证成功" -ForegroundColor Green
        } else {
            Write-Warning "签名验证失败"
        }
    } else {
        Write-Error "代码签名失败"
        exit 1
    }
} catch {
    Write-Error "签名过程中出错: `$_"
    exit 1
}
"@
    
    $signScriptPath = Join-Path $certDir "sign-mira.ps1"
    $signScript | Out-File -FilePath $signScriptPath -Encoding UTF8
    Write-Host "✓ 签名脚本已创建: $signScriptPath" -ForegroundColor Green
    
    Write-Host ""
    Write-Host "证书生成完成！" -ForegroundColor Green
    Write-Host "===============" -ForegroundColor Green
    Write-Host "证书文件位置:" -ForegroundColor Cyan
    Write-Host "  PFX: $certPath" -ForegroundColor White
    Write-Host "  CER: $cerPath" -ForegroundColor White
    Write-Host "  密码: $passwordFile" -ForegroundColor White
    Write-Host "  信息: $infoFile" -ForegroundColor White
    Write-Host "  签名脚本: $signScriptPath" -ForegroundColor White
    Write-Host ""
    Write-Host "使用示例:" -ForegroundColor Cyan
    Write-Host "  .\certificates\sign-mira.ps1 -FilePath target\release\mira.exe" -ForegroundColor White
    Write-Host ""
    Write-Host "安全提醒:" -ForegroundColor Yellow
    Write-Host "- 请妥善保管证书文件和密码" -ForegroundColor White
    Write-Host "- 不要将私钥文件提交到版本控制" -ForegroundColor White
    Write-Host "- 生产环境请使用受信任的 CA 证书" -ForegroundColor White
    
} catch {
    Write-Error "生成证书时出错: $_"
    exit 1
}