# zuuid

一个简单易用的命令行 UUID 生成工具，支持 UUID v4（随机）和 v7（时间有序）。

## 安装

```bash
cargo install zuuid
```

## 卸载

```bash
cargo uninstall zuuid
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
