# Roadmap

ProjectABE (JavaScript, Web) との機能差分と、バージョンアップ計画。

## ProjectABE との機能比較

### CPU / コア

| 機能 | ProjectABE | arduboy-emu 0.3.0 | 差分 |
|------|:---:|:---:|------|
| AVR 命令セット | ~100 命令 | 80+ 命令 | ほぼ同等 |
| ELPM (24-bit flash) | ✗ | ✓ | arduboy-emu が上回る |
| JIT コンパイル (命令→JS) | ✓ | ✗ | ProjectABE は JIT で高速化 |
| ブレークポイント | ✓ | ✓ | **v0.2.0 で追加** |
| ステップ実行 | ✓ | ✓ | **v0.2.0 で追加** |
| レジスタ/RAM ウォッチ | ✓ | ✓ (レジスタのみ) | **v0.2.0 で追加** |
| 逆アセンブラ | ✓ | ✓ | **v0.2.0 で追加** |
| ソースマップ連携 | ✓ | ✗ | コンパイラ連携なし |
| ATmega328P 対応 | ✓ | ✗ | 32u4 のみ |
| Web Worker 分離 | ✓ | ✗ | シングルスレッド |

### ディスプレイ

| 機能 | ProjectABE | arduboy-emu 0.3.0 | 差分 |
|------|:---:|:---:|------|
| SSD1306 (128×64) | ✓ | ✓ | 同等 |
| PCD8544 / Gamebuino | ✗ | ✓ | arduboy-emu が上回る |
| ディスプレイ反転 (INVERT) | ✓ | ✓ | **v0.2.0 で追加** |
| コントラスト制御 | ✓ | ✓ | **v0.2.0 で追加** |
| スケール切替 (1×–6×) | ✓ (2 段階) | ✓ (6 段階) | **v0.2.0 で追加** |
| フルスクリーン | ✓ | ✓ | **v0.2.0 で追加** |

### オーディオ

| 機能 | ProjectABE | arduboy-emu 0.3.0 | 差分 |
|------|:---:|:---:|------|
| Timer3 CTC トーン | ✓ | ✓ | 同等 |
| Timer1 CTC トーン | ✓ | ✓ | 同等 |
| Timer4 トーン | ✗ | ✓ | **v0.3.0 で追加** |
| GPIO ビットバング | ✓ (pin callback) | ✓ (PC6+PB5 edge detect) | 同等 |
| 2ch ステレオ出力 | ✓ | ✓ | **v0.2.0 で追加** |
| サンプル精度波形バッファ | ✓ | ✓ | **v0.3.0 で追加** |

### ペリフェラル

| 機能 | ProjectABE | arduboy-emu 0.3.0 | 差分 |
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
| RGB LED | ✓ | ✗ | LED 状態追跡なし |

### UI / UX

| 機能 | ProjectABE | arduboy-emu 0.3.0 | 差分 |
|------|:---:|:---:|------|
| Web ブラウザ動作 | ✓ | ✗ | デスクトップ専用 |
| モバイル対応 | ✓ (Cordova) | ✗ | — |
| Electron デスクトップ | ✓ | ✗ (ネイティブ) | ネイティブの方が軽量 |
| ゲームパッド対応 | ✗ | ✓ | arduboy-emu が上回る |
| スキンシステム | ✓ (7種) | ✗ | Arduboy/Microcard/Pipboy/Tama 等 |
| ドラッグ＆ドロップ読込 | ✓ | ✗ | CLI のみ |
| .arduboy ファイル読込 | ✓ | ✗ | .hex + .bin のみ |
| GIF 録画 | ✓ | ✗ | 未実装 |
| スクリーンショット保存 | ✓ | ✓ (BMP) | **v0.2.0 で追加** |
| ゲームリポジトリブラウザ | ✓ | ✗ | eriedリポジトリ統合 |
| クラウドコンパイラ | ✓ | ✗ | ソースコードコンパイルなし |
| 実機書き込み (AVRGirl) | ✓ | ✗ | USB flasher なし |
| QR コード生成 | ✓ | ✗ | — |
| キーボード入力 | ✓ | ✓ | 同等 |
| ミュートトグル | ✓ (M) | ✓ (M) | 同等 |

### まとめ (v0.3.0 時点)

| カテゴリ | ProjectABE が上回る | 同等 | arduboy-emu が上回る |
|----------|:---:|:---:|:---:|
| CPU/コア | 3 | 4 | 1 |
| ディスプレイ | 0 | 5 | 1 |
| オーディオ | 0 | 4 | 2 |
| ペリフェラル | 2 | 8 | 2 |
| UI/UX | 8 | 3 | 1 |
| **合計** | **13** | **24** | **7** |

**v0.1.0 → v0.3.0 での改善**: ProjectABE優位 24→13 (−11)、同等 13→24 (+11)、arduboy-emu優位 4→7 (+3)

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

### v0.4.0 — GUI フロントエンド

**目標**: ファイル操作とユーザー体験を ProjectABE 並みにする

- [ ] ドラッグ＆ドロップで .hex / .bin / .arduboy ファイル読込
- [ ] .arduboy ZIP ファイルパーサ (info.json + hex + bin)
- [ ] EEPROM 永続化 (ファイル保存/復元)
- [ ] GIF 録画 (フレームキャプチャ → GIF エンコード)
- [ ] PNG スクリーンショット (任意の倍率)
- [ ] RGB LED 状態表示 (TXLED, RXLED)
- [ ] FPS リミッタ切替 (通常 60fps / 無制限)

### v0.5.0 — Web フロントエンド

**目標**: ブラウザで動作する Web 版を提供する

- [ ] WebAssembly (wasm32) ビルド対応 (core crate)
- [ ] HTML/Canvas フロントエンド
- [ ] Web Audio API によるオーディオ出力
- [ ] キーボード入力 (Web)
- [ ] ゲームパッド API (Web Gamepad API)
- [ ] URL パラメータで .hex 読込

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
- [ ] ATmega328P サポート (Arduino Uno/Gamebuino Classic)
- [ ] スキンシステム (Arduboy / Microcard / Pipboy / Tama)

### v1.0.0 — 安定版リリース

**目標**: 完成度を高め、安定版として公開する

- [ ] 全 AVR 命令の実装完了とテスト
- [ ] ProjectABE 互換性テストスイート
- [ ] CI/CD パイプライン
- [ ] crates.io 公開 (arduboy-core)
- [ ] GitHub Releases バイナリ配布 (Linux/macOS/Windows)
- [ ] ドキュメント整備 (`cargo doc`)
- [ ] Web 版の公開ホスティング
