use constellation::matrix::MatrixEngine;
use tempfile::tempdir;
use wiremock::matchers::{method, path, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn setup_mock_versions(mock_server: &MockServer) {
    Mock::given(method("GET"))
        .and(path("/_matrix/client/versions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "versions": ["v1.1", "v1.2", "v1.3", "v1.4", "v1.5", "v1.6", "v1.7", "v1.8", "v1.9", "v1.10", "v1.11"],
            "unstable_features": {}
        })))
        .mount(mock_server)
        .await;
}

#[tokio::test]
async fn test_matrix_engine_new_with_mock_homeserver() {
    let mock_server = MockServer::start().await;
    setup_mock_versions(&mock_server).await;

    let tmp_dir = tempdir().expect("failed to create tempdir");
    let passphrase = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string();

    let engine = MatrixEngine::new_with_options(
        tmp_dir.path().to_path_buf(),
        &mock_server.uri(),
        Some(passphrase),
    )
    .await;

    assert!(
        engine.is_ok(),
        "Engine should initialize with mock homeserver and test passphrase: {:?}",
        engine.err()
    );
    let _engine = engine.unwrap();

    // Verify data directory structure was created
    assert!(tmp_dir.path().join("matrix-store").exists());
}

#[tokio::test]
async fn test_matrix_engine_sqlite_store_persistence_across_restarts() {
    let mock_server = MockServer::start().await;
    setup_mock_versions(&mock_server).await;

    let tmp_dir = tempdir().expect("failed to create tempdir");
    let passphrase = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string();

    // First instance: create store
    {
        let engine = MatrixEngine::new_with_options(
            tmp_dir.path().to_path_buf(),
            &mock_server.uri(),
            Some(passphrase.clone()),
        )
        .await
        .expect("Failed to initialize first engine");
        drop(engine);
    }

    // Verify store directory still exists
    assert!(tmp_dir.path().join("matrix-store").exists());

    // Second instance: reopen the existing store with the same passphrase
    let reopened = MatrixEngine::new_with_options(
        tmp_dir.path().to_path_buf(),
        &mock_server.uri(),
        Some(passphrase),
    )
    .await;

    assert!(
        reopened.is_ok(),
        "Engine should successfully reopen existing SQLite store: {:?}",
        reopened.err()
    );
}

#[tokio::test]
async fn test_matrix_engine_with_client() {
    let mock_server = MockServer::start().await;
    setup_mock_versions(&mock_server).await;

    let tmp_dir = tempdir().expect("failed to create tempdir");
    let client = matrix_sdk::Client::builder()
        .homeserver_url(mock_server.uri())
        .build()
        .await
        .expect("failed to build client");

    let engine = MatrixEngine::with_client(tmp_dir.path().to_path_buf(), client).await;
    // Verify engine was constructed with data_dir
    assert_eq!(engine.data_dir().await, tmp_dir.path());
}

#[tokio::test]
async fn test_matrix_engine_create_room_flow() {
    let mock_server = MockServer::start().await;
    setup_mock_versions(&mock_server).await;

    Mock::given(method("POST"))
        .and(path_regex(r"^/_matrix/client/.*?/createRoom$"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "room_id": "!integration_test_room:example.com"
        })))
        .mount(&mock_server)
        .await;

    let tmp_dir = tempdir().expect("failed to create tempdir");
    let client = matrix_sdk::test_utils::logged_in_client(Some(mock_server.uri())).await;
    let engine = MatrixEngine::with_client(tmp_dir.path().to_path_buf(), client).await;
    let room_id = engine.create_room("Integration Room", false).await;
    assert!(
        room_id.is_ok(),
        "create_room should succeed: {:?}",
        room_id.err()
    );
    assert_eq!(
        room_id.unwrap().as_str(),
        "!integration_test_room:example.com"
    );
}
