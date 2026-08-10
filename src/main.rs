//! sample-maid: 魔法少女メイドカフェの3Dオンラインゲーム(Phase 0: 2D
//! スプライトのプレースホルダー、`sample-harrier`と同じ土台を再利用)。
//!
//! **正直な開示(現在のスコープ)**: 実際の「3Dオンラインゲーム・世界の
//! 誰とでも友達になって大悪魔の魔女を倒す」という構想は、このPhase 0
//! では未実装。今回到達したのは`open-directx`のDXBC->SPIR-V翻訳済み
//! スプライトパイプライン(`sample-harrier`で先に組み立てたのと同じ
//! Renderer)を使い、(1)メイドちゃん(自機)、(2)助っ人「風天のトラさん」
//! (寅次郎風キャラのオマージュ、常に初音ミク風フィギュアを持っている
//! 設定——実際の意匠再現ではなく独自デザインとして扱う)、(3)魔女(敵)の
//! 3体を1画面に描画し、簡単な体力バー(縮む帯)と、ピンチ時にボタンで
//! 発動する「1ステージ1回まで」の救援アシストのロジックを実装した
//! ところまで。オンライン通信・3Dモデル・複数プレイヤー間のネットワーク
//! 同期・実際の攻撃/防御アクションは未実装(次回以降のロードマップ)。
//!
//! 助っ人の仕様(ユーザー指示、2026-08-10):
//! - 2人プレイ時: 2人目のプレイヤーが直接操作できる(本格的なローカル/
//!   オンライン協力プレイは未実装、このPhase 0ではキー入力のみ)。
//! - 1人プレイ時: 自機の体力がピンチ(30%以下)のときにアシストボタンを
//!   押すと、1ステージに1回だけ助っ人が発動し、魔女に「アシスト攻撃」が
//!   入って魔女の体力を減らす(このPhase 0では、ステージ=60秒ごとの
//!   タイマーループとして簡易的に扱い、ステージが切り替わるとアシスト
//!   回数がリセットされる)。
//!
//! 操作方法(ユーザー指示、2026-08-10):
//! - 1人プレイ: マウス移動で操作キャラクターを左右操作(マウスX座標を
//!   ウィンドウ幅に正規化してNDC X座標へマッピング)。デフォルトはメイド
//!   ちゃんだが、Vキーでメイドちゃん/風天のトラさんのどちらを操作するか
//!   切り替えられる(ユーザー指示、2026-08-10「一人プレイはメイドか
//!   トラさんの好きな方を選択可能」)。
//! - 2人プレイ(Tabキーで切替): 追加マウスの実配線はこのPhase 0では
//!   未実装(1つのOSマウスイベントしか受け取れないため、必要なら
//!   raw input APIでの複数マウス識別が次回課題)——代わりにテンキー
//!   矢印キー(Numpad4/Numpad6が左右)で助っ人を移動できる。
//! - 3つのアクションボタン(ユーザー指示、2026-08-10、P1/P2どちらの
//!   キーでも同じ効果=協力プレイ、共有のアクション扱い):
//!   - Z/B または F1/F10 → 味方化(近くの雑魚敵を問答無用で即時味方化)。
//!   - X/N または F2/F11 → 石化(魔女と雑魚敵の動きを一定時間停止)。
//!   - C/M または F3/F12 → サンダーボルト(魔女に直接ダメージ)。
//!   - 3ボタン同時押し → ブラックホール(全雑魚敵を即時味方化+魔女に
//!     大ダメージ)。
//!   - いずれか2ボタン同時押し → 助っ人「風天のトラさん」登場
//!     (1ステージに1回まで、既存のアシスト回数制限を共有する)。
//!
//! 味方化ギミック(ユーザー指示、2026-08-10「沢山の敵がどんどん味方に
//! なって一緒に敵と戦うゲームにして」への対応、Phase 0): 画面上に3体の
//! 雑魚敵(紫)が左右に徘徊しており、メイドちゃんが一定時間近づくと
//! 「味方化ゲージ」が満ちて味方(水色)に変わる。味方になった敵は
//! 一定間隔で自動的に魔女へ攻撃し、魔女の体力を削る——「世界中の敵を
//! お友だちにして大魔女を倒す」という原案の中核メカニクスの最小実証。
//! オンライン・複数プレイヤーでの共有・本格的な会話/イベントによる
//! 味方化(現在は単純な近接時間のみ)は未実装。
//!
//! BGM(ユーザー指示、2026-08-10): トラさん=洋風で間を意識したゆったり
//! した旋律・メイドちゃん=電子ポップ調の速い旋律という、それぞれ完全新規
//! のオリジナル曲を`audio.rs`で作曲し、Vキーでの操作キャラクター切替に
//! 応じて再生中のBGMも切り替わる。
//!
//! 実行方法: `cargo run --release`

mod audio;

use std::ffi::CString;

