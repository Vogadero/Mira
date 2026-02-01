#!/bin/bash
# Mira 代码签名证书生成脚本 (macOS)
# 用于生成自签名证书进行代码签名

set -e

# 默认参数
CERT_NAME="Mira Development Certificate"
PUBLISHER="Mira Team"
VALID_DAYS=365
INSTALL=false

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# 参数解析
while [[ $# -gt 0 ]]; do
    case $1 in
        --cert-name)
            CERT_NAME="$2"
            shift 2
            ;;
        --publisher)
            PUBLISHER="$2"
            shift 2
            ;;
        --valid-days)
            VALID_DAYS="$2"
            shift 2
            ;;
        --install)
            INSTALL=true
            shift
            ;;
        --help)
            echo "用法: $0 [选项]"
            echo "选项:"
            echo "  --cert-name NAME     证书名称 (默认: $CERT_NAME)"
            echo "  --publisher NAME     发布者名称 (默认: $PUBLISHER)"
            echo "  --valid-days DAYS    有效天数 (默认: $VALID_DAYS)"
            echo "  --install           安装证书到钥匙串"
            echo "  --help              显示此帮助信息"
            exit 0
            ;;
        *)
            echo "未知参数: $1"
            echo "使用 --help 查看帮助信息"
            exit 1
            ;;
    esac
done

echo -e "${GREEN}Mira 证书生成脚本${NC}"
echo -e "${GREEN}==================${NC}"

# 检查必要工具
if ! command -v openssl &> /dev/null; then
    echo -e "${RED}错误: 未找到 openssl 命令${NC}"
    echo "请安装 OpenSSL: brew install openssl"
    exit 1
fi

# 创建证书目录
CERT_DIR="certificates"
if [ ! -d "$CERT_DIR" ]; then
    mkdir -p "$CERT_DIR"
    echo -e "${CYAN}创建证书目录: $CERT_DIR${NC}"
fi

echo -e "${CYAN}生成自签名证书...${NC}"
echo -e "  证书名称: $CERT_NAME"
echo -e "  发布者: $PUBLISHER"
echo -e "  有效期: $VALID_DAYS 天"

# 生成随机密码
PASSWORD=$(openssl rand -base64 12)

# 创建证书配置文件
CONFIG_FILE="$CERT_DIR/cert.conf"
cat > "$CONFIG_FILE" << EOF
[req]
distinguished_name = req_distinguished_name
x509_extensions = v3_req
prompt = no

[req_distinguished_name]
C = US
ST = CA
L = San Francisco
O = $PUBLISHER
OU = Development
CN = $CERT_NAME

[v3_req]
keyUsage = keyEncipherment, dataEncipherment, digitalSignature
extendedKeyUsage = codeSigning
subjectAltName = @alt_names

[alt_names]
DNS.1 = localhost
DNS.2 = *.local
EOF

# 生成私钥
PRIVATE_KEY="$CERT_DIR/mira-private-key.pem"
echo -e "${CYAN}生成私钥...${NC}"
openssl genrsa -out "$PRIVATE_KEY" 2048

# 生成证书
CERT_FILE="$CERT_DIR/mira-cert.pem"
echo -e "${CYAN}生成证书...${NC}"
openssl req -new -x509 -key "$PRIVATE_KEY" -out "$CERT_FILE" -days "$VALID_DAYS" -config "$CONFIG_FILE"

# 生成 PKCS#12 格式证书（包含私钥）
P12_FILE="$CERT_DIR/mira-cert.p12"
echo -e "${CYAN}生成 PKCS#12 证书...${NC}"
openssl pkcs12 -export -out "$P12_FILE" -inkey "$PRIVATE_KEY" -in "$CERT_FILE" -password "pass:$PASSWORD"

# 保存密码
PASSWORD_FILE="$CERT_DIR/certificate-password.txt"
echo "$PASSWORD" > "$PASSWORD_FILE"

echo -e "${GREEN}✓ 证书生成成功${NC}"

# 获取证书信息
SERIAL=$(openssl x509 -in "$CERT_FILE" -noout -serial | cut -d= -f2)
FINGERPRINT=$(openssl x509 -in "$CERT_FILE" -noout -fingerprint -sha1 | cut -d= -f2)
NOT_AFTER=$(openssl x509 -in "$CERT_FILE" -noout -enddate | cut -d= -f2)

echo -e "  序列号: $SERIAL"
echo -e "  指纹: $FINGERPRINT"
echo -e "  到期时间: $NOT_AFTER"

# 创建证书信息文件
INFO_FILE="$CERT_DIR/certificate-info.txt"
cat > "$INFO_FILE" << EOF
Mira 代码签名证书信息
====================

