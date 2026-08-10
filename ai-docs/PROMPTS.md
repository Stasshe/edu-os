# PROMPTS

次セッション開始時、AIにそのまま貼るプロンプト集。

## セッション開始（標準）

```
edu-os続き。ai-docs/INTENT.md, ai-docs/PROGRESS.mdを先に読んで状況把握して。
PROGRESS.mdの「次アクション」に書いてあるとこから再開。
```

## 現状（このプロンプトを書いた時点）

IDT(breakpoint例外ハンドラ)着手直前。具体プラン・コード案は[[PROGRESS]]の「次アクション」セクション参照。まだ`kernel/Cargo.toml`にx86_64/lazy_static追加もkernel/src/main.rsへのコード反映もしていない状態。

## 運用ルール念押し（自動memoryにも入ってるが明示）

- ソースコード直接編集禁止、テキスト提示のみ（[[INTENT]]参照）
- 二人称「あなた」
- git操作readonly（restore例外）
