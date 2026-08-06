# PROGRESS

前提: [[INTENT]]必読。方針・判断理由そちら。

## 構造（現状）

```
edu-os/               root = "runner" crate（host向け、std使える）
├── Cargo.toml          bootloader(bios機能のみ) + anyhow依存
├── src/main.rs          kernel build → disk image化 → QEMU起動、を実行する側
├── ai-docs/
└── kernel/             bare-metal kernel本体、独立ビルド可能
    ├── .cargo/config.toml   [build] target = "x86_64-unknown-none" 固定
    ├── Cargo.toml            bootloader_api依存
    └── src/main.rs           entry_point!(kernel_main)、framebuffer単色塗りつぶし
```

workspace化はしていない（意図的、下記参照）。root/kernelは別々の独立cargoプロジェクト。

## bootloader統合の設計判断

公式README方式（cargo workspace + artifact-dependency `[unstable] bindeps = true`）は不採用。

**Why:** unstable機能でnightly依存強まる上、workspace内でtarget混在（root=host, kernel=bare-metal）するとrust-analyzerの target解決が混乱しやすい（既にhost/bare-metal target衝突で一度躓いた実績あり）。root/kernel完全分離＋`std::process::Command`でkernel buildを呼ぶ方式ならtargetの取り違えが構造的に起きない。README内でも正式な代替手段として明記されてる。

**How to apply:** kernel側に新しいbuild成果物や設定を足す時、rootのworkspace化を持ち出さない。root→kernelの連携は常にCommand越し。

## はまった点（既出、再発済み注意）

`bootloader`crateのdefault featuresに`uefi`含まれてて、環境のnightly/lldでは`wcslen`未解決でlink失敗する。`Cargo.toml`で

```toml
bootloader = { version = "0.11.17", default-features = false, features = ["bios"] }
```

これが必須。一度直したが誤って`default-features`指定なしに戻り再発した実績あり→この行を消さないよう注意。

rootで`cargo build --target x86_64-unknown-none`を誤実行すると、std前提のroot依存crate(serde等)がbare-metal target向けにビルドされ大量`not found`エラーになる。`--target x86_64-unknown-none`はkernel/配下限定というルールをREADME.mdに明記済み。

## マイルストーン

1. kernel freestanding化（`entry_point!`マクロ、`#[panic_handler]`）→ 完了
2. root/src/main.rs（Command経由でkernel build→BiosBoot::create_disk_image→qemu-system-x86_64起動）→ 完了
3. `cargo run`（root）でQEMU起動確認 → 完了。bootloaderの起動ログ出力後「Jumping to kernel entry point」→`kernel_main`到達確認
4. framebuffer単色塗りつぶし → 完了。`boot_info.framebuffer.as_mut().unwrap()`→`chunks_exact_mut(bytes_per_pixel)`でpixel単位に緑色書き込み、QEMU上で緑一色表示確認

`BootInfo`経由のframebuffer情報: `width: 1280, height: 720, pixel_format: Bgr, bytes_per_pixel: 3`（環境依存、QEMU実行時ログで確認）。

## 次アクション

未定（次セッションでユーザーと相談）。候補: framebufferへの文字描画（bitmap font要、Oppermann旧チュートリアルのVGAテキストバッファ方式とは別物）、CPU例外ハンドラ(IDT)。

範囲外（今回やらない）: ページング、自作bootloader（[[INTENT]]参照、後回し方針）。
