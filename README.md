# zuuid

一个简单易用的命令行 UUID 生成工具，支持 UUID v4（随机）和 v7（时间有序）。

## 安装

从源码构建需要 Rust 1.89 或更新版本。

Homebrew 安装：

```sh
brew install zhangbyby/tap/zuuid
```

从源码安装：

```bash
cargo install --path .
```

## 卸载

```bash
brew uninstall zuuid     # Homebrew 安装
cargo uninstall zuuid    # Cargo 安装
```

## 常用参数

| 参数 | 说明 |
|------|------|
| `-V 4` / `-v 4` | UUID v4（随机，默认） |
| `-V 7` / `-v 7` | UUID v7（时间有序，适合数据库主键） |
| `-u` / `-U` | 大写输出 |
| `-s` / `-S` | 简单格式（不带短横线，32字符） |
| `-f` / `-F` | 完整格式（带短横线，36字符，默认） |
| `-h` / `--help` | 显示帮助 |

参数可以组合使用，如 `-us` 等同于 `-u -s`。

## 使用示例

```bash
# 默认：v4 + 小写 + 完整格式
zuuid
→ a1b2c3d4-e5f6-4a5b-8c7d-1e2f3a4b5c6d

# v7 + 大写 + 简单格式（适合数据库主键）
zuuid -v7 -us
→ 019BFE397C8A7F728C09C02111996FF5

# v4 + 大写 + 完整格式
zuuid -u
→ A1B2C3D4-E5F6-4A5B-8C7D-1E2F3A4B5C6D

# v4 + 小写 + 简单格式
zuuid -s
→ a1b2c3d4e5f64a5b8c7d1e2f3a4b5c6d

# v7 时间排序演示
for i in {1..3}; do zuuid -v7 -s; sleep 0.01; done
→ 019bfe2672857ee3bc5b6b831ddfede2
→ 019bfe26729d73639a6645640fcebd1d  # 递增
→ 019bfe2672bb7fd18891a09ece4aab90  # 递增
```

## UUID 版本选择

- **v4（默认）**：完全随机，适合大多数场景
- **v7**：时间有序，适合数据库主键，天然可排序

## 冲突处理

同时使用 `-f` 和 `-s` 时，会显示警告并按参数顺序决定格式：

```bash
zuuid -fs  # 完整格式（-f 在前）
zuuid -sf  # 简单格式（-s 在前）
```


## 行为与开发检查

格式优先级由 clap 解析后的参数位置决定，不扫描长参数名称；`--uuid-version 7 -fs` 与 `-fs` 都输出完整格式。语言选择优先级为第一个非空的 `LC_ALL`、`LC_MESSAGES`、`LANG`，不支持的语言回退到英文。

批量输出使用一次 stdout 锁、缓冲写入和复用的 UUID 编码缓冲区。下游提前关闭管道（如 `zuuid -n 100000 | head -n 1`）时正常退出；其他输出错误返回非零退出码。`-n 0` 不产生输出。

```sh
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
python3 -m unittest discover -s tests -p 'test_*.py'
actionlint
goreleaser check
```

Rust 集成测试覆盖真实 CLI 的格式优先级、版本参数、语言、批量输出、关闭管道和 Unix 非 UTF-8 程序名。最新稳定依赖为 `uuid 1.29.2`、`clap 4.6.7`；UUID 的最低 Rust 版本要求为 1.89。

## 发布流水线

普通分支推送和 PR 在 Linux、macOS、Windows 上执行检查、测试和发布构建，另用 Rust 1.89 检查最低版本兼容性。发布标签会先调用同一套 CI，通过后才运行发布任务。

发布前先更新并提交 `Cargo.toml`、`Cargo.lock` 中的项目版本，再推送一致的 `v<版本>` 标签。稳定版会发布五种目标的归档及 SHA-256，并更新 `zhangbyby/homebrew-tap` 的 `Formula/zuuid.rb`；预发布版本仅创建 GitHub prerelease，不更新稳定版 Formula。

GoReleaser 固定为 2.20.2，cargo-zigbuild 固定为 0.24.1，构建使用 `--locked`。发布 runner 使用 macOS 提供 Apple SDK：四个 Unix 目标使用 Zig 0.15.2，Windows GNU 目标使用 Cargo 和 MinGW（cargo-zigbuild 不支持 Windows 目标）。

Homebrew Formula 从实际的 macOS/Linux、Intel/ARM 四个归档计算校验值生成，不依赖已弃用的 GoReleaser `brews` 配置。Release 使用内置 `GITHUB_TOKEN`；更新 tap 继续使用已有 `GH_PAT` Secret，需对 tap 仓库具有 Contents 写权限。Formula 的测试检查程序版本及 UUIDv4 输出。
