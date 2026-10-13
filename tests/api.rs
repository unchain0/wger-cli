use mockito::{Matcher, Server};
use serde_json::json;
use wger_cli::{mcp, Client};

#[test]
fn list_sends_token_and_query() {
    let mut s = Server::new();
    let m = s
        .mock("GET", "/api/v2/workoutlog/")
        .match_header("authorization", "Token secret")
        .match_query(Matcher::UrlEncoded("routine".into(), "80246".into()))
        .with_body(r#"{"count":1,"next":null,"results":[{"id":1}]}"#)
        .create();
    let c = Client::new(&s.url(), Some("secret".into()));
    let v = c
        .list("workoutlog", &[("routine".into(), "80246".into())], false)
        .unwrap();
    assert_eq!(v["results"][0]["id"], 1);
    m.assert();
}

#[test]
fn list_all_follows_next() {
    let mut s = Server::new();
    let next = format!("{}/api/v2/day/?offset=1", s.url());
    s.mock("GET", "/api/v2/day/")
        .match_query(Matcher::Missing)
        .with_body(json!({"next": next, "results": [{"id": 1}]}).to_string())
        .create();
    s.mock("GET", "/api/v2/day/")
        .match_query(Matcher::UrlEncoded("offset".into(), "1".into()))
        .with_body(r#"{"next":null,"results":[{"id":2}]}"#)
        .create();
    let v = Client::new(&s.url(), None).list("day", &[], true).unwrap();
    assert_eq!(v["count"], 2);
}

#[test]
fn unknown_resource_rejected() {
    assert!(Client::new("http://x", None).get("user", "1").is_err());
}

#[test]
fn writes_require_token() {
    let c = Client::new("http://127.0.0.1:9", None);
    let e = c
        .update("day", "1", &json!({"name": "A"}))
        .unwrap_err()
        .to_string();
    assert!(e.contains("WGER_API_TOKEN"));
}

#[test]
fn delete_requires_confirmation() {
    let mut s = Server::new();
    let m = s
        .mock("DELETE", "/api/v2/slot/5/")
        .with_status(204)
        .create();
    let c = Client::new(&s.url(), Some("t".into()));
    assert!(c.delete("slot", "5", false).is_err());
    m.expect(0);
    let mut s2 = Server::new();
    let m2 = s2
        .mock("DELETE", "/api/v2/slot/5/")
        .with_status(204)
        .create();
    let c2 = Client::new(&s2.url(), Some("t".into()));
    assert_eq!(c2.delete("slot", "5", true).unwrap()["ok"], true);
    m2.assert();
}

#[test]
fn patch_sends_body() {
    let mut s = Server::new();
    let m = s
        .mock("PATCH", "/api/v2/slot-entry/9/")
        .match_body(Matcher::Json(json!({"comment": "x"})))
        .with_body(r#"{"id":9,"comment":"x"}"#)
        .create();
    let c = Client::new(&s.url(), Some("t".into()));
    assert_eq!(
        c.update("slot-entry", "9", &json!({"comment": "x"}))
            .unwrap()["id"],
        9
    );
    m.assert();
}

#[test]
fn translation_updates_existing() {
    let mut s = Server::new();
    s.mock("GET", "/api/v2/language/")
        .match_query(Matcher::Any)
        .with_body(r#"{"results":[{"id":7,"short_name":"pt"}]}"#)
        .create();
    s.mock("GET", "/api/v2/exercise-translation/")
        .match_query(Matcher::AllOf(vec![
            Matcher::UrlEncoded("exercise".into(), "10".into()),
            Matcher::UrlEncoded("language".into(), "7".into()),
        ]))
        .with_body(r#"{"results":[{"id":54,"exercise":10,"language":2},{"id":55,"exercise":10,"language":7}]}"#)
        .create();
    let m = s
        .mock("PATCH", "/api/v2/exercise-translation/55/")
        .match_body(Matcher::Json(json!({"description": "<p>oi</p>"})))
        .with_body(r#"{"id":55}"#)
        .create();
    let c = Client::new(&s.url(), Some("t".into()));
    let v = c
        .translation_set(10, "pt", None, Some("<p>oi</p>"), None)
        .unwrap();
    assert_eq!(v["action"], "updated");
    m.assert();
}

#[test]
fn translation_creates_new() {
    let mut s = Server::new();
    s.mock("GET", "/api/v2/exercise-translation/")
        .match_query(Matcher::Any)
        .with_body(r#"{"results":[]}"#)
        .create();
    let m = s.mock("POST", "/api/v2/exercise-translation/")
        .match_body(Matcher::Json(json!({"exercise": 10, "language": 7, "name": "Agachamento", "description": "<p>d</p>"})))
        .with_status(201).with_body(r#"{"id":56}"#).create();
    let c = Client::new(&s.url(), Some("t".into()));
    assert_eq!(
        c.translation_set(10, "7", Some("Agachamento"), Some("<p>d</p>"), None)
            .unwrap()["action"],
        "created"
    );
    m.assert();
}

#[test]
fn api_error_surfaces_status() {
    let mut s = Server::new();
    s.mock("GET", "/api/v2/routine/1/")
        .with_status(404)
        .with_body(r#"{"detail":"Not found."}"#)
        .create();
    let e = Client::new(&s.url(), Some("t".into()))
        .get("routine", "1")
        .unwrap_err()
        .to_string();
    assert!(e.contains("404") && !e.contains("Token t"));
}

#[test]
fn mcp_protocol() {
    let c = Client::new("http://127.0.0.1:9", None);
    let init = mcp::handle(&c, &json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18"}})).unwrap();
    assert_eq!(init["result"]["serverInfo"]["name"], "wger");
    assert!(mcp::handle(
        &c,
        &json!({"jsonrpc":"2.0","method":"notifications/initialized"})
    )
    .is_none());
    let tl = mcp::handle(&c, &json!({"jsonrpc":"2.0","id":2,"method":"tools/list"})).unwrap();
    assert_eq!(tl["result"]["tools"].as_array().unwrap().len(), 7);
    let del = mcp::handle(&c, &json!({"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"wger_delete","arguments":{"resource":"day","id":1,"confirm":false}}})).unwrap();
    assert_eq!(del["result"]["isError"], true);
}

#[test]
fn translation_ignores_other_language_rows() {
    let mut s = Server::new();
    s.mock("GET", "/api/v2/exercise-translation/")
        .match_query(Matcher::Any)
        .with_body(r#"{"results":[{"id":54,"exercise":10,"language":2}]}"#)
        .create();
    let m = s
        .mock("POST", "/api/v2/exercise-translation/")
        .with_status(201)
        .with_body(r#"{"id":56}"#)
        .create();
    let c = Client::new(&s.url(), Some("t".into()));
    assert_eq!(
        c.translation_set(10, "7", Some("A"), Some("<p>d</p>"), None)
            .unwrap()["action"],
        "created"
    );
    m.assert();
}

#[test]
fn uuid_ids_accepted() {
    let mut s = Server::new();
    let m = s
        .mock(
            "GET",
            "/api/v2/workoutlog/0b6c1c9e-1f2a-4c1e-9a3b-2d4e5f6a7b8c/",
        )
        .with_body(r#"{"id":"x"}"#)
        .create();
    let c = Client::new(&s.url(), Some("t".into()));
    c.get("workoutlog", "0b6c1c9e-1f2a-4c1e-9a3b-2d4e5f6a7b8c")
        .unwrap();
    m.assert();
    assert!(c.get("workoutlog", "../x").is_err());
}
