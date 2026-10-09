# 越南语 Telex 词句表

这里放越南语 Telex 模式第一版使用的明文 TSV 数据。

- `words.tsv`：`词<TAB>telex<TAB>词频`
- `phrases.tsv`：`短语<TAB>telex<TAB>词频`

当前词句表由 `assets/glossary/glossary-vi.tsv` 的越南语释义抽取生成：去掉词性前缀，把单词写进 `words.tsv`，把含空格的释义写进 `phrases.tsv`，并按越南语字符反推 Telex 编码。原来的验收种子项（如 `tiếng Việt`、`tôi yêu`、`Việt Nam`）保留在表头附近。

重生成命令：

```bash
cargo run -p qingjian-dict-convert -- --out-dir data/generated vietnamese assets/glossary/glossary-vi.tsv
```

生成结果会写到 `data/generated/vietnamese/`；确认后再同步到本目录。频率来自释义表顺序与重复次数估算，不是真实越南语语料频率。
