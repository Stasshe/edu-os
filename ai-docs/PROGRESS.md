# PROGRESS

前提: [[INTENT]]必読。方針・判断理由そちら。

## 環境

- QEMU install済み、起動window確認済み
- rustup target: `x86_64-unknown-none` 追加済み
- `.cargo/config.toml`: `[build] target = "x86_64-unknown-none"` 設定済み（cargo/rust-analyzer双方このtarget固定、host targetとのstd衝突誤診断を回避）
- toolchain: nightly (edition 2024)

## src/main.rs 現状

freestanding化済み。`#![no_std]` `#![no_main]`、`_start`（`extern "C"`, `#[unsafe(no_mangle)]`）は無限spin loop、`#[panic_handler]`自前定義。

`cargo build --target x86_64-unknown-none` 成功確認済み（ELF binary生成、実行内容未検証＝まだブートすらしてない状態）。

コメント欄に理解あやふや箇所の自己メモあり（`extern "C"`が何に対しての規約か等）→ 次回セッションで深掘り余地。

## 次アクション

bootloaderクレート統合 → bootable disk image生成 → QEMU実起動確認（画面表示はまだ、spin loopが実際CPU上で回ってる事の確認が目標）。

範囲外（今回やらない）: VGA出力、割り込み(IDT)、ページング、自作bootloader（[[INTENT]]参照、後回し方針）。
