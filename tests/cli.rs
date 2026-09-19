use constellation::{is_launch_uri, parse_launch_uri};

#[test]
fn test_is_launch_uri_oidc_callback() {
    // Single-slash format required by MAS
    assert!(is_launch_uri(
        "fi.joonastuomi.constellation:/callback?code=test_code_12345&state=test_state"
    ));
    // Legacy double-slash format
    assert!(is_launch_uri(
        "fi.joonastuomi.constellation://callback?code=test_code_12345"
    ));
}

#[test]
fn test_is_launch_uri_internal_open() {
    assert!(is_launch_uri(
        "fi.joonastuomi.constellation://open?url=https%3A%2F%2Fmatrix.to%2F%23%2F!room%3Aexample.org"
    ));
}

#[test]
fn test_is_launch_uri_matrix_permalinks() {
    assert!(is_launch_uri("matrix:r/general:matrix.org"));
    assert!(is_launch_uri("matrix:roomid/!someroom:matrix.org"));
    assert!(is_launch_uri("https://matrix.to/#/!room:matrix.org"));
    assert!(is_launch_uri("https://matrix.to/#/@user:matrix.org"));
}

#[test]
fn test_is_launch_uri_rejects_unrelated_schemes() {
    assert!(!is_launch_uri("https://example.com"));
    assert!(!is_launch_uri("http://localhost:8080"));
    assert!(!is_launch_uri("file:///etc/passwd"));
    assert!(!is_launch_uri("fi.other.app:/callback"));
    assert!(!is_launch_uri(""));
    assert!(!is_launch_uri("random plain text string"));
}

#[test]
fn test_parse_launch_uri_cli_args() {
    let empty_args: Vec<String> = vec!["constellation".to_string()];
    assert_eq!(parse_launch_uri(&empty_args), None);

    let uri_args: Vec<String> = vec![
        "constellation".to_string(),
        "fi.joonastuomi.constellation:/callback?code=abc".to_string(),
    ];
    assert_eq!(
        parse_launch_uri(&uri_args),
        Some("fi.joonastuomi.constellation:/callback?code=abc".to_string())
    );

    let notify_args: Vec<String> = vec!["constellation".to_string(), "--notify".to_string()];
    assert_eq!(parse_launch_uri(&notify_args), None);

    let invalid_args: Vec<String> = vec![
        "constellation".to_string(),
        "https://random-site.org".to_string(),
    ];
    assert_eq!(parse_launch_uri(&invalid_args), None);
}

#[test]
fn test_is_notify_flag_detection() {
    let args1 = ["constellation".to_string(), "--notify".to_string()];
    assert!(args1.iter().any(|arg| arg == "--notify"));

    let args2 = [
        "constellation".to_string(),
        "fi.joonastuomi.constellation:/callback".to_string(),
    ];
    assert!(!args2.iter().any(|arg| arg == "--notify"));
}
