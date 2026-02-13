# arduboy-emu

**v0.5.0** — Rust で書かれたサイクル精度の Arduboy エミュレータです。

ATmega32u4（Arduboy）と ATmega328P（Gamebuino Classic）マイコン（16 MHz）をディスプレイ、オーディオ、ゲームパッド、Arduboy FX フラッシュ対応でエミュレートします。

## 特徴

- **デュアル CPU 対応** — ATmega32u4（デフォルト）と ATmega328P（`--cpu 328p`）
- **AVR CPU コア** — 80以上の命令を正確なフラグ計算で実装（ADD、SUB、SBC/SBCI キャリーチェーン、MUL 等）
- **SSD1306 OLED ディスプレイ** — 128×64 モノクロ、水平/垂直アドレッシングモード、コントラスト制御、反転表示
- **PCD8544 LCD** — 84×48 Nokia ディスプレイ（Gamebuino Classic 互換、自動検出、328P ではデフォルト）
- **ステレオオーディオ** — サンプル精度波形レンダリングによる2チャンネル独立出力：
  - 左: Timer3 CTC / Timer4 CTC / GPIO ビットバング (PC6)
  - 右: Timer1 CTC / GPIO ビットバング (PB5)
  - ハイブリッド音声: GPIO ビットバングはサンプル精度PCM、タイマー駆動は矩形波合成フォールバック
- **ゲームパッド対応** — gilrs によるクロスプラットフォーム対応（Windows/Linux/macOS）、ホットプラグ対応
- **Arduboy FX** — W25Q128 16 MB SPI フラッシュエミュレーション（Read、Fast Read、JEDEC ID、消去、書込）
- **ペリフェラル** — Timer0/1/2/3/4、SPI、ADC、PLL、EEPROM、USB Serial 出力
- **デバッガ** — 逆アセンブラ、ブレークポイント、ステップ実行、レジスタダンプ
- **動的表示** — スケール 1×–6× 切替、フルスクリーン、PNG スクリーンショット（現在の倍率で保存）
- **USB Serial** — UEDATX レジスタ経由で `Serial.print()` 出力をキャプチャ（32u4 のみ）
- **ヘッドレスモード** — フレームスナップショットと診断情報による自動テスト
- **.arduboy ファイル対応** — ZIP アーカイブ（info.json + hex + FX bin）を直接読込
- **EEPROM 永続化** — ゲーム横に .eep ファイルとして自動保存/復元
- **GIF 録画** — ゲームプレイをアニメーション GIF で録画（G キーでトグル、LZW 圧縮）
- **LED 状態表示** — RGB LED、TX LED、RX LED の状態をタイトルバーに表示
- **FPS 制御** — 60fps 固定と無制限を切替（F キー）
- **ホットリロード** — 実行中にゲームファイルを再読込（R キー）
- **ゲームブラウザ** — N/P キーでディレクトリ内のゲームを切替、O で一覧表示

## ビルド

```bash
# Linux: 依存パッケージのインストール
sudo apt install libudev-dev libasound2-dev

# ビルドと実行
cargo build --release
cargo run --release -- game.hex
```

## 使い方

```
arduboy-emu <file.hex|file.arduboy> [オプション]

オプション:
  --fx <file.bin>    FX フラッシュデータを読み込む
  --cpu <type>       CPU 種別: 32u4（デフォルト）または 328p（Gamebuino Classic）
  --mute             オーディオを無効化
  --debug            フレームごとの診断情報を表示
  --headless         GUI なしで実行
  --frames N         N フレーム実行（ヘッドレス、デフォルト 60）
  --press N          フレーム N で A ボタンを押す（ヘッドレス）
  --snapshot F       フレーム F でディスプレイを出力（複数指定可）
  --break <addr>     16進バイトアドレスにブレークポイント設定（複数指定可）
  --step             対話式ステップデバッガ
  --scale N          初期スケール 1-6（デフォルト 6）
  --serial           USB Serial 出力を stderr に表示
  --no-save          EEPROM 自動保存を無効化
```

