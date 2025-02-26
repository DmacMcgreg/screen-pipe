#[cfg(test)]
mod tests {
    use chrono::Utc;
    use screenpipe_server::{ContentType, DatabaseManager, SearchResult};

    async fn setup_test_db() -> DatabaseManager {
        DatabaseManager::new("sqlite::memory:").await.unwrap()
    }

    #[tokio::test]
    async fn test_insert_and_search_ocr() {
        let db = setup_test_db().await;
        let _ = db.insert_video_chunk("test_video.mp4").await.unwrap();
        let frame_id = db.insert_frame("").await.unwrap();
        db.insert_ocr_text(frame_id, "Hello, world!", "", "", "", "")
            .await
            .unwrap();

        let results = db
            .search("Hello", ContentType::OCR, 100, 0, None, None, None)
            .await
            .unwrap();
        assert_eq!(results.len(), 1);
        if let SearchResult::OCR(ocr_result) = &results[0] {
            assert_eq!(ocr_result.ocr_text, "Hello, world!");
            assert_eq!(ocr_result.file_path, "test_video.mp4");
        } else {
            panic!("Expected OCR result");
        }
    }

    #[tokio::test]
    async fn test_insert_and_search_audio() {
        let db = setup_test_db().await;
        let audio_chunk_id = db.insert_audio_chunk("test_audio.mp4").await.unwrap();
        db.insert_audio_transcription(audio_chunk_id, "Hello from audio", 0)
            .await
            .unwrap();

        let results = db
            .search("audio", ContentType::Audio, 100, 0, None, None, None)
            .await
            .unwrap();
        assert_eq!(results.len(), 1);
        if let SearchResult::Audio(audio_result) = &results[0] {
            assert_eq!(audio_result.transcription, "Hello from audio");
            assert_eq!(audio_result.file_path, "test_audio.mp4");
        } else {
            panic!("Expected Audio result");
        }
    }

    #[tokio::test]
    async fn test_search_all() {
        let db = setup_test_db().await;

        // Insert OCR data
        let _ = db.insert_video_chunk("test_video.mp4").await.unwrap();
        let frame_id = db.insert_frame("").await.unwrap();
        db.insert_ocr_text(frame_id, "Hello from OCR", "", "", "", "")
            .await
            .unwrap();

        // Insert Audio data
        let audio_chunk_id = db.insert_audio_chunk("test_audio.mp4").await.unwrap();
        db.insert_audio_transcription(audio_chunk_id, "Hello from audio", 0)
            .await
            .unwrap();

        let results = db
            .search("Hello", ContentType::All, 100, 0, None, None, None)
            .await
            .unwrap();
        assert_eq!(results.len(), 2);

        let ocr_count = results
            .iter()
            .filter(|r| matches!(r, SearchResult::OCR(_)))
            .count();
        let audio_count = results
            .iter()
            .filter(|r| matches!(r, SearchResult::Audio(_)))
            .count();

        assert_eq!(ocr_count, 1);
        assert_eq!(audio_count, 1);
    }

    #[tokio::test]
    async fn test_search_with_time_range() {
        let db = setup_test_db().await;

        let start_time = Utc::now();

        // Insert OCR data
        let _ = db.insert_video_chunk("test_video.mp4").await.unwrap();
        let frame_id1 = db.insert_frame("").await.unwrap();
        db.insert_ocr_text(frame_id1, "Hello from OCR 1", "", "", "", "")
            .await
            .unwrap();

        // Insert first audio data
        let audio_chunk_id = db.insert_audio_chunk("test_audio.mp4").await.unwrap();
        db.insert_audio_transcription(audio_chunk_id, "Hello from audio 1", 0)
            .await
            .unwrap();

        // Wait for a short time to ensure timestamp difference
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        let mid_time = Utc::now();

        // Wait for another short time
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        // Insert remaining data
        let frame_id2 = db.insert_frame("").await.unwrap();
        db.insert_ocr_text(frame_id2, "Hello from OCR 2", "", "", "", "")
            .await
            .unwrap();

        db.insert_audio_transcription(audio_chunk_id, "Hello from audio 2", 1)
            .await
            .unwrap();

        let end_time = Utc::now();

        // Test search with full time range
        let results = db
            .search(
                "Hello",
                ContentType::All,
                100,
                0,
                Some(start_time),
                Some(end_time),
                None,
            )
            .await
            .unwrap();
        println!("Full time range results: {:?}", results);
        assert_eq!(results.len(), 4, "Expected 4 results for full time range");

        // Test search with limited time range
        let results = db
            .search(
                "Hello",
                ContentType::All,
                100,
                0,
                Some(mid_time),
                Some(end_time),
                None,
            )
            .await
            .unwrap();
        println!("Limited time range results: {:?}", results);
        assert_eq!(
            results.len(),
            2,
            "Expected 2 results for limited time range"
        );

        // Test search with OCR content type and time range
        let results = db
            .search(
                "Hello",
                ContentType::OCR,
                100,
                0,
                Some(start_time),
                Some(end_time),
                None,
            )
            .await
            .unwrap();
        assert_eq!(results.len(), 2);

        // Test search with Audio content type and time range
        let results = db
            .search(
                "Hello",
                ContentType::Audio,
                100,
                0,
                Some(start_time),
                Some(end_time),
                None,
            )
            .await
            .unwrap();
        assert_eq!(results.len(), 2);
    }

    #[tokio::test]
    async fn test_count_search_results_with_time_range() {
        let db = setup_test_db().await;

        let start_time = Utc::now();

        // Insert OCR data
        let _ = db.insert_video_chunk("test_video.mp4").await.unwrap();
        let frame_id1 = db.insert_frame("").await.unwrap();
        db.insert_ocr_text(frame_id1, "Hello from OCR 1", "", "", "", "")
            .await
            .unwrap();

        // Insert first audio data
        let audio_chunk_id = db.insert_audio_chunk("test_audio.mp4").await.unwrap();
        db.insert_audio_transcription(audio_chunk_id, "Hello from audio 1", 0)
            .await
            .unwrap();

        // Capture mid_time after inserting half of the data
        let mid_time = Utc::now();

        // Wait for a short time to ensure timestamp difference
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        // Insert remaining data
        let frame_id2 = db.insert_frame("").await.unwrap();
        db.insert_ocr_text(frame_id2, "Hello from OCR 2", "", "", "", "")
            .await
            .unwrap();

        db.insert_audio_transcription(audio_chunk_id, "Hello from audio 2", 1)
            .await
            .unwrap();

        let end_time = Utc::now();

        // Test search with limited time range
        let results = db
            .search(
                "Hello",
                ContentType::All,
                100,
                0,
                Some(mid_time),
                Some(end_time),
                None,
            )
            .await
            .unwrap();

        println!("Limited time range results: {:?}", results);
        assert_eq!(
            results.len(),
            2,
            "Expected 2 results for limited time range"
        );

        // Test count with Audio content type and time range
        let count = db
            .count_search_results(
                "Hello",
                ContentType::Audio,
                Some(start_time),
                Some(end_time),
                None,
            )
            .await
            .unwrap();
        assert_eq!(count, 2);
    }
}
