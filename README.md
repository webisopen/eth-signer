# ETH Signer

一个基于 Rust 的以太坊交易签名服务，支持多种签名方式，包括私钥、助记词、密钥库以及云服务商 KMS（AWS KMS、Google Cloud KMS）。

## 功能特性

- 🔐 **多种签名方式支持**

  - 私钥签名
  - 助记词签名
  - 密钥库文件签名
  - AWS KMS 签名
  - Google Cloud KMS 签名
  - Azure Key Vault 签名（计划中）
  - 阿里云 KMS 签名（计划中）

- 🚀 **高性能 Web 服务**

  - 基于 Axum 异步框架
  - 支持 JSON-RPC 接口
  - 健康检查端点
  - 结构化日志记录

- 🐳 **容器化部署**
  - Docker 镜像支持
  - 多阶段构建优化
  - 最小化运行时镜像

## 快速开始

### 环境要求

- Rust 1.89.0+
- Docker（可选）

### 安装和运行

1. **克隆项目**

```bash
git clone <repository-url>
cd eth-signer
```

2. **配置环境变量**

```bash
# 选择签名方式并设置相应环境变量
export SIGNER_TYPE=private_key
export SIGNER_PRIVATE_KEY=your_private_key_here
```

3. **运行服务**

```bash
# 开发模式
cargo run

# 发布模式
cargo run --release
```

### Docker 部署

```bash
# 构建镜像
docker build -t eth-signer .

# 运行容器
docker run -p 8000:8000 \
  -e SIGNER_TYPE=private_key \
  -e SIGNER_PRIVATE_KEY=your_private_key_here \
  eth-signer
```

## 配置

### 支持的签名类型

#### 1. 私钥签名

```bash
export SIGNER_TYPE=private_key
export SIGNER_PRIVATE_KEY=0x1234567890abcdef...
```

#### 2. 助记词签名

```bash
export SIGNER_TYPE=mnemonic
export SIGNER_MNEMONIC="word1 word2 word3 ... word12"
```

#### 3. 密钥库文件签名

```bash
export SIGNER_TYPE=keystore
export SIGNER_KEYSTORE_PATH=/path/to/keystore.json
export SIGNER_KEYSTORE_PASSWORD=your_password
```

#### 4. AWS KMS 签名

```bash
export SIGNER_TYPE=awskms
export SIGNER_AWSKMS_KEY=arn:aws:kms:region:account:key/key-id
# AWS 凭证通过环境变量或 IAM 角色自动获取
```

#### 5. Google Cloud KMS 签名

```bash
export SIGNER_TYPE=gcpkms
export SIGNER_GCPKMS_PROJECT_ID=your-project-id
export SIGNER_GCPKMS_LOCATION=global
export SIGNER_GCPKMS_KEY_RING=your-key-ring
export SIGNER_GCPKMS_KEY=your-key-name
export SIGNER_GCPKMS_VERSION=1
# Google Cloud 凭证通过环境变量或服务账号自动获取
```

### 其他配置选项

- `PORT`: 服务端口（默认：8000）
- `RUST_LOG`: 日志级别（默认：debug）

## API 接口

### 健康检查

```http
GET /healthz
```

返回：`OK`

### 获取公钥地址

```http
GET /pub
```

返回：签名者的以太坊地址

### 签名交易

```http
POST /
Content-Type: application/json

{
  "id": 1,
  "jsonrpc": "2.0",
  "method": "eth_signTransaction",
  "params": [
    {
      "from": "0x...",
      "to": "0x...",
      "value": "0x0",
      "gas": "0x5208",
      "gasPrice": "0x3b9aca00",
      "data": "0x..."
    }
  ]
}
```

返回：

```json
{
  "id": 1,
  "jsonrpc": "2.0",
  "result": "0x02f8..."
}
```

## 开发

### 项目结构

```
src/
├── main.rs          # 主程序入口
├── config.rs        # 命令行参数和配置
├── error.rs         # 错误定义
├── prelude.rs       # 公共导入
├── route.rs         # HTTP 路由处理
└── signer/          # 签名器模块
    ├── mod.rs       # 签名器实现
    └── config.rs    # 签名器配置
```

### 构建和测试

```bash
# 构建项目
cargo build

# 运行测试
cargo test

# 代码格式化
cargo fmt

# 代码检查
cargo clippy
```

### 添加新的签名方式

1. 在 `src/signer/config.rs` 中添加新的配置变体
2. 在 `src/config.rs` 中添加相应的命令行参数
3. 在 `src/signer/mod.rs` 的 `signer()` 方法中实现签名器创建逻辑

## 安全注意事项

- 🔒 **私钥安全**：私钥和助记词应通过环境变量传递，避免在代码中硬编码
- 🔐 **密钥库密码**：密钥库密码应通过安全的方式传递
- ☁️ **云服务权限**：使用云 KMS 时，确保最小权限原则
- 🌐 **网络安全**：生产环境中应使用 HTTPS 和适当的网络隔离

## 许可证

本项目采用 [MIT 许可证](LICENSE)。

## 贡献

欢迎提交 Issue 和 Pull Request！

## 更新日志

### v0.1.0

- 初始版本发布
- 支持私钥、助记词、密钥库签名
- 支持 AWS KMS 和 Google Cloud KMS
- 提供 JSON-RPC 接口
- 容器化部署支持
