# sample-maid

魔法少女メイドカフェの3Dオンラインゲーム(Phase 0: 2Dスプライトの
プレースホルダー)。

[`open-directx`](https://github.com/aon-co-jp/open-directx) の
DXBC→SPIR-V翻訳済みスプライトパイプライン
(`directx-shader-translate` / `directx-graphics-vulkan`)を
Cargoの依存(GitHubの`open-directx`)として再利用し、実ウィンドウ+Vulkanスワップチェーンで
動作する。

## 現在のスコープ(Phase 0)

- メイドちゃん(自機)・助っ人「風天のトラさん」(寅次郎風キャラの
  オマージュ、常に初音ミク風フィギュアを持っている設定——実際の意匠
  再現ではなく独自デザイン)・魔女(敵)を1画面に描画。
- 雑魚敵3体が徘徊し、近づき続けると味方化する(「世界中の敵を
  お友だちにして大魔女を倒す」という原案の中核メカニクスの最小実証)。
  味方は自動で魔女を攻撃する。
- アクションボタン(協力プレイで共有):
  - Z/B または F1/F10 → 味方化(即時)
  - X/N または F2/F11 → 石化(魔女・雑魚敵の動きを一定時間停止)
  - C/M または F3/F12 → サンダーボルト(魔女に直接ダメージ)
  - 3ボタン同時押し → ブラックホール(全雑魚敵を即時味方化+大ダメージ)
  - いずれか2ボタン同時押し → 助っ人「風天のトラさん」登場
    (1ステージに1回まで)
- 1人プレイ: マウスで操作キャラクターを移動、Vキーでメイドちゃん/
  トラさんのどちらを操作するか選択可能。
- 2人プレイ(Tabキーで切替): テンキー矢印(Numpad4/6)で助っ人を移動。

オンライン通信・3Dモデル・複数プレイヤー間のネットワーク同期・実際の
攻撃/防御アニメーションは未実装(次回以降のロードマップ)。

## 実行方法

```bash
cargo run --release
```

Escapeで終了。

## ディレクトリ構成

`open-directx` はGitHubから自動取得されます(隣にcloneする必要はありません)。

## 注意 / Notice

登場する「風」のキャラクターは、特定の既存作品・人物・商品とは無関係の架空の設定です。実在のキャラクターの画像・音は使っていません。

The "-style" characters are fictional and unrelated to any existing work, person or product. No existing character images or sounds are used.

## ライセンス / License

MIT(`LICENSE`参照)。 / MIT (see `LICENSE`).

## 配布物 / Releases

Windows (x86_64) のビルド済みバイナリは Releases から。Vulkan対応のGPUとドライバが必要です。他のOSは `cargo run --release`(`open-directx` はGitHubから自動取得されます)。 / Prebuilt Windows (x86_64) binary on the Releases page; a Vulkan-capable GPU and driver are required. Other OSes: `cargo run --release` (`open-directx` is fetched from GitHub automatically).
