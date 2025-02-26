#[cfg(target_os = "windows")]
#[cfg(test)]
mod tests {
    use screenpipe_vision::{get_monitor, process_ocr_task, OcrEngine};
    use std::sync::Arc;
    use std::{path::PathBuf, time::Instant};
    use tokio::sync::{mpsc, Mutex};

    use screenpipe_vision::{continuous_capture, CaptureResult};
    use std::time::Duration;
    use tokio::time::timeout;

    #[cfg(target_os = "windows")]
    #[tokio::test]
    async fn test_process_ocr_task_windows() {
        // Use an absolute path that works in both local and CI environments
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("tests");
        path.push("testing_OCR.png");
        println!("Path to testing_OCR.png: {:?}", path);
        let image = image::open(&path).expect("Failed to open image");

        let image_arc = Arc::new(image);
        let frame_number = 1;
        let timestamp = Instant::now();
        let (tx, _rx) = mpsc::channel(1);
        let previous_text_json = Arc::new(Mutex::new(None));
        let ocr_engine = Arc::new(OcrEngine::WindowsNative);
        let app_name = "test_app".to_string();

        let result = process_ocr_task(
            image_arc,
            frame_number,
            timestamp,
            tx,
            &previous_text_json,
            false,
            ocr_engine,
            app_name,
        )
        .await;

        assert!(result.is_ok());
        // Add more specific assertions based on expected behavior
    }

    #[tokio::test]
    #[ignore] // TODO require UI
    async fn test_continuous_capture() {
        // Create channels for communication
        let (result_tx, mut result_rx) = mpsc::channel::<CaptureResult>(10);

        // Create a mock monitor
        let monitor = get_monitor().await;

        // Set up test parameters
        let interval = Duration::from_millis(1000);
        let save_text_files_flag = false;
        let ocr_engine = Arc::new(OcrEngine::WindowsNative);

        // Spawn the continuous_capture function
        let capture_handle = tokio::spawn(continuous_capture(
            result_tx,
            interval,
            save_text_files_flag,
            ocr_engine,
            monitor,
        ));

        // Wait for a short duration to allow some captures to occur
        let timeout_duration = Duration::from_secs(5);
        let _result = timeout(timeout_duration, async {
            let mut capture_count = 0;
            while let Some(capture_result) = result_rx.recv().await {
                capture_count += 1;
                // assert!(
                //     capture_result.image.width() == 100 && capture_result.image.height() == 100
                // );
                println!("capture_result: {:?}\n\n", capture_result.text);
                if capture_count >= 3 {
                    break;
                }
            }
        })
        .await;

        // Stop the continuous_capture task
        capture_handle.abort();

        // Assert that we received some results without timing out
        // assert!(
        //     result.is_ok(),
        //     "Test timed out or failed to receive captures"
        // );
    }
}
