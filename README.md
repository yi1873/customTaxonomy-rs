# customTaxonomy-rs

基于 Rust 的 NCBI 分类学数据库自定义过滤与修改工具。<br>
从原始 `nodes.dmp` / `names.dmp` 中筛选指定 taxid 的完整 lineage，并支持通过 `needChange.dmp` 插入新节点或修改已有节点的父节点和分类等级。

## 功能
- 读取目标 taxid 列表，自动追溯完整 lineage 节点taxid，仅生成目标lineage的taxonomy进行维护即可；
- 支持通过 `needChange.dmp` 自定义修改：插入新节点、修改已有节点的 parent 和 rank；
- 默认输出与 NCBI taxdump 格式完全一致的 `nodes_filtered.dmp` 和 `names_filtered.dmp`；

## 编译

```bash
git clone https://github.com/yi1873/customTaxonomy-rs.git
cd customTaxonomy-rs
cargo build --release
# 二进制文件: target/release/custom-taxonomy-rs
```

## 使用方法

```bash
target/release/custom-taxonomy-rs \
  --taxid-list  test/taxid.list \
  --need-change test/needChange.dmp \
  [--nodes-file /path/to/nodes.dmp] \
  [--names-file /path/to/names.dmp] \
  [--output-nodes ./nodes_filtered.dmp] \
  [--output-names ./names_filtered.dmp]
```

### needChange.dmp（13 列，制表符分隔）

每行定义一条自定义修改规则，格式合并了 names.dmp 和 nodes.dmp 的关键字段：

```
taxid  |  name  |  unique_name  |  name_type  |  taxid  |  parent  |  rank
```

**13 列详细说明：**

| 列号 (0-based) | 内容 | 说明 |
|:---:|------|------|
| 0 | taxid | 节点 taxid，**必须为 uint32 范围 (0 ~ 4,294,967,295)** |
| 1 | `\|` | 分隔符 |
| 2 | name | 物种名称 |
| 3 | `\|` | 分隔符 |
| 4 | unique_name | 唯一名称（可为空） |
| 5 | `\|` | 分隔符 |
| 6 | name_type | 名称类型（通常为 `scientific name`） |
| 7 | `\|` | 分隔符 |
| 8 | taxid | 节点 taxid（重复） |
| 9 | `\|` | 分隔符 |
| 10 | parent | 父节点 taxid |
| 11 | `\|` | 分隔符 |
| 12 | rank | 分类等级 |

**示例：**

```
1000000001	|	Escherichia-Shigella	|		|	scientific name	|	1000000001	|	543	|	customrank
561	|	Escherichia	|		|	scientific name	|	561	|	1000000001	|	genus
620	|	Shigella	|		|	scientific name	|	620	|	1000000001	|	genus
```

上述示例：
- 新增节点 `1000000001`（Escherichia-Shigella），父节点为 `543`，等级为 `customrank`
- 修改已有节点 `561`（Escherichia），将其父节点改为 `1000000001`，等级为 `genus`
- 修改已有节点 `620`（Shigella），将其父节点改为 `1000000001`，等级为 `genus`

**后续使用**
- kraken2 可直接使用；
- 可根据customrank抽提有效reads作基因组覆盖图；
- 示例仅用于解决kraken2中大肠/志贺菌的reads归类于family的问题；
- 其他自定义taxonomy可根据示例进行拓展；

