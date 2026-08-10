//! キャラクター別BGMをループ再生する(ユーザー指示、2026-08-10)。
//!
//! **正直な開示・著作権上の注意**: ユーザーからトラさんは「男はつらいよ」
//! (山田洋次監督、渥美清主演の映画シリーズ)のパロディ、魔法少女
//! (メイドちゃん)は「ミクミクにしてあげる♪」(初音ミクを用いたボーカロイド
//! 楽曲)のパロディという設定が提示されたが、**それら実在楽曲の旋律・
//! 音源(実際のメロディライン、歌詞、ボーカロイド音声)をそのまま再現・
//! 転用することは著作権上できない**ため、それは行わず、"下町の人情話を
//! 思わせるゆったりした旋律"(トラさんテーマ)・"軽快でポップな電子音"
//! (魔法少女テーマ)という**曲想(ジャンルの雰囲気)のみ**を参考にした、
//! 完全に新規のオリジナルメロディをそれぞれ作曲している。既存曲の旋律・
//! 拍数・構成の引用はない。
//!
//! 波形は矩形波(square wave)による簡易チップチューン合成
//! (`sample-mario`の`audio.rs`と同じ方式)。`rodio`の`SamplesBuffer`に
//! PCMサンプルを直接書き込み、`repeat_infinite()`で無限ループ再生する。

use rodio::{OutputStream, OutputStreamHandle, Sink, Source};

const SAMPLE_RATE: u32 = 44100;

fn square_wave_note(freq: f32, seconds: f32, volume: f32, samples: &mut Vec<f32>) {
    let n = (SAMPLE_RATE as f32 * seconds) as usize;
    for i in 0..n {
        let t = i as f32 / SAMPLE_RATE as f32;
        let phase = (t * freq).fract();
        let value = if phase < 0.5 { volume } else { -volume };
        let fade_start = n.saturating_sub(SAMPLE_RATE as usize / 200);
        let envelope = if i >= fade_start { (n - i) as f32 / (n - fade_start).max(1) as f32 } else { 1.0 };
        samples.push(value * envelope);
    }
}

fn render_notes(notes: &[(f32, f32)], volume: f32) -> Vec<f32> {
    let mut samples = Vec::new();
    for &(freq, dur) in notes {
        square_wave_note(freq, dur, volume, &mut samples);
    }
    samples
}

/// 助っ人「風天のトラさん」用のオリジナルテーマ。ゆったりしたテンポ・
/// 長めの音価で、下町の人情話・旅先の演歌を思わせる雰囲気を狙った
/// オリジナル旋律(既存楽曲の引用はない)。
fn compose_helper_theme() -> Vec<f32> {
    const LONG: f32 = 0.42;
    const SHORT: f32 = 0.21;
    let notes: &[(f32, f32)] = &[
        (293.66, LONG),  // D4
        (349.23, SHORT), // F4
        (392.00, LONG),  // G4
        (440.00, SHORT), // A4
        (392.00, LONG),  // G4
        (349.23, SHORT), // F4
        (293.66, LONG * 1.5), // D4(間を意識した長い着地)
        (261.63, SHORT), // C4
        (293.66, LONG * 2.0), // D4(終止)
    ];
    render_notes(notes, 0.11)
}

/// 魔法少女(メイドちゃん)用のオリジナルテーマ。速いテンポ・高い音域で、
/// ボーカロイド系の電子ポップな雰囲気を狙ったオリジナル旋律
/// (既存楽曲の引用はない)。
fn compose_maid_theme() -> Vec<f32> {
    const NOTE: f32 = 0.11;
    let notes: &[(f32, f32)] = &[
        (880.00, NOTE), // A5
        (987.77, NOTE), // B5
        (1174.66, NOTE), // D6
        (1318.51, NOTE), // E6
        (1174.66, NOTE), // D6
        (987.77, NOTE), // B5
        (880.00, NOTE), // A5
        (784.00, NOTE), // G5
        (880.00, NOTE), // A5
        (987.77, NOTE), // B5
        (1046.50, NOTE), // C6
        (987.77, NOTE * 1.5), // B5
        (880.00, NOTE * 2.0), // A5(終止)
    ];
    render_notes(notes, 0.09)
}

/// どちらのテーマを再生するか。
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    Helper,
    Maid,
}

/// BGMを再生開始する。返り値を呼び出し側で保持し続けている間だけ再生
/// される(dropすると停止する)。再生に失敗した場合(オーディオ出力
/// デバイスが無い等)は`None`を返し、BGM無しで続行できるようにする。
pub fn play_theme(theme: Theme) -> Option<(OutputStream, OutputStreamHandle, Sink)> {
    let (stream, handle) = match OutputStream::try_default() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("BGM再生用のオーディオ出力デバイスを開けませんでした(BGM無しで続行します): {e}");
            return None;
        }
    };
    let sink = match Sink::try_new(&handle) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("BGM再生用のSinkを作成できませんでした(BGM無しで続行します): {e}");
            return None;
        }
    };
    let melody = match theme {
        Theme::Helper => compose_helper_theme(),
        Theme::Maid => compose_maid_theme(),
    };
    let source = rodio::buffer::SamplesBuffer::new(1, SAMPLE_RATE, melody).repeat_infinite();
    sink.append(source);
    Some((stream, handle, sink))
}
