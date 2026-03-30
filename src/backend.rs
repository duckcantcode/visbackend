use std::path::Path;

use creak::{AudioInfo, SampleIterator};
use spectrum_analyzer::scaling::divide_by_N;
use spectrum_analyzer::windows::hann_window;
use spectrum_analyzer::{FrequencyLimit, samples_fft_to_spectrum};

#[derive(Clone)]
pub struct SongInfo {
    pub fft: Vec<Vec<f32>>,
    pub period: f32,
    pub song_path: String,
}

fn pad<T: Clone>(vec: &mut Vec<T>, default: T, to: usize) {
    let len = vec.len();
    if len < to {
        for _ in 0..(to - len) {
            vec.push(default.clone());
        }
    }
}
fn condense(vec: Vec<f32>, bins: usize) -> Vec<f32> {
    if vec.len() < bins {
        return vec;
    }
    vec.windows(bins)
        .map(|window| window.iter().sum::<f32>())
        .collect::<Vec<_>>()
}

fn decode(path: &Path) -> Result<(SampleIterator, AudioInfo), String> {
    let decoder = match creak::Decoder::open(path) {
        Ok(dec) => dec,
        Err(err) => return Err(err.to_string()),
    };
    let info = decoder.info();
    let samples = decoder.into_samples().unwrap();
    Ok((samples, info))
}
fn fft(samples: SampleIterator, info: AudioInfo) -> (Vec<Vec<f32>>, f32) {
    // deinterleave
    let sample_vec = samples
        .into_iter()
        .map(|sample| sample.unwrap())
        .collect::<Vec<_>>()
        .chunks(info.channels())
        .map(|x| x.iter().sum::<f32>() / 2.0 as f32)
        .collect::<Vec<_>>();

    let chunk_size = 1024;
    let chunks = sample_vec.chunks(chunk_size);
    let mut frames = vec![];
    for chunk in chunks {
        let mut chunk = chunk.to_vec();
        pad::<f32>(&mut chunk, 0., chunk_size);

        let hann_window = hann_window(chunk.as_slice());
        // calc spectrum
        let spectrum_hann_window = samples_fft_to_spectrum(
            &hann_window,
            info.sample_rate(),
            FrequencyLimit::All,
            Some(&divide_by_N),
        )
        .unwrap();
        let map = spectrum_hann_window.to_map();
        let condensed = condense(map.values().copied().collect::<Vec<_>>(), map.len() - 11);
        frames.push(condensed);
    }
    (
        frames,
        (chunk_size as f32 / info.sample_rate() as f32) * 1000.0,
    )
}
pub fn conv(path: &Path) -> SongInfo {
    let loaded = decode(path).unwrap();
    let fft = fft(loaded.0, loaded.1);
    SongInfo {
        fft: fft.0,
        period: fft.1,
        song_path: path.to_str().unwrap().to_string(),
    }
}
