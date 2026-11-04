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

## API 方針: REST 全廃・gRPC / Connect へ (2026-11-04)

### 日本語
- **REST は全廃する。新しい API は REST で作らない。** ブラウザや `curl` から呼ぶ API は **Connect の単純呼び出し**(`POST /<パッケージ>.<Service>/<Method>`、`content-type: application/json`、本文は JSON)、サービス間は **gRPC / gRPC-web**(RPoem の管理API `open_runo.admin.v1.Admin` と同じ流儀)。ストリーミングは、必要になってから。
- **既存の REST は、触るときに順に移す。** 移したら旧 REST の経路は消す。移行が終わるまで残るが、**新規には追加しない**。
- **JSON は RS-JSON に統一**する(読み込みは `from_str_secure` / `parse_secure` でサイズ・深さ・重複キーの上限つき、書き出しは `rust_json::fast`)。ブラウザの WASM に `prost` は入れない(サイズが増えるため)。
- **例外(自分たちの API ではなく、相手の規格が決めているもの)**: ACME、OAuth/OIDC のコールバック、第三者(GitHub など)からの Webhook、ヘルスチェック、静的ファイル配信、OTLP など。これらは規格どおりの HTTP を受ける。
- 参考実装: `RPoem`(`open_runo.admin.v1.Admin`)、`open-easy-web`(`server/src/connect.rs`、`proto/open_easy_web/admin/v1/admin.proto`)。
- ※ このリポジトリのコードが既に方針どおりかどうかは、各リポジトリの現状による。この節は方針の記録であり、コードの変更を意味しない。

### English
- **REST is being abolished. New APIs are not built as REST.** APIs called from browsers or `curl` use **Connect unary calls** (`POST /<package>.<Service>/<Method>`, `content-type: application/json`, JSON body); service-to-service calls use **gRPC / gRPC-web** (the same style as RPoem's admin API `open_runo.admin.v1.Admin`). Streaming only when it is actually needed.
- **Existing REST endpoints are migrated one by one, when they are touched.** The old REST route is removed once migrated. Until migration is finished they remain, but **no new ones are added**.
- **JSON is unified on RS-JSON**: reading with `from_str_secure` / `parse_secure` (size, depth and duplicate-key limits), writing with `rust_json::fast`. `prost` is not put into browser WASM (it would grow the binary).
- **Exceptions (shapes dictated by someone else's standard, not our own API)**: ACME, OAuth/OIDC callbacks, webhooks from third parties (e.g. GitHub), health checks, static file serving, OTLP, and so on. These accept plain HTTP as the standard requires.
- Reference implementations: `RPoem` (`open_runo.admin.v1.Admin`) and `open-easy-web` (`server/src/connect.rs`, `proto/open_easy_web/admin/v1/admin.proto`).
- Note: whether the code in a given repository already follows this policy depends on that repository. This section records the policy; it does not mean the code has been changed.

### Deutsch (KI-Übersetzung, noch nicht von Muttersprachlern geprüft)
- **REST wird abgeschafft. Neue APIs werden nicht als REST gebaut.** APIs, die aus dem Browser oder mit `curl` aufgerufen werden, nutzen **Connect-Unary-Aufrufe** (`POST /<Paket>.<Service>/<Methode>`, `content-type: application/json`, JSON-Body); zwischen Diensten gilt **gRPC / gRPC-web** (wie bei der Admin-API von RPoem, `open_runo.admin.v1.Admin`). Streaming erst, wenn es wirklich nötig ist.
- **Bestehende REST-Endpunkte werden einzeln migriert, sobald man sie anfasst.** Nach der Migration wird die alte REST-Route entfernt. Bis dahin bleiben sie bestehen, es werden aber **keine neuen hinzugefügt**.
- **JSON wird auf RS-JSON vereinheitlicht**: Lesen mit `from_str_secure` / `parse_secure` (Grenzen für Größe, Tiefe und doppelte Schlüssel), Schreiben mit `rust_json::fast`. `prost` kommt nicht in Browser-WASM (die Datei würde größer).
- **Ausnahmen (Formate, die ein fremder Standard vorgibt, nicht unsere eigene API)**: ACME, OAuth/OIDC-Callbacks, Webhooks von Dritten (z. B. GitHub), Health-Checks, statische Dateien, OTLP usw. Diese nehmen schlichtes HTTP entgegen, wie es der Standard verlangt.
- Referenzimplementierungen: `RPoem` (`open_runo.admin.v1.Admin`) und `open-easy-web` (`server/src/connect.rs`, `proto/open_easy_web/admin/v1/admin.proto`).
- Hinweis: Ob der Code eines Repositorys diese Richtlinie schon befolgt, hängt vom jeweiligen Repository ab. Dieser Abschnitt hält die Richtlinie fest; er bedeutet nicht, dass der Code bereits geändert wurde.

### Русский (машинный перевод, носителями языка ещё не проверен)
- **REST полностью отменяется. Новые API не создаются как REST.** API, вызываемые из браузера или через `curl`, используют **унарные вызовы Connect** (`POST /<пакет>.<Service>/<Method>`, `content-type: application/json`, тело — JSON); между сервисами — **gRPC / gRPC-web** (так же, как административный API RPoem, `open_runo.admin.v1.Admin`). Потоковая передача — только когда она действительно нужна.
- **Существующие REST-эндпоинты переносятся по одному, когда до них доходят руки.** После переноса старый REST-маршрут удаляется. До завершения переноса они остаются, но **новые не добавляются**.
- **JSON унифицируется на RS-JSON**: чтение через `from_str_secure` / `parse_secure` (ограничения на размер, глубину и повторяющиеся ключи), запись через `rust_json::fast`. `prost` не добавляется в браузерный WASM (размер вырос бы).
- **Исключения (форматы, заданные чужим стандартом, а не наш собственный API)**: ACME, обратные вызовы OAuth/OIDC, вебхуки сторонних сервисов (например, GitHub), проверки работоспособности, раздача статических файлов, OTLP и т. п. Они принимают обычный HTTP, как требует стандарт.
- Эталонные реализации: `RPoem` (`open_runo.admin.v1.Admin`) и `open-easy-web` (`server/src/connect.rs`, `proto/open_easy_web/admin/v1/admin.proto`).
- Примечание: соответствует ли код конкретного репозитория этой политике, зависит от самого репозитория. Этот раздел фиксирует политику и не означает, что код уже изменён.
