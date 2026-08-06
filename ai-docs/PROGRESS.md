# PROGRESS

前提: [[INTENT]]必読。方針・判断理由そちら。

## 構造（現状）

```
edu-os/               root = "runner" crate（host向け、std使える）
├── Cargo.toml          bootloader + anyhow依存
├── src/main.rs          kernel build → disk image化 → QEMU起動、を実行する側（実装中）
├── ai-docs/
└── kernel/             bare-metal kernel本体、独立ビルド可能
    ├── .cargo/config.toml   [build] target = "x86_64-unknown-none" 固定
    ├── Cargo.toml            bootloader_api依存
    └── src/main.rs           entry_point!(kernel_main)、中身spin loopのみ
```

workspace化はしていない（意図的、下記参照）。root/kernelは別々の独立cargoプロジェクト。

## bootloader統合の設計判断

公式README方式（cargo workspace + artifact-dependency `[unstable] bindeps = true`）は不採用。

**Why:** unstable機能でnightly依存強まる上、workspace内でtarget混在（root=host, kernel=bare-metal）するとrust-analyzerの target解決が混乱しやすい（既にhost/bare-metal target衝突で一度躓いた実績あり）。root/kernel完全分離＋`std::process::Command`でkernel buildを呼ぶ方式ならtargetの取り違えが構造的に起きない。README内でも正式な代替手段として明記されてる。

**How to apply:** kernel側に新しいbuild成果物や設定を足す時、rootのworkspace化を持ち出さない。root→kernelの連携は常にCommand越し。

## kernel/src/main.rs 現状

freestanding化済み。`entry_point!(kernel_main)`マクロ使用（`_start`手書きは廃止）、`kernel_main`中身は無限spin loop、`#[panic_handler]`自前定義。

`cd kernel && cargo build --target x86_64-unknown-none` 成功確認済み。実行内容（実際にCPU上で動くか）は未検証＝まだブートすらしてない状態。

## root/src/main.rs 現状

未完成、ユーザー実装中。設計: `Command`でkernelを`cargo build`→`kernel/target/x86_64-unknown-none/debug/edu-os`のELF取得→`bootloader::BiosBoot::new(...).create_disk_image(...)`でdisk image化→`qemu-system-x86_64`起動、の4段階。

ユーザーはRust初心者、`Command` / `?`演算子 / `anyhow::Result` / `anyhow::ensure!`が今回新規登場、理解途上。

## AI運用の補足（[[INTENT]]に追記）

コードファイルの直接編集はしない（Write/Edit禁止、提示のみ）。例外: ディレクトリ構造のmvなど、ユーザーが明示的に依頼した非コード・機械的操作は可（例: `kernel/`への移動は実施済み）。

## 次アクション

root/src/main.rs完成 → `cargo run`（root側）で disk image生成 → QEMU実起動確認（画面表示はまだ、spin loopが実際CPU上で回ってる事の確認が目標）。

範囲外（今回やらない）: VGA出力、割り込み(IDT)、ページング、自作bootloader（[[INTENT]]参照、後回し方針）。