### 対応ファイル形式

| 形式 | 説明 |
|------|------|
| `.hex` | Intel HEX バイナリ（同名の `.bin` / `-fx.bin` を FX データとして自動検出）|
| `.arduboy` | ZIP アーカイブ（`info.json`、`.hex`、FX `.bin` を含む）|

### FX フラッシュの自動検出

`.hex` ファイルと同名の `.bin` ファイルがあれば自動的に読み込まれます：

```
game.hex + game.bin       → 自動読込
game.hex + game-fx.bin    → 自動読込
game.hex --fx custom.bin  → 明示的なパス指定
game.arduboy              → ZIP から hex + fx を自動抽出
```

### EEPROM 永続化

EEPROM はゲームファイル横に `.eep` ファイルとして自動保存されます：

```
game.hex → game.eep（10秒ごと + 終了時に自動保存）
```

`--no-save` で無効化できます。ホットリロード（R キー）でも EEPROM は保持されます。

### ゲームブラウザ

**O** キーでゲームファイルのあるディレクトリ内の `.hex`/`.arduboy` ファイル一覧を表示し、
**N**（次）/ **P**（前）で切り替えられます。EEPROM はゲームごとに自動保存/復元されます。

```
--- Games in ./roms (5 found) ---
    1. arcodia.hex
    2. breakout.hex <<
    3. circuit-dude.arduboy
    4. nineteen44.hex
    5. starduino.hex
---
```

## 操作方法

| Arduboy       | キーボード | Xbox コントローラー          | PlayStation                   |
|---------------|------------|------------------------------|-------------------------------|
| 十字キー      | 矢印キー   | 十字キー / 左スティック       | 十字キー / 左スティック        |
| A             | Z          | X, Y, LB, RB, LT, RT, Select | □, △, L1, R1, L2, R2, Select |
| B             | X          | A, B, Start                  | ×, ○, Start                   |
| スケール 1×–6× | 1–6 キー  | —                            | —                             |
| フルスクリーン | F11        | —                            | —                             |
| スクリーンショット | S      | —                            | — (現在の倍率で PNG 保存)      |
| GIF 録画      | G          | —                            | —                             |
| 次のゲーム    | N          | —                            | —                             |
| 前のゲーム    | P          | —                            | —                             |
| ゲーム一覧    | O          | —                            | —                             |
| リロード      | R          | —                            | —                             |
| FPS 無制限    | F          | —                            | — (60fps ↔ 無制限)            |
| レジスタダンプ | D          | —                            | —                             |
| ミュート      | M          | —                            | —                             |
| ぼかし        | B          | —                            | — (ドットをわずかに平滑化)     |
| 液晶エフェクト | L          | —                            | — (実機風カラー・グリッド・残像) |
| 終了          | Escape     | —                            | —                             |

キーボードとゲームパッドの入力は OR 結合されるため、同時に使用できます。

## アーキテクチャ

```
arduboy-emu/
├── crates/
│   ├── core/                    # プラットフォーム非依存のエミュレーションコア
│   │   └── src/
│   │       ├── lib.rs           # Arduboy 構造体：トップレベルエミュレータ
│   │       ├── cpu.rs           # AVR CPU ステートと命令実行
│   │       ├── opcodes.rs       # 命令デコーダ（16/32ビット → enum）
│   │       ├── memory.rs        # データ空間、フラッシュ、EEPROM
│   │       ├── display.rs       # SSD1306 OLED コントローラ
│   │       ├── pcd8544.rs       # PCD8544 Nokia LCD コントローラ
│   │       ├── hex.rs           # Intel HEX パーサ
│   │       ├── disasm.rs        # 逆アセンブラ（デバッガ用）
│   │       ├── audio_buffer.rs  # サンプル精度波形バッファ
│   │       ├── arduboy_file.rs  # .arduboy ZIP ファイルパーサ
│   │       ├── png.rs           # PNG エンコーダ（依存なし）
│   │       ├── gif.rs           # アニメーション GIF エンコーダ（LZW 圧縮）
│   │       └── peripherals/
│   │           ├── timer8.rs    # Timer/Counter0（millis/delay）
│   │           ├── timer16.rs   # Timer/Counter1 & 3（オーディオトーン）
│   │           ├── timer4.rs    # Timer/Counter4（10-bit 高速 PWM）
│   │           ├── spi.rs       # SPI マスターコントローラ
│   │           ├── adc.rs       # ADC（乱数シード）
│   │           ├── pll.rs       # PLL 周波数シンセサイザ
│   │           ├── eeprom.rs    # EEPROM コントローラ
│   │           └── fx_flash.rs  # W25Q128 外部フラッシュ（16 MB）
│   └── frontend-minifb/         # デスクトップフロントエンド
│       └── src/main.rs          # ウィンドウ、オーディオ、ゲームパッド、CLI
└── roms/                        # テスト ROM ディレクトリ
```