use ash::{vk, Entry};
use directx_graphics_vulkan::{Rgba8, TextureRgba8};
use directx_shader_translate::spirv_gen::{translate_sprite_pixel_shader, translate_sprite_vertex_shader};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use winit::event::{ElementState, Event, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

const WIDTH: u32 = 640;
const HEIGHT: u32 = 480;
/// Phase 0の簡易ステージ長(秒相当、実際はフレーム数で近似)。
const STAGE_FRAMES: u32 = 60 * 60;

#[repr(C)]
#[derive(Clone, Copy)]
struct Vertex {
    pos: [f32; 3],
    uv: [f32; 2],
}

fn quad_vertices(dest_ndc: [f32; 4]) -> [Vertex; 6] {
    let [x0, y0, x1, y1] = dest_ndc;
    let tl = Vertex { pos: [x0, y0, 0.0], uv: [0.0, 0.0] };
    let tr = Vertex { pos: [x1, y0, 0.0], uv: [1.0, 0.0] };
    let br = Vertex { pos: [x1, y1, 0.0], uv: [1.0, 1.0] };
    let bl = Vertex { pos: [x0, y1, 0.0], uv: [0.0, 1.0] };
    [tl, tr, br, tl, br, bl]
}

const ENEMY_COUNT: usize = 3;
/// 味方化に必要な近接フレーム数(60fps換算でおよそ1.5秒)。
const BEFRIEND_FRAMES: f32 = 90.0;
const BEFRIEND_RANGE: f32 = 0.12;
/// 味方化した敵が魔女を攻撃する間隔(フレーム)。
const ALLY_ATTACK_INTERVAL: u32 = 90;

struct Enemy {
    base_x: f32,
    phase: f32,
    is_ally: bool,
    befriend_progress: f32,
    attack_cooldown: u32,
}

impl Enemy {
    fn new(base_x: f32, phase: f32) -> Self {
        Enemy { base_x, phase, is_ally: false, befriend_progress: 0.0, attack_cooldown: ALLY_ATTACK_INTERVAL }
    }

    fn x(&self) -> f32 {
        self.base_x + 0.15 * self.phase.sin()
    }
}

/// 石化の持続フレーム数(60fps換算でおよそ2秒)。
const PETRIFY_FRAMES: u32 = 120;

/// 1人プレイ時にマウスで操作するキャラクター
/// (ユーザー指示、2026-08-10「一人プレイはメイドかトラさんの好きな方を
/// 選択可能」)。
#[derive(Clone, Copy, PartialEq, Eq)]
enum PlayableCharacter {
    Maid,
    Helper,
}

struct GameState {
    maid_x: f32,
    helper_x: f32,
    helper_left_held: bool,
    helper_right_held: bool,
    two_player: bool,
    /// 1人プレイ時にどちらのキャラクターをマウスで操作するか
    /// (Vキーで切替)。2人プレイ時は無視される(常にメイド=P1、
    /// 助っ人=P2として両方描画・操作される)。
    solo_character: PlayableCharacter,
    maid_hp: f32,
    witch_hp: f32,
    assist_used_this_stage: bool,
    assist_flash_timer: u32,
    stage_frame_counter: u32,
    enemies: [Enemy; ENEMY_COUNT],
    friend_count: u32,
    /// P1(Z/X/C, F1/F2/F3)・P2(B/N/M, F10/F11/F12)のどちらかが押されて
    /// いれば true になる、協力プレイ用の共有アクション押下状態。
    action_befriend_held: bool,
    action_petrify_held: bool,
    action_thunder_held: bool,
    /// 現在のボタン組み合わせで既に効果を発動済みかどうか
    /// (押しっぱなしで毎フレーム連続発動しないためのガード、全ボタンが
    /// 離されたらリセットされる)。
    combo_fired: bool,
    petrify_timer: u32,
}

impl GameState {
    fn new() -> Self {
        GameState {
            maid_x: -0.5,
            helper_x: 0.5,
            helper_left_held: false,
            helper_right_held: false,
            two_player: false,
            solo_character: PlayableCharacter::Maid,
            maid_hp: 1.0,
            witch_hp: 1.0,
            assist_used_this_stage: false,
            assist_flash_timer: 0,
            stage_frame_counter: 0,
            enemies: [Enemy::new(-0.6, 0.0), Enemy::new(0.0, 2.0), Enemy::new(0.6, 4.0)],
            friend_count: 0,
            action_befriend_held: false,
            action_petrify_held: false,
            action_thunder_held: false,
            combo_fired: false,
            petrify_timer: 0,
        }
    }

    fn instant_befriend_one(&mut self) {
        if let Some(enemy) = self.enemies.iter_mut().find(|e| !e.is_ally) {
            enemy.is_ally = true;
            enemy.befriend_progress = BEFRIEND_FRAMES;
            self.friend_count += 1;
            println!("味方化(Z/B/F1/F10): 敵が味方になった! (現在の味方数: {})", self.friend_count);
        }
    }

    fn cast_thunderbolt(&mut self) {
        self.witch_hp = (self.witch_hp - 0.15).max(0.0);
        println!("サンダーボルト(C/M/F3/F12): 魔女に直接ダメージ!");
    }

    fn cast_petrify(&mut self) {
        self.petrify_timer = PETRIFY_FRAMES;
        println!("石化(X/N/F2/F11): 魔女と雑魚敵の動きを一定時間停止!");
    }

    fn cast_black_hole(&mut self) {
        for enemy in self.enemies.iter_mut() {
            if !enemy.is_ally {
                enemy.is_ally = true;
                enemy.befriend_progress = BEFRIEND_FRAMES;
                self.friend_count += 1;
            }
        }
        self.witch_hp = (self.witch_hp - 0.5).max(0.0);
        println!("ブラックホール発動! 全雑魚敵を即時味方化+魔女に大ダメージ!");
    }

    fn summon_helper_assist(&mut self) {
        if !self.assist_used_this_stage {
            self.assist_used_this_stage = true;
            self.assist_flash_timer = 30;
            self.witch_hp = (self.witch_hp - 0.25).max(0.0);
            println!("2ボタン同時押し: 助っ人「風天のトラさん」参上! 魔女に一撃(このステージでは以後使用不可)");
        }
    }

    /// マウスX(NDC、-1.0〜1.0)を、1人プレイで選択中のキャラクターの
    /// 位置へ直接反映する(ユーザー指示: 1人プレイはマウス操作、
    /// メイドかトラさんのどちらを操作するかは`solo_character`で選択)。
    fn set_solo_x_from_mouse(&mut self, ndc_x: f32) {
        let x = ndc_x.clamp(-1.0, 1.0);
        match self.solo_character {
            PlayableCharacter::Maid => self.maid_x = x,
            PlayableCharacter::Helper => self.helper_x = x,
        }
    }

    fn toggle_solo_character(&mut self) {
        self.solo_character = match self.solo_character {
            PlayableCharacter::Maid => PlayableCharacter::Helper,
            PlayableCharacter::Helper => PlayableCharacter::Maid,
        };
        println!(
            "1人プレイの操作キャラクターを切替: {}",
            match self.solo_character {
                PlayableCharacter::Maid => "メイドちゃん",
                PlayableCharacter::Helper => "風天のトラさん",
            }
        );
    }

    fn update(&mut self) {
        let speed = 0.02;
        if self.two_player {
            if self.helper_left_held {
                self.helper_x -= speed;
            }
            if self.helper_right_held {
                self.helper_x += speed;
            }
            self.helper_x = self.helper_x.clamp(-1.0, 1.0);
        }

        // ステージタイマー: 一定フレームごとにリセットし、アシスト使用回数も戻す。
        self.stage_frame_counter += 1;
        if self.stage_frame_counter >= STAGE_FRAMES {
            self.stage_frame_counter = 0;
            self.assist_used_this_stage = false;
            println!("新しいステージ: 助っ人アシストが再び使用可能になりました");
        }

        // 石化中は魔女(≒ゆっくり自然減少するデモ用HP)・雑魚敵の動きを止める。
        if self.petrify_timer > 0 {
            self.petrify_timer -= 1;
        } else {
            self.maid_hp = (self.maid_hp - 0.0006).max(0.0);
        }

        // アクションボタンの組み合わせ判定(P1/P2共有、協力プレイ)。
        // 押しっぱなしで毎フレーム発動しないよう、全ボタンが離れるまで
        // 一度だけ発動する(`combo_fired`ガード)。
        let held_count = [self.action_befriend_held, self.action_petrify_held, self.action_thunder_held]
            .iter()
            .filter(|h| **h)
            .count();
        if held_count == 0 {
            self.combo_fired = false;
        } else if !self.combo_fired {
            self.combo_fired = true;
            match held_count {
                3 => self.cast_black_hole(),
                2 => self.summon_helper_assist(),
                1 => {
                    if self.action_befriend_held {
                        self.instant_befriend_one();
                    } else if self.action_petrify_held {
                        self.cast_petrify();
                    } else if self.action_thunder_held {
                        self.cast_thunderbolt();
                    }
                }
                _ => {}
            }
        }

        if self.assist_flash_timer > 0 {
            self.assist_flash_timer -= 1;
        }

        // 味方化ギミック: 雑魚敵に近づき続けると味方になり、以後は自動で
        // 魔女を攻撃する(石化中は敵の動き・攻撃クールダウンも止める)。
        for enemy in self.enemies.iter_mut() {
            if self.petrify_timer > 0 {
                continue;
            }
            enemy.phase += 0.015;
            if !enemy.is_ally {
                let dx = (self.maid_x - enemy.x()).abs();
                if dx < BEFRIEND_RANGE {
                    enemy.befriend_progress += 1.0;
                    if enemy.befriend_progress >= BEFRIEND_FRAMES {
                        enemy.is_ally = true;
                        self.friend_count += 1;
                        println!("敵が味方になった! (現在の味方数: {})", self.friend_count);
                    }
                } else {
                    enemy.befriend_progress = (enemy.befriend_progress - 2.0).max(0.0);
                }
            } else {
                if enemy.attack_cooldown > 0 {
                    enemy.attack_cooldown -= 1;
                } else {
                    enemy.attack_cooldown = ALLY_ATTACK_INTERVAL;
                    self.witch_hp = (self.witch_hp - 0.02).max(0.0);
                }
            }
        }
    }

    fn maid_dest_ndc(&self) -> [f32; 4] {
        [self.maid_x - 0.08, 0.5, self.maid_x + 0.08, 0.75]
    }
    fn helper_dest_ndc(&self) -> [f32; 4] {
        [self.helper_x - 0.08, 0.5, self.helper_x + 0.08, 0.75]
    }
    fn witch_dest_ndc(&self) -> [f32; 4] {
        [-0.15, -0.7, 0.15, -0.35]
    }
    fn maid_hp_bar_ndc(&self) -> [f32; 4] {
        let w = 0.5 * self.maid_hp;
        [-0.9, -0.95, -0.9 + w, -0.88]
    }
    fn witch_hp_bar_ndc(&self) -> [f32; 4] {
        let w = 0.5 * self.witch_hp;
        [0.4 - w, -0.95, 0.4, -0.88]
    }
    fn enemy_dest_ndc(&self, i: usize) -> [f32; 4] {
        let x = self.enemies[i].x();
        [x - 0.06, 0.1, x + 0.06, 0.3]
    }
}

#[allow(unused_assignments, unused_variables)]
fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("sample-maid: magical-girl maid-cafe 3D online game (Phase 0: 2D sprite placeholder)")
        .with_inner_size(winit::dpi::LogicalSize::new(WIDTH, HEIGHT))
        .build(&event_loop)
        .expect("create window");

    let mut renderer = unsafe { Renderer::new(&window) }.expect("create Vulkan renderer");
    let mut state = GameState::new();

    // BGM再生ハンドルはイベントループが動いている間ずっと保持する
    // (dropすると再生が止まる)。Vキーでの操作キャラクター切替に応じて
    // 再代入し、旧テーマを止めて新テーマを再生する。
    let mut bgm = audio::play_theme(audio::Theme::Maid);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::WindowEvent { event: WindowEvent::CursorMoved { position, .. }, .. } => {
                // マウスX座標をウィンドウ幅で正規化してNDC(-1.0〜1.0)へ
                // マッピングする(1人プレイのメイドちゃん操作、ユーザー指示)。
                let ndc_x = (position.x as f32 / WIDTH as f32) * 2.0 - 1.0;
                if !state.two_player {
                    state.set_solo_x_from_mouse(ndc_x);
                }
            }
            Event::WindowEvent { event: WindowEvent::KeyboardInput { input, .. }, .. } => {
                let pressed = input.state == ElementState::Pressed;
                match input.virtual_keycode {
                    // 3つのアクションボタン、P1(Z/X/C, F1/F2/F3)とP2
                    // (N/M, F11/F12)は同じ効果を共有する(協力プレイ)。
                    // 味方化はZ/F1のみ(B/F10は助っ人登場専用キーへ変更済み)。
                    Some(VirtualKeyCode::Z) | Some(VirtualKeyCode::F1) => {
                        state.action_befriend_held = pressed;
                    }
                    Some(VirtualKeyCode::X) | Some(VirtualKeyCode::F2) | Some(VirtualKeyCode::N) | Some(VirtualKeyCode::F11) => {
                        state.action_petrify_held = pressed;
                    }
                    Some(VirtualKeyCode::C) | Some(VirtualKeyCode::F3) | Some(VirtualKeyCode::M) | Some(VirtualKeyCode::F12) => {
                        state.action_thunder_held = pressed;
                    }
                    // 助っ人「風天のトラさん」登場専用キー(1ステージに1回まで、
                    // 既存のアシスト回数制限を共有する)。
                    Some(VirtualKeyCode::H) | Some(VirtualKeyCode::F10) if pressed => state.summon_helper_assist(),
                    // 2人プレイ: テンキー矢印(Numpad4/Numpad6)で助っ人を移動。
                    Some(VirtualKeyCode::Numpad4) => state.helper_left_held = pressed,
                    Some(VirtualKeyCode::Numpad6) => state.helper_right_held = pressed,
                    Some(VirtualKeyCode::Tab) if pressed => state.two_player = !state.two_player,
                    Some(VirtualKeyCode::V) if pressed && !state.two_player => {
                        state.toggle_solo_character();
                        let theme = match state.solo_character {
                            PlayableCharacter::Maid => audio::Theme::Maid,
                            PlayableCharacter::Helper => audio::Theme::Helper,
                        };
                        bgm = audio::play_theme(theme);
                    }
                    Some(VirtualKeyCode::Escape) => *control_flow = ControlFlow::Exit,
                    _ => {}
                }
            }
            Event::MainEventsCleared => {
                state.update();
                unsafe { renderer.draw_frame(&state) };
            }
            _ => {}
        }
    });
}

