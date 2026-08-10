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
4. framebuffer単色塗りつぶし → 完了
5. framebuffer文字描画 → 完了。`noto-sans-mono-bitmap`crate（kernel/Cargo.tomlに追加）の`get_raster`でbitmap取得、pixel輝度をB/G/R全chに書き込み。`draw_char`関数で1文字描画、x座標をglyph幅分進めながら文字列描画

はまった点: bootloader自身が起動ログをframebufferに直接描画してて(`bootloader-x86_64-common`が内部で同じ`noto-sans-mono-bitmap`使用)、kernel突入後もその描画が残る。`BootloaderConfig`にログ無効化オプション無し→kernel側で`buffer.fill(0)`して描画前にクリアする方式で対処。

`BootInfo`経由のframebuffer情報: `width: 1280, height: 720, pixel_format: Bgr, bytes_per_pixel: 3`（環境依存、QEMU実行時ログで確認）。

## 次アクション: IDT（割り込み記述子テーブル）、breakpoint例外

着手前（プラン提示済み、コード未反映）。目的: 今panic時は無言でspinするだけ、例外ハンドラの土台作る。まずbreakpoint(`int3`)だけ、安全な例外1個で疎通確認。

プラン:
1. kernel/Cargo.tomlに追加: `x86_64 = "0.15.5"`, `lazy_static = { version = "1.5.0", features = ["spin_no_std"] }`
2. `InterruptDescriptorTable::new()`はconst fnじゃないが`.load()`は`&'static self`要求→`lazy_static!`マクロで実行時初期化+static化
3. `extern "x86-interrupt" fn breakpoint_handler(...)`定義、`idt.breakpoint.set_handler_fn(...)`で登録
4. `kernel_main`で`IDT.load()`→`x86_64::instructions::interrupts::int3()`→その後にもう1行テキスト描画
5. 検証: 2行目のテキストが出れば成功（例外ハンドラが処理して普通に処理続行した証拠）。IDT未設定/ミスなら例外→handler無し→double fault→triple fault→**QEMU自体が再起動ループ**する（bootログが繰り返し出る）、これが失敗の見た目

先の話（今回のスコープ外）: interrupt handler内からframebufferに文字を出すには、`boot_info`由来のframebufferをグローバル(static + Mutex等)で保持する必要が出てくる。現状kernel_mainのローカル変数のみなので、handlerからは触れない。将来pageフォルト等の詳細をハンドラ内から画面表示したくなったら、この「グローバルmutable state」の壁に当たる→そこが次の次の学習ポイント。

範囲外（今回やらない）: ページング、自作bootloader（[[INTENT]]参照、後回し方針）。
