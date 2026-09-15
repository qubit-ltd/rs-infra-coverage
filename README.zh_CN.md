# rs-infra-coverage

[![Rust CI](https://github.com/qubit-ltd/rs-infra-coverage/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-infra-coverage/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-infra-coverage/coverage-badge.json)](https://qubit-ltd.github.io/rs-infra-coverage/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-infra-coverage.svg?color=blue)](https://crates.io/crates/qubit-infra-coverage)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![English Document](https://img.shields.io/badge/Document-English-blue.svg)](README.md)

收集 LLVM 覆盖率数据、评估项目覆盖率阈值，并执行 coverage cfg 兼容性
Clippy 检查。

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

## 命令

```bash
rs-infra-coverage --project . collect
rs-infra-coverage --project . check --input target/infra/coverage/raw.json
rs-infra-coverage --project . report --input target/infra/coverage/raw.json
rs-infra-coverage --project . clippy --coverage-cfg
```

`collect` 支持 `scope`（`default-members`、`workspace` 或 `package`）和
`exclude_packages`。`check` 在排除 `threshold_exempt_files` 后应用配置的
`thresholds`。如果不存在 `.infra/ci/coverage.json`，会兼容读取旧的
`.rs-ci-coverage.json` 并输出迁移警告。

当使用 `--coverage-cfg`、设置 `RUN_COVERAGE_CFG_CLIPPY=1`、配置
`coverage_cfg_clippy: true` 或 `clippy.coverage_cfg: true` 时，Clippy 命令会
设置 `RUSTFLAGS=--cfg coverage`。

## 覆盖率策略

默认门禁要求 **lines >90%、functions >=95%、regions >85%**。
lines 和 regions 等于阈值时不通过，自定义阈值也采用同样规则。branches
可选，采用包含等号的下限。检查按所选文件的 covered/total 总数计算；必需
计数不可用时失败。省略 `thresholds`、使用 `{}` 或省略其中的字段，均保留
对应默认值。显式 `null` 不能关闭 lines、functions 或 regions 检查。

例如，Cargo 中声明的 package 名称为 `my-crate` 时：

```json
{
  "scope": "workspace",
  "source_dirs": {"my-crate": ["src/core", "src/io"]},
  "threshold_exempt_files": {"my-crate": ["src/core/generated.rs"]},
  "thresholds": {"lines": 90, "functions": 95, "regions": 85}
}
```

`collect`、`check` 和 `report` 使用 `cargo metadata --no-deps` 解析所选
package，因此需要 Cargo 以及包含 manifest 的源码工作副本。`default-members`
遵循 Cargo 的默认成员选择；`workspace` 选择全部成员；`package` 要求项目
manifest 声明 package。`exclude_packages` 排除指定成员。未知名称、未选中
package 的路径映射、重复排除项和空的选择结果都会报错。

路径相对于所指定 package 的 manifest 目录。所选 package 没有 `source_dirs`
条目时默认使用 `src`。source 数组不能为空；目录和豁免文件必须存在且类型
正确。路径必须相对寻址、使用正斜杠，不含 `..`、冒号或控制字符，同一数组
中不能有重复项。解析后重复的 source root 也会报错。

**应用豁免之前**，每个 source root 都必须至少命中一个报告文件。豁免按
完整规范路径精确匹配，因此一个 package 的 `src/lib.rs` 豁免不会影响另一个
package 的同名相对路径。过滤后没有文件会失败。`report` 与 `check` 执行
相同的 package/path 映射及逐个 root 命中校验，但不执行覆盖率百分比门禁。

## 能力与限制

项目策略放在 `.infra`，任务编排交给 `rs-infra-ci`。兼容范围限定为上文
记录的命令和配置字段。

## 延伸阅读

可通过命令帮助和源码测试了解实际接口。切换到 [English README](README.md)。

## 测试

```bash
# 使用默认 feature 集运行测试
cargo test

# 使用项目声明的全部 feature 运行测试
cargo test --all-features

# 运行项目 CI 检查
./ci-check.sh

# 检查代码覆盖率
./coverage.sh
```

## 许可证

Copyright (c) 2025 - 2026. Haixing Hu. All rights reserved.

本项目基于 Apache License 2.0 授权。完整许可证文本请参阅
[LICENSE](LICENSE)。

## 贡献

欢迎贡献。请遵循 Rust API 指南，及时更新公共 API 文档与测试，并在提交
Pull Request 前运行 `./align-ci.sh`格式化代码，运行`./ci-check.sh`对齐CI要求。

## 作者

**Haixing Hu** - *Qubit Co. Ltd.*

仓库地址：[https://github.com/qubit-ltd/rs-infra-coverage](https://github.com/qubit-ltd/rs-infra-coverage)