/// 実ウィンドウ向けVulkanレンダラー。`sample-harrier`/`open-directx`の
/// `directx-graphics-window`と同一パターン(固定サイズスワップチェーン、
/// 単一コマンドバッファ+フェンス待機)。リサイズ非対応。
struct Renderer {
    _entry: Entry,
    instance: ash::Instance,
    surface_loader: ash::extensions::khr::Surface,
    surface: vk::SurfaceKHR,
    device: ash::Device,
    queue: vk::Queue,
    swapchain_loader: ash::extensions::khr::Swapchain,
    swapchain: vk::SwapchainKHR,
    _swapchain_format: vk::Format,
    swapchain_extent: vk::Extent2D,
    swapchain_image_views: Vec<vk::ImageView>,
    render_pass: vk::RenderPass,
    framebuffers: Vec<vk::Framebuffer>,
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
    descriptor_pool: vk::DescriptorPool,
    descriptor_set_layout: vk::DescriptorSetLayout,
    maid_descriptor_set: vk::DescriptorSet,
    helper_descriptor_set: vk::DescriptorSet,
    witch_descriptor_set: vk::DescriptorSet,
    maid_hp_descriptor_set: vk::DescriptorSet,
    witch_hp_descriptor_set: vk::DescriptorSet,
    assist_flash_descriptor_set: vk::DescriptorSet,
    enemy_notally_descriptor_set: vk::DescriptorSet,
    enemy_ally_descriptor_set: vk::DescriptorSet,
    vertex_buffer: vk::Buffer,
    vertex_buffer_memory: vk::DeviceMemory,
    command_pool: vk::CommandPool,
    command_buffer: vk::CommandBuffer,
    image_available: vk::Semaphore,
    render_finished: vk::Semaphore,
    in_flight: vk::Fence,
    _maid_texture_image: vk::Image,
    _helper_texture_image: vk::Image,
    _witch_texture_image: vk::Image,
    _maid_hp_texture_image: vk::Image,
    _witch_hp_texture_image: vk::Image,
    _assist_flash_texture_image: vk::Image,
    _enemy_notally_texture_image: vk::Image,
    _enemy_ally_texture_image: vk::Image,
    sampler: vk::Sampler,
}