证书名称: $CERT_NAME
发布者: $PUBLISHER
生成时间: $(date '+%Y-%m-%d %H:%M:%S')
有效期: $VALID_DAYS 天
到期时间: $NOT_AFTER

序列号: $SERIAL
指纹: $FINGERPRINT

文件说明:
- mira-cert.pem: PEM 格式证书文件
- mira-private-key.pem: PEM 格式私钥文件
- mira-cert.p12: PKCS#12 格式证书文件（包含私钥）
- certificate-password.txt: P12 证书的密码

使用方法:
1. 代码签名 (需要 Apple Developer 账户):
   codesign --sign "$CERT_NAME" --verbose mira

2. 验证签名:
   codesign --verify --verbose mira
   spctl --assess --verbose mira

注意事项:
- 这是自签名证书，仅用于开发和测试
- macOS 应用分发需要 Apple Developer 证书
- 请妥善保管私钥文件和密码
- 不要将私钥文件提交到版本控制系统
EOF

echo -e "${GREEN}✓ 证书信息已保存: $INFO_FILE${NC}"

# 安装证书到钥匙串（如果请求）
if [ "$INSTALL" = true ]; then
    echo -e "${CYAN}安装证书到钥匙串...${NC}"
    
    # 检查是否有管理员权限
    if security add-trusted-cert -d -r trustRoot -k /Library/Keychains/System.keychain "$CERT_FILE" 2>/dev/null; then
        echo -e "${GREEN}✓ 证书已安装到系统钥匙串${NC}"
    else
        echo -e "${YELLOW}警告: 无法安装到系统钥匙串，尝试安装到用户钥匙串...${NC}"
        security add-cert -k ~/Library/Keychains/login.keychain "$CERT_FILE"
        echo -e "${GREEN}✓ 证书已安装到用户钥匙串${NC}"
    fi
    
    # 导入私钥
    security import "$P12_FILE" -k ~/Library/Keychains/login.keychain -P "$PASSWORD" -T /usr/bin/codesign
    echo -e "${GREEN}✓ 私钥已导入到钥匙串${NC}"
else
    echo -e "${YELLOW}提示: 使用 --install 参数可以将证书安装到钥匙串${NC}"
fi

# 创建签名脚本
SIGN_SCRIPT="$CERT_DIR/sign-mira.sh"
cat > "$SIGN_SCRIPT" << 'EOF'
#!/bin/bash
# Mira 代码签名脚本
# 使用生成的证书对 Mira 可执行文件进行签名

set -e

# 参数
FILE_PATH="$1"
CERT_NAME="${2:-Mira Development Certificate}"

if [ -z "$FILE_PATH" ]; then
    echo "用法: $0 <文件路径> [证书名称]"
    echo "示例: $0 target/release/mira"
    exit 1
fi

# 检查文件是否存在
if [ ! -f "$FILE_PATH" ]; then
    echo "错误: 文件不存在: $FILE_PATH"
    exit 1
fi

echo "对文件进行代码签名: $FILE_PATH"
echo "使用证书: $CERT_NAME"

# 执行签名
if codesign --sign "$CERT_NAME" --verbose "$FILE_PATH"; then
    echo "✓ 代码签名成功"
    
    # 验证签名
    echo "验证签名..."
    if codesign --verify --verbose "$FILE_PATH"; then
        echo "✓ 签名验证成功"
        
        # 显示签名信息
        echo "签名信息:"
        codesign --display --verbose "$FILE_PATH"
    else
        echo "⚠ 签名验证失败"
        exit 1
    fi
else
    echo "✗ 代码签名失败"
    exit 1
fi
EOF

chmod +x "$SIGN_SCRIPT"
echo -e "${GREEN}✓ 签名脚本已创建: $SIGN_SCRIPT${NC}"

# 清理临时文件
rm -f "$CONFIG_FILE"

echo ""
echo -e "${GREEN}证书生成完成！${NC}"
echo -e "${GREEN}===============${NC}"
echo -e "${CYAN}证书文件位置:${NC}"
echo -e "  PEM 证书: $CERT_FILE"
echo -e "  私钥: $PRIVATE_KEY"
echo -e "  P12 证书: $P12_FILE"
echo -e "  密码: $PASSWORD_FILE"
echo -e "  信息: $INFO_FILE"
echo -e "  签名脚本: $SIGN_SCRIPT"
echo ""
echo -e "${CYAN}使用示例:${NC}"
echo -e "  $SIGN_SCRIPT target/release/mira"
echo ""
echo -e "${YELLOW}安全提醒:${NC}"
echo -e "- 请妥善保管证书文件和密码"
echo -e "- 不要将私钥文件提交到版本控制"
echo -e "- macOS 应用分发需要 Apple Developer 证书"