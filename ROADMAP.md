# Roadmap

ProjectABE (JavaScript, Web) との機能差分と、バージョンアップ計画。

## ProjectABE との機能比較

### CPU / コア

| 機能 | ProjectABE | arduboy-emu 0.4.0 | 差分 |
|------|:---:|:---:|------|
| AVR 命令セット | ~100 命令 | 80+ 命令 | ほぼ同等 |
| ELPM (24-bit flash) | ✗ | ✓ | arduboy-emu が上回る |
| JIT コンパイル (命令→JS) | ✓ | ✗ | ProjectABE は JIT で高速化 |
| ブレークポイント | ✓ | ✓ | **v0.2.0 で追加** |
| ステップ実行 | ✓ | ✓ | **v0.2.0 で追加** |
| レジスタ/RAM ウォッチ | ✓ | ✓ (レジスタのみ) | **v0.2.0 で追加** |
| 逆アセンブラ | ✓ | ✓ | **v0.2.0 で追加** |
| ソースマップ連携 | ✓ | ✗ | コンパイラ連携なし |
| ATmega328P 対応 | ✓ | ✓ (v0.5.0) | — |
| Web Worker 分離 | ✓ | ✗ | シングルスレッド |

### ディスプレイ

| 機能 | ProjectABE | arduboy-emu 0.4.0 | 差分 |
|------|:---:|:---:|------|
| SSD1306 (128×64) | ✓ | ✓ | 同等 |
| PCD8544 / Gamebuino | ✗ | ✓ | arduboy-emu が上回る |
| ディスプレイ反転 (INVERT) | ✓ | ✓ | **v0.2.0 で追加** |
| コントラスト制御 | ✓ | ✓ | **v0.2.0 で追加** |
| スケール切替 (1×–6×) | ✓ (2 段階) | ✓ (6 段階) | **v0.2.0 で追加** |
| フルスクリーン | ✓ | ✓ | **v0.2.0 で追加** |

### オーディオ

| 機能 | ProjectABE | arduboy-emu 0.4.0 | 差分 |
|------|:---:|:---:|------|
| Timer3 CTC トーン | ✓ | ✓ | 同等 |
| Timer1 CTC トーン | ✓ | ✓ | 同等 |
| Timer4 トーン | ✗ | ✓ | **v0.3.0 で追加** |
| GPIO ビットバング | ✓ (pin callback) | ✓ (PC6+PB5 edge detect) | 同等 |
| 2ch ステレオ出力 | ✓ | ✓ | **v0.2.0 で追加** |
| サンプル精度波形バッファ | ✓ | ✓ | **v0.3.0 で追加** |

### ペリフェラル

| 機能 | ProjectABE | arduboy-emu 0.4.0 | 差分 |
|------|:---:|:---:|------|
| Timer0 (8-bit) | ✓ | ✓ | 同等 |
| Timer1 (16-bit) | ✓ | ✓ | 同等 |
| Timer3 (16-bit) | ✓ | ✓ | 同等 |
| Timer4 (10-bit 高速) | ✗ | ✓ | **v0.3.0 で追加** |
| SPI | ✓ | ✓ | 同等 |
| ADC | ✓ | ✓ | 同等 |
| PLL | ✓ | ✓ | 同等 |
| EEPROM | ✓ | ✓ | 同等 |
| USB Serial エミュレーション | ✓ | ✓ | **v0.2.0 で追加** |
| FX Flash (W25Q128) | ✗ | ✓ | arduboy-emu が上回る |
| 外部ペリフェラル (HC-SR04 等) | ✓ | ✗ | センサーエミュなし |
| RGB LED | ✓ | ✓ | **v0.4.0 で追加** |

### UI / UX

| 機能 | ProjectABE | arduboy-emu 0.4.0 | 差分 |
|------|:---:|:---:|------|
| Web ブラウザ動作 | ✓ | ✗ | デスクトップ専用 |
| モバイル対応 | ✓ (Cordova) | ✗ | — |
| Electron デスクトップ | ✓ | ✗ (ネイティブ) | ネイティブの方が軽量 |
| ゲームパッド対応 | ✗ | ✓ | arduboy-emu が上回る |
| スキンシステム | ✓ (7種) | ✗ | Arduboy/Microcard/Pipboy/Tama 等 |
| ドラッグ＆ドロップ読込 | ✓ | ✗ | N/P ゲームブラウザで代替 |
| .arduboy ファイル読込 | ✓ | ✓ | **v0.4.0 で追加** |
| GIF 録画 | ✓ | ✓ | **v0.4.0 で追加** |
| PNG スクリーンショット | ✓ | ✓ (任意倍率) | **v0.4.0 で追加** |
| EEPROM 永続化 | ✓ | ✓ (自動保存) | **v0.4.0 で追加** |
| ゲームブラウザ | ✓ (リポジトリ) | ✓ (ディレクトリ) | **v0.4.0 で追加** |
| ゲームリポジトリブラウザ | ✓ | ✗ | eriedリポジトリ統合 |
| クラウドコンパイラ | ✓ | ✗ | ソースコードコンパイルなし |
| 実機書き込み (AVRGirl) | ✓ | ✗ | USB flasher なし |
| QR コード生成 | ✓ | ✗ | — |
| キーボード入力 | ✓ | ✓ | 同等 |
| ミュートトグル | ✓ (M) | ✓ (M) | 同等 |
| FPS リミッタ | ✓ | ✓ | **v0.4.0 で追加** |

### まとめ (v0.5.0 時点)