/// 固定スプライト(メイド・助っ人・魔女・HPバー2本・アシストフラッシュ)
/// +雑魚敵`ENEMY_COUNT`体を毎フレーム描画する。
const FIXED_SPRITE_COUNT: usize = 6;
const DRAWN_SPRITE_COUNT: usize = FIXED_SPRITE_COUNT + ENEMY_COUNT;
/// 一意のテクスチャ数(雑魚敵は味方化状態に応じて2種のテクスチャの
/// いずれかを描画時に選ぶため、`DRAWN_SPRITE_COUNT`より少ない)。
const TEXTURE_COUNT: usize = FIXED_SPRITE_COUNT + 2;
const VERTS_PER_SPRITE: usize = 6;

impl Renderer {
    unsafe fn new(window: &winit::window::Window) -> Result<Self, String> {
        let entry = Entry::load().map_err(|e| e.to_string())?;

        let app_name = CString::new("sample-maid").unwrap();
        let app_info = vk::ApplicationInfo::builder().application_name(&app_name).api_version(vk::API_VERSION_1_1);
        let display_handle = window.raw_display_handle();
        let window_handle = window.raw_window_handle();
        let required_extensions =
            ash_window::enumerate_required_extensions(display_handle).map_err(|e| e.to_string())?;
        let instance_info = vk::InstanceCreateInfo::builder().application_info(&app_info).enabled_extension_names(required_extensions);
        let instance = entry.create_instance(&instance_info, None).map_err(|e| e.to_string())?;

        let surface = ash_window::create_surface(&entry, &instance, display_handle, window_handle, None).map_err(|e| e.to_string())?;
        let surface_loader = ash::extensions::khr::Surface::new(&entry, &instance);

        let physical_devices = instance.enumerate_physical_devices().map_err(|e| e.to_string())?;
        let mut selected: Option<(vk::PhysicalDevice, u32)> = None;
        for &pd in &physical_devices {
            let families = instance.get_physical_device_queue_family_properties(pd);
            for (idx, f) in families.iter().enumerate() {
                let supports_present = surface_loader.get_physical_device_surface_support(pd, idx as u32, surface).unwrap_or(false);
                if f.queue_flags.contains(vk::QueueFlags::GRAPHICS) && supports_present {
                    selected = Some((pd, idx as u32));
                    break;
                }
            }
            if selected.is_some() {
                break;
            }
        }
        let (physical_device, queue_family_index) = selected.ok_or("no graphics+present capable device found")?;

        let priorities = [1.0f32];
        let queue_info = [vk::DeviceQueueCreateInfo::builder().queue_family_index(queue_family_index).queue_priorities(&priorities).build()];
        let device_extensions = [ash::extensions::khr::Swapchain::name().as_ptr()];
        let device_info = vk::DeviceCreateInfo::builder().queue_create_infos(&queue_info).enabled_extension_names(&device_extensions);
        let device = instance.create_device(physical_device, &device_info, None).map_err(|e| e.to_string())?;
        let queue = device.get_device_queue(queue_family_index, 0);
        let memory_properties = instance.get_physical_device_memory_properties(physical_device);

        let swapchain_loader = ash::extensions::khr::Swapchain::new(&instance, &device);
        let surface_caps = surface_loader.get_physical_device_surface_capabilities(physical_device, surface).map_err(|e| e.to_string())?;
        let surface_formats = surface_loader.get_physical_device_surface_formats(physical_device, surface).map_err(|e| e.to_string())?;
        let surface_format = surface_formats
            .iter()
            .find(|f| f.format == vk::Format::B8G8R8A8_UNORM)
            .copied()
            .unwrap_or(surface_formats[0]);
        let present_mode = vk::PresentModeKHR::FIFO;
        let extent = vk::Extent2D { width: WIDTH, height: HEIGHT };
        let image_count = (surface_caps.min_image_count + 1).min(if surface_caps.max_image_count > 0 { surface_caps.max_image_count } else { u32::MAX });

        let swapchain_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(surface)
            .min_image_count(image_count)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(surface_caps.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true);
        let swapchain = swapchain_loader.create_swapchain(&swapchain_info, None).map_err(|e| e.to_string())?;
        let swapchain_images = swapchain_loader.get_swapchain_images(swapchain).map_err(|e| e.to_string())?;

        let swapchain_image_views: Vec<vk::ImageView> = swapchain_images
            .iter()
            .map(|&img| {
                let info = vk::ImageViewCreateInfo::builder()
                    .image(img)
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(surface_format.format)
                    .subresource_range(vk::ImageSubresourceRange {
                        aspect_mask: vk::ImageAspectFlags::COLOR,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    });
                device.create_image_view(&info, None).expect("create swapchain image view")
            })
            .collect();

        let attachment = vk::AttachmentDescription::builder()
            .format(surface_format.format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
            .build();
        let color_ref = vk::AttachmentReference { attachment: 0, layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL };
        let color_refs = [color_ref];
        let subpass = vk::SubpassDescription::builder().pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS).color_attachments(&color_refs).build();
        let attachments = [attachment];
        let subpasses = [subpass];
        let dependency = vk::SubpassDependency::builder()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
            .build();
        let dependencies = [dependency];
        let render_pass_info = vk::RenderPassCreateInfo::builder().attachments(&attachments).subpasses(&subpasses).dependencies(&dependencies);
        let render_pass = device.create_render_pass(&render_pass_info, None).map_err(|e| e.to_string())?;

        let framebuffers: Vec<vk::Framebuffer> = swapchain_image_views
            .iter()
            .map(|&view| {
                let attachments_fb = [view];
                let info = vk::FramebufferCreateInfo::builder()
                    .render_pass(render_pass)
                    .attachments(&attachments_fb)
                    .width(extent.width)
                    .height(extent.height)
                    .layers(1);
                device.create_framebuffer(&info, None).expect("create framebuffer")
            })
            .collect();

        // メイド=ピンク、助っ人(風天のトラさん)=茶+橙のオマージュ色、魔女=紫、
        // HPバー=緑、アシストフラッシュ=白。
        let (maid_image, maid_view) = create_solid_texture(&device, &memory_properties, queue, queue_family_index, Rgba8 { r: 240, g: 150, b: 190, a: 255 })?;
        let (helper_image, helper_view) = create_solid_texture(&device, &memory_properties, queue, queue_family_index, Rgba8 { r: 200, g: 120, b: 50, a: 255 })?;
        let (witch_image, witch_view) = create_solid_texture(&device, &memory_properties, queue, queue_family_index, Rgba8 { r: 120, g: 40, b: 160, a: 255 })?;
        let (maid_hp_image, maid_hp_view) = create_solid_texture(&device, &memory_properties, queue, queue_family_index, Rgba8 { r: 60, g: 220, b: 90, a: 255 })?;
        let (witch_hp_image, witch_hp_view) = create_solid_texture(&device, &memory_properties, queue, queue_family_index, Rgba8 { r: 220, g: 60, b: 60, a: 255 })?;
        let (assist_flash_image, assist_flash_view) = create_solid_texture(&device, &memory_properties, queue, queue_family_index, Rgba8 { r: 255, g: 255, b: 255, a: 255 })?;
        let (enemy_notally_image, enemy_notally_view) = create_solid_texture(&device, &memory_properties, queue, queue_family_index, Rgba8 { r: 150, g: 50, b: 170, a: 255 })?;
        let (enemy_ally_image, enemy_ally_view) = create_solid_texture(&device, &memory_properties, queue, queue_family_index, Rgba8 { r: 90, g: 200, b: 230, a: 255 })?;

        let sampler_info = vk::SamplerCreateInfo::builder().mag_filter(vk::Filter::NEAREST).min_filter(vk::Filter::NEAREST);
        let sampler = device.create_sampler(&sampler_info, None).map_err(|e| e.to_string())?;

        let dsl_binding = vk::DescriptorSetLayoutBinding::builder()
            .binding(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT)
            .build();
        let dsl_bindings = [dsl_binding];
        let dsl_info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(&dsl_bindings);
        let descriptor_set_layout = device.create_descriptor_set_layout(&dsl_info, None).map_err(|e| e.to_string())?;

        let pool_size = vk::DescriptorPoolSize { ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER, descriptor_count: TEXTURE_COUNT as u32 };
        let pool_sizes = [pool_size];
        let descriptor_pool_info = vk::DescriptorPoolCreateInfo::builder().pool_sizes(&pool_sizes).max_sets(TEXTURE_COUNT as u32);
        let descriptor_pool = device.create_descriptor_pool(&descriptor_pool_info, None).map_err(|e| e.to_string())?;

        let set_layouts = vec![descriptor_set_layout; TEXTURE_COUNT];
        let ds_alloc_info = vk::DescriptorSetAllocateInfo::builder().descriptor_pool(descriptor_pool).set_layouts(&set_layouts);
        let sets = device.allocate_descriptor_sets(&ds_alloc_info).map_err(|e| e.to_string())?;
        let (
            maid_descriptor_set,
            helper_descriptor_set,
            witch_descriptor_set,
            maid_hp_descriptor_set,
            witch_hp_descriptor_set,
            assist_flash_descriptor_set,
            enemy_notally_descriptor_set,
            enemy_ally_descriptor_set,
        ) = (sets[0], sets[1], sets[2], sets[3], sets[4], sets[5], sets[6], sets[7]);

        for (set, view) in [
            (maid_descriptor_set, maid_view),
            (helper_descriptor_set, helper_view),
            (witch_descriptor_set, witch_view),
            (maid_hp_descriptor_set, maid_hp_view),
            (witch_hp_descriptor_set, witch_hp_view),
            (assist_flash_descriptor_set, assist_flash_view),
            (enemy_notally_descriptor_set, enemy_notally_view),
            (enemy_ally_descriptor_set, enemy_ally_view),
        ] {
            let image_info = vk::DescriptorImageInfo::builder()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(view)
                .sampler(sampler)
                .build();
            let image_infos = [image_info];
            let write = vk::WriteDescriptorSet::builder()
                .dst_set(set)
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(&image_infos)
                .build();
            device.update_descriptor_sets(&[write], &[]);
        }

        let vbuf_size = ((DRAWN_SPRITE_COUNT * VERTS_PER_SPRITE) * std::mem::size_of::<Vertex>()) as vk::DeviceSize;
        let (vertex_buffer, vertex_buffer_memory) =
            create_host_visible_buffer(&device, &memory_properties, vbuf_size, vk::BufferUsageFlags::VERTEX_BUFFER)?;

        let vs = translate_sprite_vertex_shader(include_bytes!("../../open-directx/crates/directx-shader-translate/shaders/sprite_vs.dxbc")).map_err(|e| e.to_string())?;
        let ps = translate_sprite_pixel_shader(include_bytes!("../../open-directx/crates/directx-shader-translate/shaders/sprite_ps.dxbc")).map_err(|e| e.to_string())?;

        let vs_module_info = vk::ShaderModuleCreateInfo::builder().code(&vs.spirv_words);
        let vs_module = device.create_shader_module(&vs_module_info, None).map_err(|e| e.to_string())?;
        let ps_module_info = vk::ShaderModuleCreateInfo::builder().code(&ps.spirv_words);
        let ps_module = device.create_shader_module(&ps_module_info, None).map_err(|e| e.to_string())?;

        let entry_name = CString::new("main").unwrap();
        let stages = [
            vk::PipelineShaderStageCreateInfo::builder().stage(vk::ShaderStageFlags::VERTEX).module(vs_module).name(&entry_name).build(),
            vk::PipelineShaderStageCreateInfo::builder().stage(vk::ShaderStageFlags::FRAGMENT).module(ps_module).name(&entry_name).build(),
        ];

        let binding_desc = vk::VertexInputBindingDescription { binding: 0, stride: std::mem::size_of::<Vertex>() as u32, input_rate: vk::VertexInputRate::VERTEX };
        let attr_descs = [
            vk::VertexInputAttributeDescription { location: 0, binding: 0, format: vk::Format::R32G32B32_SFLOAT, offset: 0 },
            vk::VertexInputAttributeDescription { location: 1, binding: 0, format: vk::Format::R32G32_SFLOAT, offset: std::mem::size_of::<[f32; 3]>() as u32 },
        ];
        let binding_descs = [binding_desc];
        let vertex_input = vk::PipelineVertexInputStateCreateInfo::builder().vertex_binding_descriptions(&binding_descs).vertex_attribute_descriptions(&attr_descs);
        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::builder().topology(vk::PrimitiveTopology::TRIANGLE_LIST);
        let viewport = vk::Viewport { x: 0.0, y: 0.0, width: extent.width as f32, height: extent.height as f32, min_depth: 0.0, max_depth: 1.0 };
        let scissor = vk::Rect2D { offset: vk::Offset2D { x: 0, y: 0 }, extent };
        let viewports = [viewport];
        let scissors = [scissor];
        let viewport_state = vk::PipelineViewportStateCreateInfo::builder().viewports(&viewports).scissors(&scissors);
        let rasterization = vk::PipelineRasterizationStateCreateInfo::builder()
            .polygon_mode(vk::PolygonMode::FILL)
            .cull_mode(vk::CullModeFlags::NONE)
            .front_face(vk::FrontFace::CLOCKWISE)
            .line_width(1.0);
        let multisample = vk::PipelineMultisampleStateCreateInfo::builder().rasterization_samples(vk::SampleCountFlags::TYPE_1);
        let color_blend_attachment = vk::PipelineColorBlendAttachmentState { blend_enable: vk::FALSE, color_write_mask: vk::ColorComponentFlags::RGBA, ..Default::default() };
        let color_blend_attachments = [color_blend_attachment];
        let color_blend = vk::PipelineColorBlendStateCreateInfo::builder().attachments(&color_blend_attachments);
        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::builder().set_layouts(&set_layouts[..1]);
        let pipeline_layout = device.create_pipeline_layout(&pipeline_layout_info, None).map_err(|e| e.to_string())?;
        let pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(&stages)
            .vertex_input_state(&vertex_input)
            .input_assembly_state(&input_assembly)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterization)
            .multisample_state(&multisample)
            .color_blend_state(&color_blend)
            .layout(pipeline_layout)
            .render_pass(render_pass)
            .subpass(0)
            .build();
        let pipeline = device
            .create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_info], None)
            .map_err(|(_, e)| e.to_string())?[0];
        device.destroy_shader_module(vs_module, None);
        device.destroy_shader_module(ps_module, None);

        let pool_info = vk::CommandPoolCreateInfo::builder().queue_family_index(queue_family_index).flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);
        let command_pool = device.create_command_pool(&pool_info, None).map_err(|e| e.to_string())?;
        let cmd_alloc_info = vk::CommandBufferAllocateInfo::builder().command_pool(command_pool).level(vk::CommandBufferLevel::PRIMARY).command_buffer_count(1);
        let command_buffer = device.allocate_command_buffers(&cmd_alloc_info).map_err(|e| e.to_string())?[0];

        let sem_info = vk::SemaphoreCreateInfo::builder();
        let image_available = device.create_semaphore(&sem_info, None).map_err(|e| e.to_string())?;
        let render_finished = device.create_semaphore(&sem_info, None).map_err(|e| e.to_string())?;
        let fence_info = vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);
        let in_flight = device.create_fence(&fence_info, None).map_err(|e| e.to_string())?;

        println!(
            "sample-maid: 実ウィンドウ+実スワップチェーンを作成しました({}x{}, format_raw={}, images={})",
            extent.width,
            extent.height,
            surface_format.format.as_raw(),
            swapchain_images.len()
        );

        Ok(Renderer {
            _entry: entry,
            instance,
            surface_loader,
            surface,
            device,
            queue,
            swapchain_loader,
            swapchain,
            _swapchain_format: surface_format.format,
            swapchain_extent: extent,
            swapchain_image_views,
            render_pass,
            framebuffers,
            pipeline_layout,
            pipeline,
            descriptor_pool,
            descriptor_set_layout,
            maid_descriptor_set,
            helper_descriptor_set,
            witch_descriptor_set,
            maid_hp_descriptor_set,
            witch_hp_descriptor_set,
            assist_flash_descriptor_set,
            enemy_notally_descriptor_set,
            enemy_ally_descriptor_set,
            vertex_buffer,
            vertex_buffer_memory,
            command_pool,
            command_buffer,
            image_available,
            render_finished,
            in_flight,
            _maid_texture_image: maid_image,
            _helper_texture_image: helper_image,
            _witch_texture_image: witch_image,
            _maid_hp_texture_image: maid_hp_image,
            _witch_hp_texture_image: witch_hp_image,
            _assist_flash_texture_image: assist_flash_image,
            _enemy_notally_texture_image: enemy_notally_image,
            _enemy_ally_texture_image: enemy_ally_image,
            sampler,
        })
    }

    unsafe fn draw_frame(&mut self, state: &GameState) {
        self.device.wait_for_fences(&[self.in_flight], true, u64::MAX).expect("wait_for_fences");

        let (image_index, _) = match self.swapchain_loader.acquire_next_image(self.swapchain, u64::MAX, self.image_available, vk::Fence::null()) {
            Ok(r) => r,
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => return,
            Err(e) => panic!("vkAcquireNextImageKHR failed: {e}"),
        };
        self.device.reset_fences(&[self.in_flight]).expect("reset_fences");

        // アシストフラッシュは発動中のみ画面中央に大きく表示、それ以外は
        // 画面外(NDC範囲外)へ寄せて実質非表示にする(専用の可視/不可視フラグは
        // このパイプラインには無いため、座標で代用する簡易実装)。
        let assist_flash_ndc = if state.assist_flash_timer > 0 { [-0.3, -0.3, 0.3, 0.3] } else { [-2.0, -2.0, -1.9, -1.9] };

        let mut sprites_ndc = [[0.0f32; 4]; DRAWN_SPRITE_COUNT];
        sprites_ndc[0] = state.maid_dest_ndc();
        // 助っ人は2人プレイ、または1人プレイでトラさんを選択中なら表示する。
        let helper_visible = state.two_player || state.solo_character == PlayableCharacter::Helper;
        sprites_ndc[1] = if helper_visible { state.helper_dest_ndc() } else { [-2.0, -2.0, -1.9, -1.9] };
        sprites_ndc[2] = state.witch_dest_ndc();
        sprites_ndc[3] = state.maid_hp_bar_ndc();
        sprites_ndc[4] = state.witch_hp_bar_ndc();
        sprites_ndc[5] = assist_flash_ndc;
        for i in 0..ENEMY_COUNT {
            sprites_ndc[FIXED_SPRITE_COUNT + i] = state.enemy_dest_ndc(i);
        }
        let mut all_verts = [Vertex { pos: [0.0; 3], uv: [0.0; 2] }; DRAWN_SPRITE_COUNT * VERTS_PER_SPRITE];
        for (i, ndc) in sprites_ndc.iter().enumerate() {
            let verts = quad_vertices(*ndc);
            all_verts[i * VERTS_PER_SPRITE..(i + 1) * VERTS_PER_SPRITE].copy_from_slice(&verts);
        }
        let vbuf_size = std::mem::size_of_val(&all_verts) as vk::DeviceSize;
        let ptr = self.device.map_memory(self.vertex_buffer_memory, 0, vbuf_size, vk::MemoryMapFlags::empty()).expect("map vertex buffer");
        std::ptr::copy_nonoverlapping(all_verts.as_ptr() as *const u8, ptr as *mut u8, vbuf_size as usize);
        self.device.unmap_memory(self.vertex_buffer_memory);

        self.device.reset_command_buffer(self.command_buffer, vk::CommandBufferResetFlags::empty()).expect("reset command buffer");
        let begin_info = vk::CommandBufferBeginInfo::builder();
        self.device.begin_command_buffer(self.command_buffer, &begin_info).expect("begin command buffer");

        let clear_value = vk::ClearValue { color: vk::ClearColorValue { float32: [0.15, 0.05, 0.2, 1.0] } };
        let clear_values = [clear_value];
        let render_pass_begin = vk::RenderPassBeginInfo::builder()
            .render_pass(self.render_pass)
            .framebuffer(self.framebuffers[image_index as usize])
            .render_area(vk::Rect2D { offset: vk::Offset2D { x: 0, y: 0 }, extent: self.swapchain_extent })
            .clear_values(&clear_values);
        self.device.cmd_begin_render_pass(self.command_buffer, &render_pass_begin, vk::SubpassContents::INLINE);
        self.device.cmd_bind_pipeline(self.command_buffer, vk::PipelineBindPoint::GRAPHICS, self.pipeline);
        self.device.cmd_bind_vertex_buffers(self.command_buffer, 0, &[self.vertex_buffer], &[0]);

        let mut sets = [
            self.maid_descriptor_set,
            self.helper_descriptor_set,
            self.witch_descriptor_set,
            self.maid_hp_descriptor_set,
            self.witch_hp_descriptor_set,
            self.assist_flash_descriptor_set,
            self.enemy_notally_descriptor_set,
            self.enemy_notally_descriptor_set,
            self.enemy_notally_descriptor_set,
        ];
        for i in 0..ENEMY_COUNT {
            if state.enemies[i].is_ally {
                sets[FIXED_SPRITE_COUNT + i] = self.enemy_ally_descriptor_set;
            }
        }
        for (i, &set) in sets.iter().enumerate() {
            self.device.cmd_bind_descriptor_sets(self.command_buffer, vk::PipelineBindPoint::GRAPHICS, self.pipeline_layout, 0, &[set], &[]);
            self.device.cmd_draw(self.command_buffer, VERTS_PER_SPRITE as u32, 1, (i * VERTS_PER_SPRITE) as u32, 0);
        }

        self.device.cmd_end_render_pass(self.command_buffer);
        self.device.end_command_buffer(self.command_buffer).expect("end command buffer");

        let wait_semaphores = [self.image_available];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let signal_semaphores = [self.render_finished];
        let command_buffers = [self.command_buffer];
        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&wait_stages)
            .command_buffers(&command_buffers)
            .signal_semaphores(&signal_semaphores)
            .build();
        self.device.queue_submit(self.queue, &[submit_info], self.in_flight).expect("queue_submit");

        let swapchains = [self.swapchain];
        let image_indices = [image_index];
        let present_info = vk::PresentInfoKHR::builder().wait_semaphores(&signal_semaphores).swapchains(&swapchains).image_indices(&image_indices);
        match self.swapchain_loader.queue_present(self.queue, &present_info) {
            Ok(_) | Err(vk::Result::ERROR_OUT_OF_DATE_KHR) | Err(vk::Result::SUBOPTIMAL_KHR) => {}
            Err(e) => panic!("vkQueuePresentKHR failed: {e}"),
        }
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            let _ = self.device.device_wait_idle();
            self.device.destroy_semaphore(self.image_available, None);
            self.device.destroy_semaphore(self.render_finished, None);
            self.device.destroy_fence(self.in_flight, None);
            self.device.destroy_command_pool(self.command_pool, None);
            self.device.destroy_pipeline(self.pipeline, None);
            self.device.destroy_pipeline_layout(self.pipeline_layout, None);
            self.device.destroy_descriptor_pool(self.descriptor_pool, None);
            self.device.destroy_descriptor_set_layout(self.descriptor_set_layout, None);
            self.device.destroy_sampler(self.sampler, None);
            self.device.destroy_buffer(self.vertex_buffer, None);
            self.device.free_memory(self.vertex_buffer_memory, None);
            for &fb in &self.framebuffers {
                self.device.destroy_framebuffer(fb, None);
            }
            self.device.destroy_render_pass(self.render_pass, None);
            for &view in &self.swapchain_image_views {
                self.device.destroy_image_view(view, None);
            }
            self.swapchain_loader.destroy_swapchain(self.swapchain, None);
            self.device.destroy_device(None);
            self.surface_loader.destroy_surface(self.surface, None);
            self.instance.destroy_instance(None);
        }
    }
}

