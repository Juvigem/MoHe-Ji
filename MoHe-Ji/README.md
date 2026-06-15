# MoHe-Ji

`MoHe-Ji` は、Rust + `eframe/egui` で作成した、PPW曲線を中心に扱うベクター/ラスター混在型の簡易イラストエディタです。

PPW曲線（Phi-Psi-Weight Curves）をポリライン化して描画し、点編集、塗りつぶし、SVG保存/読込、PNG出力、ラスター描画、画像配置などを扱えるようにしています。

---

## 主な機能

### ベクター機能

- PPW曲線パスの作成
- 点追加
- 点選択
- 点の範囲選択
- 複数点のまとめて移動
- パス線の選択
- 選択中パス線の分割
- 長方形ツール
- 楕円ツール
- Stroke幅変更
- Stroke色変更
- Fill ON/OFF
- Fill色変更
- 閉じたパスの塗りつぶし
- 三角形分割表示
- SVG保存/読込
- 選択点のコピー/ペースト

### ラスター機能

- ラスターレイヤー
- ラスターブラシ
- ラスター消しゴム
- ブラシ太さ変更
- 消しゴム太さ変更
- ブラシ色・透明度変更
- SVG内へのラスター情報保存/読込

### 画像機能

- PNG/JPEG画像追加
- 画像をラスターデータとして保持
- 画像の移動
- 画像の拡大縮小
- 画像選択中に削除

### 表示・UI

- ダークモード切替
- キャンバスは常に白背景
- 左メニューバー開閉
- 左メニューバーのスクロール
- ツールバータブ
- 右クリックメニュー
- ズーム
- パン
- PNG書き出し

---

## ビルド方法

### 必要なもの

- Rust
- Cargo

Rustが未導入の場合は、Rust公式の `rustup` で導入してください。

### 実行

プロジェクトのルートで以下を実行します。

```bash
cargo run
```

リリースビルドする場合：

```bash
cargo run --release
```

---

## 依存クレート

`Cargo.toml` では主に以下を使用しています。

```toml
eframe = "0.33"
egui = "0.33"
image = "0.25"
rfd = "0.15"
```

- `eframe` / `egui`: GUI
- `image`: PNG/JPEG画像読込、PNG出力
- `rfd`: ファイル選択ダイアログ

---

## 基本操作

### キャンバス操作

| 操作 | 内容 |
|---|---|
| `+` | ズームイン |
| `-` | ズームアウト |
| ホイールクリック + ドラッグ | キャンバス移動 |
| `P` | 点表示のON/OFF |
| 右クリック | 現在タブに応じたメニュー表示 |

### ツール切替

| キー | 内容 |
|---|---|
| `F` | Selectツール |
| `V` | ベクターレイヤーではAdd Point、ラスターレイヤーではRaster Brush |
| `Space` | 状況に応じた削除/切替 |
| `1` | Toolsタブ |
| `2` | Fileタブ |
| `3` | Editタブ |
| `4` | Viewタブ |
| `5` | Pathタブ |
| `6` | Layerタブ |

---

## ツール

### Select

点、パス線、画像を選択するツールです。

できること：

- 点をクリック選択
- ドラッグで範囲選択
- 選択点をまとめて移動
- パス線をクリック選択
- 画像をクリック選択
- 画像をドラッグ移動
- 画像右下ハンドルでサイズ変更

ショートカット：

| 操作 | 内容 |
|---|---|
| `Space` | 選択点を削除。画像選択中なら画像削除 |
| `G` | 選択中のパス線を分割 |

### Add Point

ベクターレイヤー上で点を追加します。

| 操作 | 内容 |
|---|---|
| クリック | 現在のパスに点追加 |
| `Space` | 最後の点を削除 |

### Rectangle

ドラッグで長方形を作成します。

| 操作 | 内容 |
|---|---|
| ドラッグ | 長方形作成 |
| Shift + ドラッグ | 正方形作成 |

長方形はPPWパスとして作成されます。

### Ellipse

ドラッグで楕円を作成します。

| 操作 | 内容 |
|---|---|
| ドラッグ | 楕円作成 |
| Shift + ドラッグ | 正円作成 |

楕円もPPWパスとして作成されます。

### Raster Brush

ラスターレイヤー上に描画します。

| 操作 | 内容 |
|---|---|
| ドラッグ | ラスター描画 |
| `Space` | Raster Eraserへ切替 |

### Raster Eraser

ラスターレイヤー上の描画を消します。

| 操作 | 内容 |
|---|---|
| ドラッグ | ピクセル単位風に消去 |
| `Space` | Raster Brushへ切替 |

---

## PPWパラメータ編集

Selectツールで点を選択しているとき、以下のキーで編集対象を選び、マウスホイールで値を変更できます。

| キー | 編集対象 |
|---|---|
| `Q` | Weight |
| `W` | 前側 Psi |
| `E` | 次側 Psi |
| `S` | 前側 Phi |
| `D` | 次側 Phi |

編集対象はマウスカーソル横に表示されます。

複数点を選択している場合、選択された複数点に対してまとめて反映されます。

---

## Stroke / Brush / Eraser幅の編集

| キー | 内容 |
|---|---|
| `R` + マウスホイール | Stroke、Raster Brush、Raster Eraserの太さを変更 |