| カテゴリ | ProjectABE が上回る | 同等 | arduboy-emu が上回る |
|----------|:---:|:---:|:---:|
| CPU/コア | 3 | 4 | 1 |
| ディスプレイ | 0 | 5 | 1 |
| オーディオ | 0 | 4 | 2 |
| ペリフェラル | 0 | 9 | 3 |
| UI/UX | 7 | 3 | 7 |
| **合計** | **10** | **25** | **14** |

**v0.1.0 → v0.5.0 での改善**: ProjectABE優位 24→10 (−14)、同等 13→25 (+12)、arduboy-emu優位 4→14 (+10)

---

## バージョンアップ計画

### ~~v0.2.0 — デバッグ基盤とディスプレイ強化~~ ✅ 完了

- [x] 逆アセンブラ (PC → 命令テキスト)
- [x] `--break <addr>` CLI ブレークポイント
- [x] ステップ実行 (`--step` モード)
- [x] レジスタ/SREG/SP ダンプ表示
- [x] SSD1306 Display Invert コマンド (0xA6/0xA7)
- [x] SSD1306 Contrast 制御 (0x81)
- [x] ウィンドウリサイズ / スケール切替 (1×, 2×, 3×, 4×, 5×, 6×)
- [x] フルスクリーン (F11)
- [x] スクリーンショット保存 (BMP, S キー)

### ~~v0.3.0 — オーディオ改善と USB Serial~~ ✅ 完了

- [x] 2ch ステレオ出力 (Speaker1: PC6, Speaker2: PB5)
- [x] サンプル精度波形バッファ (pin-change → AudioBuffer → PCM)
- [x] USB Serial エミュレーション (UEDATX → --serial stderr出力)
- [x] Serial Monitor 表示 (--serial フラグ)
- [x] Timer4 (10-bit 高速 PWM) — CTC/Normal/PWM モード

### ~~v0.4.0 — GUI フロントエンド~~ ✅ 完了

**目標**: ファイル操作とユーザー体験を ProjectABE 並みにする

- [x] .arduboy ZIP ファイルパーサ (info.json + hex + bin)
- [x] EEPROM 永続化 (自動保存/復元, .eep ファイル, 10秒間隔)
- [x] GIF 録画 (G キー, LZW 圧縮, フレームキャプチャ)
- [x] PNG スクリーンショット (S キー, 現在のスケール倍率で保存)
- [x] RGB LED / TX LED / RX LED 状態表示 (タイトルバー)
- [x] FPS リミッタ切替 (F キー: 60fps ↔ 無制限)
- [x] ゲームブラウザ (N=次 / P=前 / O=一覧, ディレクトリ内ファイルスキャン)
- [x] ホットリロード (R キー)

> ※ ネイティブ D&D は minifb 未対応のため、ディレクトリスキャン方式で代替。

### v0.5.0 — ATmega328P / Gamebuino Classic 対応

**目標**: ATmega328P CPU をサポートし、Gamebuino Classic のゲームを実行可能にする

- [x] `CpuType` enum (`Atmega32u4` / `Atmega328p`) と構成切替
- [x] ATmega328P メモリマップ (SRAM 2KB, `Memory::new_with_size()`)
- [x] ATmega328P 割り込みベクタテーブル (26 本)
- [x] Timer2 (8-bit 非同期) ペリフェラル (Timer8 再利用, 専用アドレス・ベクタ)
- [x] ポート制限 (PORTB/C/D のみ, Timer3/Timer4/USB は 32u4 限定)
- [x] Gamebuino Classic ボタンマッピング (UP=PB1, DOWN=PD6, LEFT=PB0, RIGHT=PD7, A=PD4, B=PD2)
- [x] `--cpu 328p` CLI オプション
- [x] PCD8544 SPI ルーティング (CS=PC2, DC=PC3)

### v0.6.0 — 高度なデバッグ機能

**目標**: 本格的な開発ツールとしてのデバッグ環境

- [ ] RAM ビューア (リアルタイム可視化)
- [ ] I/O レジスタビューア (名前付き表示)
- [ ] メモリブレークポイント (ウォッチポイント)
- [ ] 実行プロファイラ (PC ヒストグラム → ホットスポット)
- [ ] ELF / DWARF デバッグ情報読込
- [ ] ソースレベルデバッグ (C/C++ 行 ↔ PC マッピング)
- [ ] GDB Remote Serial Protocol サーバ (gdb-server)

### v0.7.0 — エコシステム統合

**目標**: Arduboy コミュニティのエコシステムと連携する

- [ ] ゲームリポジトリブラウザ (eried/ArduboyCollection JSON)
- [ ] ゲームリスト/サムネイル表示
- [ ] ワンクリック起動
- [ ] スキンシステム (Arduboy / Microcard / Pipboy / Tama)

### v0.8.0 — Web フロントエンド

**目標**: ブラウザで動作する Web 版を提供する

- [ ] WebAssembly (wasm32) ビルド対応 (core crate)
- [ ] HTML/Canvas フロントエンド
- [ ] Web Audio API によるオーディオ出力
- [ ] キーボード入力 (Web)
- [ ] ゲームパッド API (Web Gamepad API)
- [ ] URL パラメータで .hex 読込
- [ ] ドラッグ＆ドロップ ファイル読込 (Web)

### v1.0.0 — 安定版リリース

**目標**: 完成度を高め、安定版として公開する

- [ ] 全 AVR 命令の実装完了とテスト
- [ ] ProjectABE 互換性テストスイート
- [ ] CI/CD パイプライン
- [ ] crates.io 公開 (arduboy-core)
- [ ] GitHub Releases バイナリ配布 (Linux/macOS/Windows)
- [ ] ドキュメント整備 (`cargo doc`)
- [ ] Web 版の公開ホスティング
