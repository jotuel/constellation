use constellation::constellation::{Message, Tab, app};
use constellation::matrix::{MatrixEngine, RoomData};
use constellation::settings::config::Config;
use cosmic::{Application, Core};
use tempfile::tempdir;
use wiremock::MockServer;

#[tokio::test]
async fn test_app_init_and_engine_ready() {
    let mock_server = MockServer::start().await;
    let tmp_dir = tempdir().expect("failed to create tempdir");
    let client = matrix_sdk::test_utils::logged_in_client(Some(mock_server.uri())).await;
    let engine = MatrixEngine::with_client(tmp_dir.path().to_path_buf(), client).await;

    let mut application = app(Core::default(), Config::default());
    assert!(application.matrix().is_none());

    let _task = application.update(Message::EngineReady(Ok(engine)));
    assert!(application.matrix().is_some());
}

#[test]
fn test_app_room_selection_and_tab_lifecycle() {
    let mut application = app(Core::default(), Config::default());

    let room1_id: std::sync::Arc<str> = std::sync::Arc::from("!room1:example.com");
    let room2_id: std::sync::Arc<str> = std::sync::Arc::from("!room2:example.com");

    let room1 = RoomData {
        id: room1_id.clone(),
        name: Some("General".to_string()),
        last_message: None,
        unread_count: 0,
        unread_count_str: None,
        avatar_url: None,
        room_type: None,
        is_space: false,
        parent_space_id: None,
        order: None,
        join_rule: None,
        allowed_spaces: Vec::new(),
        suggested: false,
    };

    let room2 = RoomData {
        id: room2_id.clone(),
        name: Some("Random".to_string()),
        last_message: None,
        unread_count: 0,
        unread_count_str: None,
        avatar_url: None,
        room_type: None,
        is_space: false,
        parent_space_id: None,
        order: None,
        join_rule: None,
        allowed_spaces: Vec::new(),
        suggested: false,
    };

    application.set_rooms_for_test(vec![room1, room2]);
    assert_eq!(application.room_list().len(), 2);

    // 1. Select a room -> room is selected and room tab is opened
    let _ = application.update(Message::RoomSelected(room1_id.clone()));
    assert_eq!(application.selected_room(), Some(&room1_id));
    assert_eq!(application.open_tabs(), &[Tab::Room(room1_id.clone())]);

    // 2. Select a second room -> tabs grow
    let _ = application.update(Message::RoomSelected(room2_id.clone()));
    assert_eq!(application.selected_room(), Some(&room2_id));
    assert_eq!(
        application.open_tabs(),
        &[Tab::Room(room1_id.clone()), Tab::Room(room2_id.clone())]
    );

    // 3. Close first room tab -> activates adjacent tab
    let _ = application.update(Message::CloseRoom(room1_id));
    assert_eq!(application.open_tabs(), &[Tab::Room(room2_id.clone())]);
    assert_eq!(application.selected_room(), Some(&room2_id));
}

#[test]
fn test_app_search_and_tab_flow() {
    let mut application = app(Core::default(), Config::default());

    // Enter search query
    let _ = application.update(Message::SearchQueryChanged("matrix client".to_string()));
    assert_eq!(application.search_query(), "matrix client");

    // Launch search -> creates search tab
    let _ = application.update(Message::SubmitSearch);
    assert_eq!(
        application.open_tabs(),
        &[Tab::Search {
            room_id: None,
            query: "matrix client".to_string()
        }]
    );

    // Close the search tab
    let search_tab = application.open_tabs()[0].clone();
    let _ = application.update(Message::CloseTab(search_tab));
    assert!(application.open_tabs().is_empty());
}

#[test]
fn test_app_logout_clears_session_state() {
    let mut application = app(Core::default(), Config::default());

    let room_id: std::sync::Arc<str> = std::sync::Arc::from("!room:example.com");
    application.set_rooms_for_test(vec![RoomData {
        id: room_id.clone(),
        name: Some("Room".to_string()),
        last_message: None,
        unread_count: 0,
        unread_count_str: None,
        avatar_url: None,
        room_type: None,
        is_space: false,
        parent_space_id: None,
        order: None,
        join_rule: None,
        allowed_spaces: Vec::new(),
        suggested: false,
    }]);
    application.set_selected_room_for_test(Some(room_id));
    application.set_user_id_for_test(Some("@alice:example.com".to_string()));

    assert!(application.user_id().is_some());
    assert!(!application.room_list().is_empty());
    assert!(application.selected_room().is_some());
    assert!(!application.open_tabs().is_empty());

    // Dispatch logout
    let _ = application.update(Message::LogoutFinished);
    assert!(application.user_id().is_none());
    assert!(application.room_list().is_empty());
    assert!(application.selected_room().is_none());
    assert!(application.open_tabs().is_empty());
}