### エミュレーションループ

1フレームごと（60 FPS で約 13.5 ms）：

1. キーボードとゲームパッドをポーリング → GPIO ピン状態を設定
2. 216,000 サイクル分の CPU 命令を実行
3. SPI バッファをフラッシュ → ディスプレイまたは FX フラッシュにルーティング
4. タイマーを更新し、保留中の割り込みを発火
5. トーン周波数を取得（Timer3 / Timer1 / GPIO）→ オーディオスレッドに反映
6. RGBA フレームバッファを 6 倍スケールでウィンドウに描画

### オーディオ検出（ステレオ、サンプル精度）

GPIO ビットバングはフレームごとのエッジバッファでサンプル精度レンダリング。
タイマー駆動のオーディオは周波数ベースの矩形波合成にフォールバック。

| チャンネル | 優先度 | 方式 | メカニズム | 対応ゲーム例 |
|-----------|--------|------|------------|-------------|
| 左 | 1 | Timer3 CTC | コンペアマッチで OC3A トグル | Arduboy2 ライブラリ使用ゲーム全般 |
| 左 | 2 | Timer4 CTC | コンペアマッチで OC4A トグル | PWM オーディオゲーム |
| 左 | 3 | GPIO ビットバング | PORTC ビット6 の直接トグル | Arcodia |
| 右 | 1 | Timer1 CTC | Timer1 で同方式 | デュアルトーンゲーム |
| 右 | 2 | GPIO ビットバング | PORTB ビット5 の直接トグル | カスタムエンジン |

## テスト済みゲーム

- **Nineteen44** — スクロールシューティング（Timer3 オーディオ、複雑な SPI 制御）
- **Arcodia** — スペースインベーダー風（GPIO ビットバングオーディオ）
- **101 Starships** — 艦隊管理ゲーム
- その他 Arduboy2 ライブラリ使用ゲーム各種

## ロードマップ

ProjectABE との詳細な機能比較と v1.0.0 までの開発フェーズは
[ROADMAP.md](ROADMAP.md) を参照してください。

リリース履歴は [CHANGELOG.md](CHANGELOG.md) を参照してください。

## 注意事項

本ソフトウェアは、人間のオペレーターとの対話的な開発セッションを通じて
AI（Claude by Anthropic）により生成されました。既存のエミュレータプロジェクト
（ProjectABE 等）のコードは一切使用していません。実装は公開されている
ハードウェアデータシート（ATmega32u4、SSD1306、PCD8544、W25Q128）および
Intel HEX フォーマット仕様のみに基づいています。

## ライセンス

以下のいずれかのライセンスの下で提供されます（選択可）：

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) または <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT License ([LICENSE-MIT](LICENSE-MIT) または <http://opensource.org/licenses/MIT>)

### コントリビューション

明示的に別段の定めがない限り、あなたが本プロジェクトに対して意図的に提出した
コントリビューションは、Apache-2.0 ライセンスの定義に従い、上記のデュアル
ライセンスの下で提供されるものとします。追加の条件や条項はありません。
