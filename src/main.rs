use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::mpsc::{channel, Sender};
use tokio::runtime::Runtime;
use tonic::Request;
use prost_types::{Duration, Timestamp};
use crate::protos::google::cloud::speech::v1::speech_client::SpeechClient;
use crate::protos::google::cloud::speech::v1::{RecognitionConfig, StreamingRecognizeRequest, StreamingRecognitionConfig};

pub mod protos {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/protos/google.cloud.speech.v1.rs"));
}

const SAMPLE_RATE: u32 = 48000;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = SpeechClient::connect("https://speech.googleapis.com").await?;
    let (tx, rx) = channel();

    // Ses cihazı ayarları
    let host = cpal::default_host();
    let device = host.default_input_device().expect("Giriş cihazı bulunamadı");
    let config = device.default_input_config().expect("Varsayılan konfigürasyon alınamadı");

    // Ses akışını başlat
    let stream = device.build_input_stream(
        &config.config(),
        move |data: &[f32], _: &cpal::InputCallbackInfo| {
            let samples: Vec<i16> = data.iter().map(|&sample| (sample * 32767.0) as i16).collect();
            tx.send(samples).unwrap();
        },
        |err| {
            eprintln!("Hata oluştu: {:?}", err);
        },
        None,
    )?;

    stream.play()?;

    let recognition_config = RecognitionConfig {
        encoding: 3, // LINEAR16
        sample_rate_hertz: SAMPLE_RATE as i32,
        language_code: "tr-TR".to_string(),
        ..Default::default()
    };

    let streaming_config = StreamingRecognitionConfig {
        config: Some(recognition_config),
        interim_results: true,
        ..Default::default()
    };

    let mut request = client.streaming_recognize(Request::new(streaming_config)).await?.into_inner();

    while let Ok(samples) = rx.recv() {
        let audio_content = samples.iter().flat_map(|&sample| sample.to_le_bytes().to_vec()).collect::<Vec<u8>>();
        request.send(StreamingRecognizeRequest {
            streaming_request: Some(crate::protos::streaming_recognize_request::StreamingRequest::AudioContent(audio_content)),
        }).await?;
    }

    while let Some(response) = request.message().await? {
        for result in response.results {
            if let Some(alternative) = result.alternatives.into_iter().next() {
                println!("Transcription: {}", alternative.transcript);
            }
        }
    }

    Ok(())
}