太さは対数的、つまり倍率的に変化します。  
細いときは細かく、太いときは大きく変化します。

---

## 色編集

色はHue / Brightness / Alphaで編集します。

### Fill色

| キー | 編集対象 |
|---|---|
| `Z` + ホイール | Fill Hue |
| `X` + ホイール | Fill Brightness |
| `C` + ホイール | Fill Alpha |

### Stroke / Raster Brush色

| キー | 編集対象 |
|---|---|
| `Shift + Z` + ホイール | Stroke/Raster Brush Hue |
| `Shift + X` + ホイール | Stroke/Raster Brush Brightness |
| `Shift + C` + ホイール | Stroke/Raster Brush Alpha |

---

## レイヤー

このエディタには2種類のレイヤーがあります。

### Vector Layer

PPW曲線、図形、塗りつぶし、Strokeを扱います。

### Raster Layer

Raster Brush、Raster Eraser、画像などのラスターデータを扱います。

レイヤー操作：

- Add Vector Layer
- Add Raster Layer
- Duplicate
- Delete Layer
- 表示/非表示
- Lock
- ドラッグによる並び替え

---

## ファイル操作

### SVG保存

Fileタブから保存できます。

保存対象：

- Canvasサイズ
- ベクターレイヤー
- PPWパス
- Stroke情報
- Fill情報
- ラスターレイヤー情報
- 画像情報

### SVG読込

Fileタブの「Load SVG...」または Ctrl+O でファイル選択ダイアログを開き、読み込むSVGファイルを指定できます。
選択したファイルのパスはSVG file path欄に反映されます。

### PNG出力

Canvas範囲をPNGとして出力できます。

設定できる内容：

- 保存フォルダ
- ファイル名
- 透明背景のON/OFF

PNG出力には閉じたパスのFillも含まれます。

---

## コピー/ペースト

Selectツールで点を選択している状態で使用します。

| 操作 | 内容 |
|---|---|
| `Copy Selected Points button` | 選択点の座標、Weight、Psi、Phiをコピー |
| `Paste Points button` | 現在のレイヤーへペースト |

別レイヤーへのペーストにも対応しています。

---

## キャンバスサイズ

Canvasサイズは設定可能です。

例：

```text
600px × 800px
```

PNG出力時は、このCanvas範囲が出力対象になります。

---

## 画面構成

### 上部ツールバー

タブ式です。

- Tools
- File
- Edit
- View
- Path
- Layer

数字キー `1〜6` でも切り替えられます。

### 左メニューバー

詳細設定を表示します。

- SVG
- Layers
- Tool Help
- Path / Stroke / PPW

上部の `Hide Panel` / `Show Panel` ボタンで開閉できます。

---

## ソース構成

```text
src/
├─ main.rs
├─ app/
│  ├─ mod.rs
│  ├─ update.rs
│  └─ view.rs
├─ io/
│  ├─ mod.rs
│  ├─ png_export.rs
│  └─ svg.rs
├─ model/
│  ├─ mod.rs
│  └─ document.rs
├─ ppw/
│  ├─ mod.rs
│  ├─ curve.rs
│  ├─ path.rs
│  ├─ polygon.rs
│  ├─ shape_util.rs
│  └─ vec2.rs
└─ render/
   ├─ mod.rs
   └─ canvas.rs
```

### `src/main.rs`

アプリケーション起動処理です。  
ウィンドウ名は `MoHe-Ji` です。

### `src/app/`

UI、ショートカット、アプリ状態を管理します。

- `mod.rs`: アプリ全体の状態、ツール、選択状態
- `view.rs`: UI表示、ツールバー、ショートカット
- `update.rs`: eframe update処理

### `src/model/`

ドキュメント、レイヤー、ラスター、画像などのデータ構造を定義します。

### `src/ppw/`

PPW曲線の計算処理です。

- PPWPath
- PPWCurve
- PPWPolygon
- 三角形分割補助
- Vec2

### `src/render/`

キャンバス描画、マウス操作、点選択、パス選択、画像操作などを処理します。

### `src/io/`

SVG保存/読込、PNG出力を処理します。

---

## 開発メモ

このプロジェクトは、まず動作確認しやすい簡易イラストエディタとして構成されています。  
大規模化する場合は、以下の分離を進めると保守しやすくなります。

- ツールごとの処理を `tools/` に分離
- 選択状態を `selection.rs` に分離
- Undo/Redoを `history.rs` に分離
- PNG/SVG出力をより厳密なレンダリングパイプラインへ統合
- ラスター描画をピクセルバッファ方式へ完全移行
- 画像の回転、トリミング、レイヤーマスク対応

---

## 注意点

- SVGはPPW曲線をそのまま標準SVGとして表現できないため、独自属性やポリライン化情報を併用する設計です。
- PNG出力はCanvas範囲を対象にします。
- ラスター機能は簡易実装のため、本格的なペイントソフトのような高性能ブラシエンジンではありません。
- 画像はラスターデータとして保持するため、大きい画像を大量に配置するとメモリ使用量が増えます。

---

## ライセンス

MIT License

## Application Icon

The window icon is loaded from `assets/app_icon.png` at startup.
