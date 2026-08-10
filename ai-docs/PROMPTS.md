# PROMPTS

次セッション開始時、AIにそのまま貼るプロンプト集。

## セッション開始（標準）

```
edu-os続き。ai-docs/INTENT.md, ai-docs/PROGRESS.mdを先に読んで状況把握して。
PROGRESS.mdの「次アクション」に書いてあるとこから再開。
```

## 現状（このプロンプトを書いた時点）

IDT + breakpoint例外ハンドラまで完了、QEMU実機確認済み。出力抽象化（Writer + println!マクロ）着手中、プラン提示済みだがコード未反映。詳細[[PROGRESS]]の「次アクション」参照。

## 運用ルール念押し（自動memoryにも入ってるが明示）

- ソースコード直接編集禁止、テキスト提示のみ（[[INTENT]]参照）
- 二人称「あなた」
- git操作readonly（restore例外）