unsafe fn create_host_visible_buffer(
    device: &ash::Device,
    memory_properties: &vk::PhysicalDeviceMemoryProperties,
    size: vk::DeviceSize,
    usage: vk::BufferUsageFlags,
) -> Result<(vk::Buffer, vk::DeviceMemory), String> {
    let buffer_info = vk::BufferCreateInfo::builder().size(size).usage(usage).sharing_mode(vk::SharingMode::EXCLUSIVE);
    let buffer = device.create_buffer(&buffer_info, None).map_err(|e| e.to_string())?;
    let mem_req = device.get_buffer_memory_requirements(buffer);
    let mem_type = find_memory_type(memory_properties, mem_req.memory_type_bits, vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT)
        .ok_or("no suitable host-visible memory type")?;
    let alloc_info = vk::MemoryAllocateInfo::builder().allocation_size(mem_req.size).memory_type_index(mem_type);
    let memory = device.allocate_memory(&alloc_info, None).map_err(|e| e.to_string())?;
    device.bind_buffer_memory(buffer, memory, 0).map_err(|e| e.to_string())?;
    Ok((buffer, memory))
}

fn find_memory_type(props: &vk::PhysicalDeviceMemoryProperties, type_bits: u32, required: vk::MemoryPropertyFlags) -> Option<u32> {
    (0..props.memory_type_count).find(|&i| type_bits & (1 << i) != 0 && props.memory_types[i as usize].property_flags.contains(required))
}

