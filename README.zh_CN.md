# rs-infra-coverage

[![Rust CI](https://github.com/qubit-ltd/rs-infra-coverage/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-infra-coverage/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-infra-coverage/coverage-badge.json)](https://qubit-ltd.github.io/rs-infra-coverage/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-infra-coverage.svg?color=blue)](https://crates.io/crates/qubit-infra-coverage)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![English Document](https://img.shields.io/badge/Document-English-blue.svg)](README.md)

收集 LLVM 覆盖率数据，并根据配置评估项目覆盖率阈值。

## 安装

```bash
cargo install --git https://github.com/qubit-ltd/rs-infra-coverage.git --tag v0.1.0 qubit-infra-coverage
```

## 快速开始

在 Rust 项目根目录查看命令帮助：

```bash
cargo run --manifest-path /path/to/rs-infra-coverage/Cargo.toml -- --help
```

项目的 `.infra` 配置仍然是行为的唯一来源；工具仓库不会复制项目配置。具体策略由项目配置决定。

## 能力与限制

当前版本只提供上文列出的专门能力，刻意保持为小型基础设施组件：项目策略放在 `.infra`，任务编排交给 `rs-infra-ci`。对于旧版 `rs-ci` 脚本，只有测试覆盖的命令可视为兼容。

## 延伸阅读

可通过命令帮助和源码测试了解实际接口。切换到 [English README](README.md)。

## 测试

```bash
cargo test
cargo test --all-features
./ci-check.sh
./coverage.sh
```

## 许可证

Copyright (c) 2025 - 2026. Haixing Hu. All rights reserved.

本项目基于 Apache License 2.0 授权。完整许可证文本请参阅 [LICENSE](LICENSE)。

## 贡献

欢迎贡献。请遵循 Rust API 指南，及时更新公共 API 文档与测试，并在提交 Pull Request 前运行 `./align-ci.sh` 格式化代码，运行 `./ci-check.sh` 满足 CI 要求。

## 作者

**Haixing Hu** - *Qubit Co. Ltd.*

仓库地址：[https://github.com/qubit-ltd/rs-infra-coverage](https://github.com/qubit-ltd/rs-infra-coverage)
