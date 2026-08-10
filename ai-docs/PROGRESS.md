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
    ├── Cargo.toml            bootloader_api, noto-sans-mono-bitmap, lazy_static, x86_64, spin依存
    └── src/
        ├── main.rs           entry_point!(kernel_main)、IDT+breakpoint/double fault例外
        ├── writer.rs         Writer構造体(framebuffer出力)、println!マクロ
        └── gdt.rs            GDT+TSS+IST(double fault専用スタック)
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
5. framebuffer文字描画 → 完了（後に6でWriter抽象化に置き換え）
6. IDT + breakpoint例外ハンドラ → 完了。`x86_64`crate + `lazy_static`(`spin_no_std`feature)追加、`extern "x86-interrupt" fn breakpoint_handler`登録、`IDT.load()`→`int3()`→後続テキスト描画まで到達確認（QEMU再起動ループにならず成功）
7. 出力抽象化（`Writer` + `println!`マクロ）→ 完了。`spin::Mutex`でグローバル保持、`core::fmt::Write`実装で`writeln!`経由の出力可能に。以後kernel_main/interrupt handlerどこからでも`println!("...")`で画面出力できる状態
8. `writer.rs`へモジュール分離 → 完了。`Writer`構造体・`println!`マクロを`kernel/src/writer.rs`に移動、`main.rs`は`mod writer;`のみ
9. Double Fault handler + IST → 完了。`kernel/src/gdt.rs`新規(TSS+GDT、IST0にdouble fault専用スタック確保)、`kernel_main`冒頭で`gdt::init()`(IDT.load()より前)、IDTに`double_fault.set_handler_fn(...).set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX)`登録。故意に未マップ番地書込み(`0xdeadbeef`)→page fault未処理→double fault発生→ハンドラでログ出力し停止、再起動ループにならないこと確認済み
10. Hardware割込み(PIC + timer + keyboard) → 完了。`pic8259`crate追加、`kernel/src/interrupts.rs`新規(`ChainedPics` static、`InterruptIndex` enum でPIC_1_OFFSET=32起点の割込み番号管理)。timer/keyboard共にIDT登録、handler内`notify_end_of_interrupt`必須(EOI忘れると2回目以降割込み来ない)。keyboardは`Port::new(0x60)`から生scancode読むだけ(pc-keyboardでのASCIIデコードは未実施、次アクション)。初期化順序: `gdt::init()`→`IDT.load()`→`PICS.lock().initialize()`→`interrupts::enable()`(sti)。この順序を守らないと(特にIDT.loadより先にsti)未定義動作になる

はまった点:
- GDT入替後、CSは`CS::set_reg()`で明示的に張り替えたがSSを放置→bootloaderが設定した古いSSセレクタの数値が、新GDTでは偶然TSSディスクリプタのスロットと一致→timer割込みでスタック操作時にGP fault(#13)発生。`SS::set_reg(SegmentSelector::NULL)`で対処(long mode ring0ではSS=NULLでも合法)。**GDT入替時はCSだけでなくSS(理想はDS/ES/FSも)を明示的に張り直すこと**、が教訓
- この手のバグはQEMU headless実行+`-d int,cpu_reset`(割込み/例外/リセットログ出力)で原因特定できた。GUIで単に「反応ない」だけだと分からない → 詰まったら`qemu-system-x86_64 -drive format=raw,file=target/disk.img -display none -d int,cpu_reset -no-reboot`が有効な調査手段
- IDTへのkeyboard handler登録で`idt[InterruptIndex::Timer.as_u8()]`をコピペし忘れて2箇所とも`Timer`のままにしたバグ発生。結果、keyboard_interrupt_handlerがtimerのentryを上書き→timer割込み毎にport 0x60を無条件read(ゴミ値`0xfa`連発)、本物のkeyboard IRQ1は未登録のまま→実キー押下でGP fault→DOUBLE FAULTに昇格。**IDTのindex指定は enumの意図した variant と実際渡してる値が一致してるか、登録直後に目視確認**する癖が要る
- bootloader自身が起動ログをframebufferに直接描画してて(`bootloader-x86_64-common`が内部で同じ`noto-sans-mono-bitmap`使用)、kernel突入後もその描画が残る。`BootloaderConfig`にログ無効化オプション無し→kernel側で`buffer.fill(0)`して描画前にクリアする方式で対処
- `lazy_static!`マクロは`static ref NAME: TYPE = 式;`構文、ブロック式の後にも`;`要る（忘れがちなので注意）
- `extern "x86-interrupt"`はnightly限定unstable機能。`#![feature(abi_x86_interrupt)]`をcrate属性に追加必要
- `#[macro_export]`な`macro_rules!`を別モジュールに置く場合、マクロ本体内の識別子は`$crate::module::NAME`絶対パスで書く（マクロは呼び出し元モジュールのスコープでテキスト展開されるため、裸の名前だと呼び出し元で解決されようとして壊れる）

`BootInfo`経由のframebuffer情報: `width: 1280, height: 720, pixel_format: Bgr, bytes_per_pixel: 3`（環境依存、QEMU実行時ログで確認）。

`Writer`構築時のframebuffer取得は`boot_info.framebuffer.as_mut().unwrap()` → `.buffer_mut()`（`&'static mut BootInfo`からの再借用が`'static`に落ち着く、所有権を奪う`.take()`+`.into_buffer()`への切替は未実施だが現状問題なく動作）。

## 次アクション

進行中: keyboard scancode→ASCIIデコード(`pc-keyboard`crate導入)。blog_os標準手順準拠、次点候補:
- ページング基礎〜実装
- page fault専用ハンドラ（現状はmissing entry経由でdouble faultに落ちてるだけ、CR2レジスタからfault番地読んで表示、等はまだ）
- Heap allocator

範囲外（今回やらない）: 自作bootloader（[[INTENT]]参照、後回し方針）。