unsafe fn create_solid_texture(
    device: &ash::Device,
    memory_properties: &vk::PhysicalDeviceMemoryProperties,
    queue: vk::Queue,
    queue_family_index: u32,
    color: Rgba8,
) -> Result<(vk::Image, vk::ImageView), String> {
    let texture = TextureRgba8 { width: 1, height: 1, pixels: vec![color] };
    const FORMAT: vk::Format = vk::Format::R8G8B8A8_UNORM;

    let tex_bytes_size = 4u64;
    let (staging_buffer, staging_memory) = create_host_visible_buffer(device, memory_properties, tex_bytes_size, vk::BufferUsageFlags::TRANSFER_SRC)?;
    let ptr = device.map_memory(staging_memory, 0, tex_bytes_size, vk::MemoryMapFlags::empty()).map_err(|e| e.to_string())?;
    let dst = std::slice::from_raw_parts_mut(ptr as *mut u8, 4);
    let p = texture.pixels[0];
    dst.copy_from_slice(&[p.r, p.g, p.b, p.a]);
    device.unmap_memory(staging_memory);

    let image_info = vk::ImageCreateInfo::builder()
        .image_type(vk::ImageType::TYPE_2D)
        .format(FORMAT)
        .extent(vk::Extent3D { width: 1, height: 1, depth: 1 })
        .mip_levels(1)
        .array_layers(1)
        .samples(vk::SampleCountFlags::TYPE_1)
        .tiling(vk::ImageTiling::OPTIMAL)
        .usage(vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED)
        .sharing_mode(vk::SharingMode::EXCLUSIVE)
        .initial_layout(vk::ImageLayout::UNDEFINED);
    let image = device.create_image(&image_info, None).map_err(|e| e.to_string())?;
    let mem_req = device.get_image_memory_requirements(image);
    let mem_type = find_memory_type(memory_properties, mem_req.memory_type_bits, vk::MemoryPropertyFlags::DEVICE_LOCAL).ok_or("no device-local memory type")?;
    let alloc_info = vk::MemoryAllocateInfo::builder().allocation_size(mem_req.size).memory_type_index(mem_type);
    let memory = device.allocate_memory(&alloc_info, None).map_err(|e| e.to_string())?;
    device.bind_image_memory(image, memory, 0).map_err(|e| e.to_string())?;

    let view_info = vk::ImageViewCreateInfo::builder()
        .image(image)
        .view_type(vk::ImageViewType::TYPE_2D)
        .format(FORMAT)
        .subresource_range(vk::ImageSubresourceRange { aspect_mask: vk::ImageAspectFlags::COLOR, base_mip_level: 0, level_count: 1, base_array_layer: 0, layer_count: 1 });
    let view = device.create_image_view(&view_info, None).map_err(|e| e.to_string())?;

    let pool_info = vk::CommandPoolCreateInfo::builder().queue_family_index(queue_family_index);
    let pool = device.create_command_pool(&pool_info, None).map_err(|e| e.to_string())?;
    let cmd_alloc_info = vk::CommandBufferAllocateInfo::builder().command_pool(pool).level(vk::CommandBufferLevel::PRIMARY).command_buffer_count(1);
    let cmd = device.allocate_command_buffers(&cmd_alloc_info).map_err(|e| e.to_string())?[0];
    let begin_info = vk::CommandBufferBeginInfo::builder();
    device.begin_command_buffer(cmd, &begin_info).map_err(|e| e.to_string())?;

    let subresource = vk::ImageSubresourceRange { aspect_mask: vk::ImageAspectFlags::COLOR, base_mip_level: 0, level_count: 1, base_array_layer: 0, layer_count: 1 };
    let to_dst = vk::ImageMemoryBarrier::builder()
        .old_layout(vk::ImageLayout::UNDEFINED)
        .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
        .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE)
        .image(image)
        .subresource_range(subresource)
        .build();
    device.cmd_pipeline_barrier(cmd, vk::PipelineStageFlags::TOP_OF_PIPE, vk::PipelineStageFlags::TRANSFER, vk::DependencyFlags::empty(), &[], &[], &[to_dst]);
    let copy_region = vk::BufferImageCopy {
        buffer_offset: 0,
        buffer_row_length: 0,
        buffer_image_height: 0,
        image_subresource: vk::ImageSubresourceLayers { aspect_mask: vk::ImageAspectFlags::COLOR, mip_level: 0, base_array_layer: 0, layer_count: 1 },
        image_offset: vk::Offset3D { x: 0, y: 0, z: 0 },
        image_extent: vk::Extent3D { width: 1, height: 1, depth: 1 },
    };
    device.cmd_copy_buffer_to_image(cmd, staging_buffer, image, vk::ImageLayout::TRANSFER_DST_OPTIMAL, &[copy_region]);
    let to_read = vk::ImageMemoryBarrier::builder()
        .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
        .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
        .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
        .dst_access_mask(vk::AccessFlags::SHADER_READ)
        .image(image)
        .subresource_range(subresource)
        .build();
    device.cmd_pipeline_barrier(cmd, vk::PipelineStageFlags::TRANSFER, vk::PipelineStageFlags::FRAGMENT_SHADER, vk::DependencyFlags::empty(), &[], &[], &[to_read]);
    device.end_command_buffer(cmd).map_err(|e| e.to_string())?;

    let fence = device.create_fence(&vk::FenceCreateInfo::builder(), None).map_err(|e| e.to_string())?;
    let cmds = [cmd];
    let submit = vk::SubmitInfo::builder().command_buffers(&cmds).build();
    device.queue_submit(queue, &[submit], fence).map_err(|e| e.to_string())?;
    device.wait_for_fences(&[fence], true, u64::MAX).map_err(|e| e.to_string())?;

    device.destroy_fence(fence, None);
    device.destroy_command_pool(pool, None);
    device.destroy_buffer(staging_buffer, None);
    device.free_memory(staging_memory, None);

    Ok((image, view))
}
