// cargo bench --bench db_benchmarks

use criterion::{criterion_group, criterion_main, Criterion};
use rand::Rng;
use screenpipe_server::{ContentType, DatabaseManager};
use tokio::runtime::Runtime;

async fn setup_large_db(size: usize) -> DatabaseManager {
    let db = DatabaseManager::new("sqlite::memory:").await.unwrap();
    let mut rng = rand::thread_rng();

    for _ in 0..size {
        let _video_id = db.insert_video_chunk("test_video.mp4").await.unwrap();
        let frame_id = db.insert_frame().await.unwrap();
        let ocr_text = format!("OCR text {}", rng.gen::<u32>());
        let text_json = format!(r#"{{"text": "{}"}}"#, ocr_text);
        let new_text_json_vs_previous_frame = format!(r#"{{"text": "{}"}}"#, ocr_text);
        let raw_data_output_from_ocr = format!(r#"{{"output": "{}"}}"#, ocr_text);
        db.insert_ocr_text(
            frame_id,
            &ocr_text,
            &text_json,
            &new_text_json_vs_previous_frame,
            &raw_data_output_from_ocr,
        )
        .await
        .unwrap();

        let audio_id = db.insert_audio_chunk("test_audio.mp4").await.unwrap();
        let audio_text = format!("Audio transcription {}", rng.gen::<u32>());
        db.insert_audio_transcription(audio_id, &audio_text, 0)
            .await
            .unwrap();
    }

    db
}

fn bench_search(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let db_sizes = [10000, 100000];
    let content_types = [ContentType::OCR, ContentType::Audio, ContentType::All];
    let search_queries = ["random", "specific"];

    let mut group = c.benchmark_group("search_benchmarks");
    group.sample_size(5);
    group.measurement_time(std::time::Duration::from_secs(30));

    for &size in &db_sizes {
        for &content_type in &content_types {
            for &query in &search_queries {
                group.bench_function(
                    format!("{:?}_db_size_{}_query_{}", content_type, size, query),
                    |b| {
                        b.to_async(&rt).iter(|| async {
                            let db = setup_large_db(size).await;
                            db.search(query, content_type, 100, 0, None, None)
                                .await
                                .unwrap()
                        });
                    },
                );
            }
        }
    }

    group.finish();
}

criterion_group!(benches, bench_search);
criterion_main!(benches);
