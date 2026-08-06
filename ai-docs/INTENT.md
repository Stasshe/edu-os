# INTENT

目的: Rust自作OS通じRust+OS学習。完成度より学習優先。ユーザーRustほぼ初心者、TS/C++既知。

## AI運用方針

コードはユーザーが全部書く。AIはreadonly原則。役割:
- 概念説明（ユーザー既知言語=TS/C++に紐付けて説明）
- 進捗確認・ビルド実行確認（`cargo build`等の非破壊コマンドは可）
- ai-docs/更新
コード生成・ファイル編集はしない。理由: 「勉強」が目的、答え渡すと学びにならない（ユーザー明言）。

git操作はreadonly。restore例外許可。それ以外(add/commit/restore以外の書込系)禁止。

## bootloader: クレート使用 or 自作、順序判断

両方学びたい意向あるが、同時進行は保留。

判断: OS(kernel)側 先、bootloader自作 後（別プロジェクト化予定）。

理由:
- kernel側は毎回可視フィードバックある（画面出力・割り込み動作等）→ モチベ維持しやすい
- bootloader単体（real mode→protected→long mode遷移）はQEMUレジスタダンプ頼りで地味、しかも「OS学びたい」という本題に触れないままの期間が長引く
- 自作bootloader×初学者Rustカーネル同時進行は、詰まった時に asm起因かRust/kernel起因か切り分けにくい。知識ドメイン分離した方が理解が濃くなる
- bootloaderの起動シーケンス自体は固定仕様（x86変わらない）→ 後回しにしても知識が目減りしない。OS側は学習範囲広く可変→先にやる方が効率いい

当面: `bootloader`クレート（v0.11系、Oppermann "Writing an OS in Rust"準拠）に丸投げしてkernel実装に集中。

自作bootloaderは別リポ/別branchで独立プロジェクト化予定（未着手）。
