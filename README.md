# ETH Signer

A Rust-based Ethereum transaction signing service that supports multiple signing methods, including private keys, mnemonics, keystores, and cloud KMS services (AWS KMS, Google Cloud KMS).

## Features

- 🔐 **Multiple Signing Methods**

  - Private key signing
  - Mnemonic phrase signing
  - Keystore file signing
  - AWS KMS signing
  - Google Cloud KMS signing
  - Azure Key Vault signing (planned)
  - Alibaba Cloud KMS signing (planned)

- 🚀 **High-Performance Web Service**

  - Built on Axum async framework
  - JSON-RPC interface support
  - Health check endpoint
  - Structured logging

- 🐳 **Containerized Deployment**
  - Docker image support
  - Multi-stage build optimization
  - Minimal runtime image

## Quick Start

### Prerequisites

- Rust 1.89.0+
- Docker (optional)

### Installation and Running

1. **Clone the repository**

```bash
git clone <repository-url>
cd eth-signer
```

2. **Configure environment variables**

```bash
# Choose signing method and set corresponding environment variables
export SIGNER_TYPE=private_key
export SIGNER_PRIVATE_KEY=your_private_key_here
```

3. **Run the service**

```bash
# Development mode
cargo run

# Release mode
cargo run --release
```

### Docker Deployment

```bash
# Build image
docker build -t eth-signer .

# Run container
docker run -p 8000:8000 \
  -e SIGNER_TYPE=private_key \
  -e SIGNER_PRIVATE_KEY=your_private_key_here \
  eth-signer
```

## Configuration

### Supported Signing Types

#### 1. Private Key Signing

```bash
export SIGNER_TYPE=private_key
export SIGNER_PRIVATE_KEY=0x1234567890abcdef...
```

#### 2. Mnemonic Phrase Signing

```bash
export SIGNER_TYPE=mnemonic
export SIGNER_MNEMONIC="word1 word2 word3 ... word12"
```

#### 3. Keystore File Signing

```bash
export SIGNER_TYPE=keystore
export SIGNER_KEYSTORE_PATH=/path/to/keystore.json
export SIGNER_KEYSTORE_PASSWORD=your_password
```

#### 4. AWS KMS Signing

```bash
export SIGNER_TYPE=awskms
export SIGNER_AWSKMS_KEY=arn:aws:kms:region:account:key/key-id
# AWS credentials are automatically obtained via environment variables or IAM roles
```

#### 5. Google Cloud KMS Signing

```bash
export SIGNER_TYPE=gcpkms
export SIGNER_GCPKMS_PROJECT_ID=your-project-id
export SIGNER_GCPKMS_LOCATION=global
export SIGNER_GCPKMS_KEY_RING=your-key-ring
export SIGNER_GCPKMS_KEY=your-key-name
export SIGNER_GCPKMS_VERSION=1
# Google Cloud credentials are automatically obtained via environment variables or service accounts
```

### Other Configuration Options

- `PORT`: Service port (default: 8000)
- `RUST_LOG`: Log level (default: debug)

## API Reference

### Health Check

```http
GET /healthz
```

Returns: `OK`

### Get Public Key Address

```http
GET /pub
```

Returns: The signer's Ethereum address

### Sign Transaction

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

Response:

```json
{
  "id": 1,
  "jsonrpc": "2.0",
  "result": "0x02f8..."
}
```

## Development

### Project Structure

```
src/
├── main.rs          # Main program entry point
├── config.rs        # Command line arguments and configuration
├── error.rs         # Error definitions
├── prelude.rs       # Common imports
├── route.rs         # HTTP route handlers
└── signer/          # Signer module
    ├── mod.rs       # Signer implementation
    └── config.rs    # Signer configuration
```

### Build and Test

```bash
# Build the project
cargo build

# Run tests
cargo test

# Format code
cargo fmt

# Lint code
cargo clippy
```

### Adding New Signing Methods

1. Add new configuration variant in `src/signer/config.rs`
2. Add corresponding command line arguments in `src/config.rs`
3. Implement signer creation logic in the `signer()` method in `src/signer/mod.rs`

## Security Considerations

- 🔒 **Private Key Security**: Private keys and mnemonics should be passed via environment variables, avoid hardcoding in code
- 🔐 **Keystore Passwords**: Keystore passwords should be passed securely
- ☁️ **Cloud Service Permissions**: When using cloud KMS, ensure the principle of least privilege
- 🌐 **Network Security**: Use HTTPS and appropriate network isolation in production environments

## License

This project is licensed under the [MIT License](LICENSE).

## Contributing

Issues and Pull Requests are welcome!

## Changelog

### v0.1.0

- Initial release
- Support for private key, mnemonic, and keystore signing
- Support for AWS KMS and Google Cloud KMS
- JSON-RPC interface
- Containerized deployment support
