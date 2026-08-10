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
    ├── Cargo.toml            bootloader_api, noto-sans-mono-bitmap, lazy_static, x86_64依存
    └── src/main.rs           entry_point!(kernel_main)、framebuffer文字描画、IDT+breakpoint例外
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
6. IDT + breakpoint例外ハンドラ → 完了。`x86_64`crate + `lazy_static`(`spin_no_std`feature)追加、`extern "x86-interrupt" fn breakpoint_handler`登録、`IDT.load()`→`int3()`→後続テキスト描画まで到達確認（QEMU再起動ループにならず成功）

はまった点:
- bootloader自身が起動ログをframebufferに直接描画してて(`bootloader-x86_64-common`が内部で同じ`noto-sans-mono-bitmap`使用)、kernel突入後もその描画が残る。`BootloaderConfig`にログ無効化オプション無し→kernel側で`buffer.fill(0)`して描画前にクリアする方式で対処
- `lazy_static!`マクロは`static ref NAME: TYPE = 式;`構文、ブロック式の後にも`;`要る（忘れがちなので注意）
- `extern "x86-interrupt"`はnightly限定unstable機能。`#![feature(abi_x86_interrupt)]`をcrate属性に追加必要

`BootInfo`経由のframebuffer情報: `width: 1280, height: 720, pixel_format: Bgr, bytes_per_pixel: 3`（環境依存、QEMU実行時ログで確認）。

先の話（未着手）: interrupt handler内からframebufferに文字を出すには、`boot_info`由来のframebufferをグローバル(static + Mutex等)で保持する必要が出てくる。現状kernel_mainのローカル変数のみなので、handlerからは触れない。page fault等の詳細をハンドラ内から画面表示したくなったら、この「グローバルmutable state」の壁に当たる→次の学習ポイント候補。

## 次アクション: 出力抽象化（Writer + println!マクロ）

着手中（プラン提示済み、コード未反映）。ユーザー発案「出力形態の抽象化ほしい」→[[INTENT]]の「先の話」で挙がってたグローバルframebuffer問題の解決も兼ねる。

プラン:
1. kernel/Cargo.tomlに`spin = "0.9.9"`追加
2. `draw_char`関数削除、代わりに`Writer`構造体（`buffer: &'static mut [u8]`, `info: FrameBufferInfo`, `x: usize`, `y: usize`）定義。`write_char`メソッドに文字1個描画＋カーソル移動ロジック集約（改行`\n`対応、画面幅超えたら自動改行も追加）
3. `Writer`に`core::fmt::Write`トレイト実装（`write_str`のみ）→ `write!`/`writeln!`が使えるようになる
4. `static WRITER: spin::Mutex<Option<Writer>> = Mutex::new(None);`グローバル定義
5. `println!`マクロを`macro_rules!`で自作、内部で`WRITER.lock()`→`writeln!`
6. `kernel_main`: `boot_info.framebuffer.take().unwrap()`で所有権ごと取得→`.into_buffer()`で`&'static mut [u8]`取得（`bootloader_api`公式メソッド、寿命問題なく`'static`が手に入る）→`WRITER`初期化→以後`println!("...")`で出力

効果: kernel_main側の座標管理コード消える。breakpoint_handler等interrupt handler内からも`println!`が呼べるようになる（[[PROGRESS]]既出の「先の話」の壁が解消）。

範囲外（今回やらない）: page fault等ハンドラ追加、ページング、自作bootloader（[[INTENT]]参照、後回し方針）。
