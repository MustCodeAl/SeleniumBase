use std::collections::HashMap;
use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post},
    Router,
};
use serde_json::{json, Value};
use tower_http::cors::CorsLayer;
use uuid::Uuid;

use seleniumbase_rs::BaseCase;

use crate::models::*;
use crate::store::{apply_profile_overrides, build_config, make_session_id, set_cookies, AppState};

pub type ApiError = (StatusCode, Json<ApiResponse<Value>>);
pub type ApiResult<T> = Result<Json<ApiResponse<T>>, ApiError>;

#[allow(clippy::result_large_err)]
fn err<T>(code: u16, msg: impl Into<String>) -> ApiResult<T> {
    let status = ApiStatus {
        error_code: "ERROR".into(),
        http_code: code,
        message: msg.into(),
    };
    Err((
        StatusCode::from_u16(code).unwrap_or(StatusCode::BAD_REQUEST),
        Json(ApiResponse::err(status)),
    ))
}

#[allow(clippy::result_large_err)]
fn ok<T>(data: T) -> ApiResult<T> {
    Ok(Json(ApiResponse::ok(data)))
}

#[allow(clippy::result_large_err)]
fn ok_msg<T>(data: T, msg: impl Into<String>) -> ApiResult<T> {
    Ok(Json(ApiResponse::ok_msg(data, msg)))
}

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/v1/version", get(version))
        .route("/api/v1/status", get(status_all))
        .route("/api/v1/profile_status", get(profile_status))
        .route("/api/v1/profiles", get(profile_search).post(profile_create))
        .route(
            "/api/v1/profiles/:id",
            get(profile_get).post(profile_update).delete(profile_delete),
        )
        .route("/api/v1/profiles/:id/start", get(profile_start))
        .route("/api/v1/profiles/:id/stop", get(profile_stop))
        .route("/api/v1/profiles/:id/clone", post(profile_clone))
        .route("/api/v1/profiles/:id/export", get(profile_export))
        .route("/api/v1/profiles/import", post(profile_import))
        .route("/api/v1/cookie_import", post(cookie_import))
        .route("/api/v1/cookie_export", post(cookie_export))
        .route("/api/v1/proxy/validate", post(proxy_validate))
        .route("/api/v1/tags", get(tag_list).post(tag_create))
        .route("/api/v1/tags/:id", post(tag_update).delete(tag_delete))
        .route("/api/v1/folders", get(folder_list).post(folder_create))
        .route(
            "/api/v1/folders/:id",
            post(folder_update).delete(folder_delete),
        )
        .route("/api/v1/screen_resolution", get(screen_resolution))
        .route("/api/v1/script_runner/start", post(script_runner_start))
        .route("/api/v1/script_runner/stop", post(script_runner_stop))
        .route("/api/v1/browser_cores", get(browser_core_list))
        .route("/api/v1/load_browser_core", post(load_browser_core))
        .route("/api/v1/delete_browser_core", delete(delete_browser_core))
        .route("/api/v1/stop_all", get(stop_all))
        .route("/api/v1/workspaces", get(workspaces))
        .route("/api/v1/user/signin", post(user_signin))
        .route("/api/v1/user/refresh_token", post(user_refresh_token))
        .route("/api/v1/bookmarks/export", post(bookmarks_export))
        .route("/api/v1/bookmarks/import", post(bookmarks_import))
        .route("/api/v1/2fa/setup", post(twofa_setup))
        .route("/api/v1/2fa/enable", post(twofa_enable))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

async fn version() -> ApiResult<Value> {
    ok_msg(
        json!({ "version": "0.1.0", "launcher": "seleniumbase-rs" }),
        "",
    )
}

async fn status_all(State(state): State<Arc<AppState>>) -> ApiResult<Value> {
    let sessions = state.session_info.lock().await.clone();
    ok_msg(json!({ "sessions": sessions }), "")
}

async fn profile_status(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
) -> ApiResult<Value> {
    let id = params.get("profile_id").cloned().unwrap_or_default();
    let active = state
        .session_info
        .lock()
        .await
        .values()
        .any(|s| s.profile_id == id);
    ok_msg(json!({ "profile_id": id, "active": active }), "")
}

async fn profile_search(State(state): State<Arc<AppState>>) -> ApiResult<Vec<Profile>> {
    let profiles = state.profiles.lock().await.clone();
    ok(profiles)
}

async fn profile_create(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<NewProfile>,
) -> ApiResult<Profile> {
    let profile = Profile {
        id: Uuid::new_v4().to_string(),
        name: payload.name,
        container_url: payload.container_url,
        browser: payload.browser,
        mode: payload.mode,
        user_agent: payload.user_agent,
        proxy: payload.proxy,
        locale: payload.locale,
        latitude: payload.latitude,
        longitude: payload.longitude,
        accuracy: payload.accuracy,
        headless: payload.headless,
        tags: payload.tags,
        folder_id: if payload.folder_id.is_empty() {
            "default".into()
        } else {
            payload.folder_id
        },
        cookies: vec![],
    };
    state.profiles.lock().await.push(profile.clone());
    ok_msg(profile, "Profile created")
}

async fn profile_get(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Profile> {
    let profiles = state.profiles.lock().await;
    let profile = profiles.iter().find(|p| p.id == id).cloned();
    match profile {
        Some(p) => ok(p),
        None => err(404, "Profile not found"),
    }
}

async fn profile_update(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(payload): Json<Value>,
) -> ApiResult<Profile> {
    let mut profiles = state.profiles.lock().await;
    let Some(idx) = profiles.iter().position(|p| p.id == id) else {
        return err(404, "Profile not found");
    };
    if let Some(v) = payload.get("name").and_then(|v| v.as_str()) {
        profiles[idx].name = v.to_owned();
    }
    if let Some(v) = payload.get("container_url").and_then(|v| v.as_str()) {
        profiles[idx].container_url = v.to_owned();
    }
    if let Some(v) = payload.get("user_agent").and_then(|v| v.as_str()) {
        profiles[idx].user_agent = Some(v.to_owned());
    }
    if let Some(v) = payload.get("proxy").and_then(|v| v.as_str()) {
        profiles[idx].proxy = Some(v.to_owned());
    }
    if let Some(v) = payload.get("locale").and_then(|v| v.as_str()) {
        profiles[idx].locale = Some(v.to_owned());
    }
    if let Some(v) = payload.get("latitude").and_then(|v| v.as_f64()) {
        profiles[idx].latitude = Some(v);
    }
    if let Some(v) = payload.get("longitude").and_then(|v| v.as_f64()) {
        profiles[idx].longitude = Some(v);
    }
    if let Some(v) = payload.get("accuracy").and_then(|v| v.as_f64()) {
        profiles[idx].accuracy = Some(v);
    }
    if let Some(v) = payload.get("headless").and_then(|v| v.as_bool()) {
        profiles[idx].headless = v;
    }
    if let Some(v) = payload.get("tags").and_then(|v| v.as_array()) {
        profiles[idx].tags = v
            .iter()
            .filter_map(|x| x.as_str().map(String::from))
            .collect();
    }
    let p = profiles[idx].clone();
    ok(p)
}

async fn profile_delete(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Value> {
    let mut profiles = state.profiles.lock().await;
    let before = profiles.len();
    profiles.retain(|p| p.id != id);
    if profiles.len() == before {
        return err(404, "Profile not found");
    }
    ok_msg(json!({ "deleted": true }), "Profile removed")
}

async fn profile_start(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> ApiResult<StartProfileData> {
    let profile = {
        let profiles = state.profiles.lock().await;
        profiles.iter().find(|p| p.id == id).cloned()
    };
    let Some(profile) = profile else {
        return err(404, "Profile not found");
    };

    let config = build_config(&profile);
    let mut sb = BaseCase::new(config).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::err(ApiStatus::err(
                "LAUNCH_FAILED",
                e.to_string(),
            ))),
        )
    })?;
    apply_profile_overrides(&mut sb, &profile)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::err(ApiStatus::err("OVERRIDE_FAILED", e))),
            )
        })?;
    if !profile.cookies.is_empty() {
        set_cookies(&mut sb, &profile.cookies).await.map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::err(ApiStatus::err("COOKIE_FAILED", e))),
            )
        })?;
    }
    if let Some(url) = params.get("url") {
        sb.open(url).await.map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::err(ApiStatus::err(
                    "OPEN_FAILED",
                    e.to_string(),
                ))),
            )
        })?;
    }

    let session_id = make_session_id();
    let info = SessionInfo {
        session_id: session_id.clone(),
        profile_id: profile.id.clone(),
        profile_name: profile.name.clone(),
        container_url: profile.container_url.clone(),
    };
    state.sessions.lock().await.insert(session_id.clone(), sb);
    state
        .session_info
        .lock()
        .await
        .insert(session_id.clone(), info);

    let port: u16 = profile
        .container_url
        .rsplit(':')
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or(4444);

    ok_msg(
        StartProfileData {
            profile_id: profile.id,
            session_id,
            port,
            ws_endpoint: profile.container_url.clone(),
            message: "Profile started".into(),
        },
        "Profile started",
    )
}

async fn profile_stop(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Value> {
    let session_id = {
        let infos = state.session_info.lock().await;
        infos
            .values()
            .find(|s| s.profile_id == id)
            .map(|s| s.session_id.clone())
    };
    let Some(session_id) = session_id else {
        return err(404, "No active session for profile");
    };
    let mut sessions = state.sessions.lock().await;
    let sb = sessions.remove(&session_id).ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::err(ApiStatus::err(
                "NOT_FOUND",
                "Session not found",
            ))),
        )
    })?;
    sb.quit().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::err(ApiStatus::err(
                "QUIT_FAILED",
                e.to_string(),
            ))),
        )
    })?;
    state.session_info.lock().await.remove(&session_id);
    ok_msg(json!({ "stopped": true }), "Profile stopped")
}

async fn profile_clone(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Profile> {
    let mut profiles = state.profiles.lock().await;
    let Some(source) = profiles.iter().find(|p| p.id == id).cloned() else {
        return err(404, "Profile not found");
    };
    let mut clone = source;
    clone.id = Uuid::new_v4().to_string();
    clone.name = format!("{} (clone)", clone.name);
    profiles.push(clone.clone());
    ok_msg(clone, "Profile cloned")
}

async fn profile_export(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Value> {
    let profiles = state.profiles.lock().await;
    let profile = profiles.iter().find(|p| p.id == id).cloned();
    match profile {
        Some(p) => ok_msg(
            serde_json::to_value(p).unwrap_or_default(),
            "Profile exported",
        ),
        None => err(404, "Profile not found"),
    }
}

async fn profile_import(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<Value>,
) -> ApiResult<Profile> {
    let profile: Profile = serde_json::from_value(payload).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::err(ApiStatus::err(
                "BAD_REQUEST",
                e.to_string(),
            ))),
        )
    })?;
    let mut profiles = state.profiles.lock().await;
    profiles.push(profile.clone());
    ok_msg(profile, "Profile imported")
}

async fn cookie_import(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CookieImportRequest>,
) -> ApiResult<Value> {
    let mut profiles = state.profiles.lock().await;
    let Some(idx) = profiles.iter().position(|p| p.id == payload.profile_id) else {
        return err(404, "Profile not found");
    };
    profiles[idx].cookies = payload.cookies.clone();
    let mut sessions = state.sessions.lock().await;
    if let Some(sb) = sessions.get_mut(&payload.profile_id) {
        set_cookies(sb, &payload.cookies).await.map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::err(ApiStatus::err("COOKIE_FAILED", e))),
            )
        })?;
    }
    ok_msg(
        json!({ "imported": payload.cookies.len() }),
        "Cookies successfully imported",
    )
}

async fn cookie_export(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CookieExportRequest>,
) -> ApiResult<Value> {
    let profiles = state.profiles.lock().await;
    let cookies = profiles
        .iter()
        .find(|p| p.id == payload.profile_id)
        .map(|p| p.cookies.clone())
        .unwrap_or_default();
    ok_msg(json!({ "cookies": cookies }), "Cookies exported")
}
fn bad_request(msg: impl Into<String>) -> ApiError {
    let status = ApiStatus::err("BAD_REQUEST", msg);
    (StatusCode::BAD_REQUEST, Json(ApiResponse::err(status)))
}

fn internal_error(msg: impl Into<String>) -> ApiError {
    let status = ApiStatus::err("INTERNAL_ERROR", msg);
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ApiResponse::err(status)),
    )
}

fn bad_gateway(msg: impl Into<String>) -> ApiError {
    let status = ApiStatus::err("BAD_GATEWAY", msg);
    (StatusCode::BAD_GATEWAY, Json(ApiResponse::err(status)))
}

async fn proxy_validate(Json(payload): Json<ProxyValidateRequest>) -> ApiResult<ProxyValidateData> {
    let proxy_url = if let (Some(u), Some(p)) = (payload.username, payload.password) {
        format!(
            "{}://{}:{}@{}:{}",
            payload.proxy_type, u, p, payload.host, payload.port
        )
    } else {
        format!("{}://{}:{}", payload.proxy_type, payload.host, payload.port)
    };

    let proxy =
        reqwest::Proxy::all(&proxy_url).map_err(|e| bad_request(format!("Invalid proxy: {e}")))?;

    let client = reqwest::Client::builder()
        .proxy(proxy)
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| internal_error(e.to_string()))?;

    let resp = client
        .get("https://ipinfo.io/json")
        .send()
        .await
        .map_err(|e| bad_gateway(e.to_string()))?;

    let data: Value = resp.json().await.map_err(|e| bad_gateway(e.to_string()))?;

    let loc = data.get("loc").and_then(|v| v.as_str()).unwrap_or("0,0");
    let mut parts = loc.split(',');
    let lat = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0.0);
    let lon = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0.0);

    ok_msg(
        ProxyValidateData {
            ip: data.get("ip").and_then(|v| v.as_str()).unwrap_or("").into(),
            country_code: data
                .get("country")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .into(),
            latitude: lat,
            longitude: lon,
            timezone: data
                .get("timezone")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .into(),
        },
        "",
    )
}

async fn tag_list(State(state): State<Arc<AppState>>) -> ApiResult<Vec<Tag>> {
    ok(state.tags.lock().await.clone())
}

async fn tag_create(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateTagRequest>,
) -> ApiResult<Tag> {
    let tag = Tag {
        id: Uuid::new_v4().to_string(),
        name: payload.name,
        color: payload.color.unwrap_or_else(|| "#396cd8".into()),
    };
    state.tags.lock().await.push(tag.clone());
    ok_msg(tag, "Tag created")
}

async fn tag_update(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(payload): Json<Value>,
) -> ApiResult<Tag> {
    let mut tags = state.tags.lock().await;
    let Some(tag) = tags.iter_mut().find(|t| t.id == id) else {
        return err(404, "Tag not found");
    };
    if let Some(v) = payload.get("name").and_then(|v| v.as_str()) {
        tag.name = v.to_owned();
    }
    if let Some(v) = payload.get("color").and_then(|v| v.as_str()) {
        tag.color = v.to_owned();
    }
    ok(tag.clone())
}

async fn tag_delete(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Value> {
    let mut tags = state.tags.lock().await;
    let before = tags.len();
    tags.retain(|t| t.id != id);
    if tags.len() == before {
        return err(404, "Tag not found");
    }
    ok_msg(json!({ "deleted": true }), "Tag removed")
}

async fn folder_list(State(state): State<Arc<AppState>>) -> ApiResult<Vec<Folder>> {
    ok(state.folders.lock().await.clone())
}

async fn folder_create(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateFolderRequest>,
) -> ApiResult<Folder> {
    let folder = Folder {
        id: Uuid::new_v4().to_string(),
        name: payload.name,
    };
    state.folders.lock().await.push(folder.clone());
    ok_msg(folder, "Folder created")
}

async fn folder_update(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(payload): Json<Value>,
) -> ApiResult<Folder> {
    let mut folders = state.folders.lock().await;
    let Some(folder) = folders.iter_mut().find(|f| f.id == id) else {
        return err(404, "Folder not found");
    };
    if let Some(v) = payload.get("name").and_then(|v| v.as_str()) {
        folder.name = v.to_owned();
    }
    ok(folder.clone())
}

async fn folder_delete(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Value> {
    let mut folders = state.folders.lock().await;
    let before = folders.len();
    folders.retain(|f| f.id != id);
    if folders.len() == before {
        return err(404, "Folder not found");
    }
    ok_msg(json!({ "deleted": true }), "Folder removed")
}

async fn screen_resolution() -> ApiResult<Value> {
    ok_msg(
        json!({ "resolutions": ["1920x1080", "1366x768", "1440x900", "1536x864", "1280x720"] }),
        "",
    )
}

async fn script_runner_start(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RunScriptRequest>,
) -> ApiResult<Value> {
    let mut results = Vec::new();
    for profile_id in &payload.profile_ids {
        let profile = {
            let profiles = state.profiles.lock().await;
            profiles.iter().find(|p| p.id == *profile_id).cloned()
        };
        let Some(profile) = profile else { continue };
        let config = build_config(&profile);
        let sb = match BaseCase::new(config).await {
            Ok(sb) => sb,
            Err(e) => {
                results.push(json!({ "profile_id": profile_id, "error": e.to_string() }));
                continue;
            }
        };
        let result = sb
            .execute_script(&payload.script)
            .await
            .map(|v| v.to_string())
            .unwrap_or_else(|e| e.to_string());
        results.push(json!({ "profile_id": profile_id, "result": result }));
        let _ = sb.quit().await;
    }
    ok_msg(json!({ "results": results }), "Script runner started")
}

async fn script_runner_stop() -> ApiResult<Value> {
    ok_msg(json!({ "stopped": true }), "Script runner stopped")
}

async fn browser_core_list() -> ApiResult<Value> {
    ok_msg(
        json!({ "cores": ["chrome-120", "chrome-121", "chrome-122"] }),
        "",
    )
}

async fn load_browser_core() -> ApiResult<Value> {
    ok_msg(json!({ "message": "Download started" }), "")
}

async fn delete_browser_core() -> ApiResult<Value> {
    ok_msg(json!({ "message": "" }), "")
}

async fn stop_all(State(state): State<Arc<AppState>>) -> ApiResult<Value> {
    let ids: Vec<String> = state.sessions.lock().await.keys().cloned().collect();
    for id in ids {
        if let Some(sb) = state.sessions.lock().await.remove(&id) {
            let _ = sb.quit().await;
        }
        state.session_info.lock().await.remove(&id);
    }
    ok_msg(json!({ "stopped_all": true }), "All profiles stopped")
}

async fn workspaces() -> ApiResult<Value> {
    ok_msg(
        json!({ "workspaces": [{ "id": "default", "name": "Default Workspace" }] }),
        "",
    )
}

async fn user_signin() -> ApiResult<Value> {
    ok_msg(json!({ "token": "dummy-token", "expires_in": 1800 }), "")
}

async fn user_refresh_token() -> ApiResult<Value> {
    ok_msg(json!({ "token": "dummy-token", "expires_in": 1800 }), "")
}

async fn bookmarks_export() -> ApiResult<Value> {
    ok_msg(json!({ "bookmarks": [] }), "")
}

async fn bookmarks_import() -> ApiResult<Value> {
    ok_msg(json!({ "imported": 0 }), "")
}

async fn twofa_setup() -> ApiResult<Value> {
    ok_msg(json!({ "secret": "DUMMYSECRET", "qr": "" }), "")
}

async fn twofa_enable() -> ApiResult<Value> {
    ok_msg(json!({ "enabled": true }), "")
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::{to_bytes, Body};
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    fn test_state() -> Arc<AppState> {
        Arc::new(AppState::new())
    }

    async fn read_json(res: axum::response::Response<Body>) -> Value {
        let bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
        serde_json::from_slice(&bytes).unwrap()
    }

    #[tokio::test]
    async fn version_endpoint() {
        let app = router(test_state());
        let res = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/version")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body = read_json(res).await;
        assert_eq!(body["status"]["error_code"].as_str(), Some(""));
    }

    #[tokio::test]
    async fn profile_crud() {
        let app = router(test_state());

        let create = app.clone().oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/profiles")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"name":"Test","container_url":"http://localhost:4444","tags":["tag1"]}"#))
                .unwrap(),
        ).await.unwrap();
        assert_eq!(create.status(), StatusCode::OK);
        let body = read_json(create).await;
        let id = body["data"]["id"].as_str().unwrap().to_string();

        let list = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/profiles")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let list_body = read_json(list).await;
        assert_eq!(list_body["data"].as_array().unwrap().len(), 1);

        let get = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/profiles/{id}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(get.status(), StatusCode::OK);

        let update = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/profiles/{id}"))
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"name":"Updated"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(update.status(), StatusCode::OK);

        let delete = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!("/api/v1/profiles/{id}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(delete.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn tags_and_folders() {
        let app = router(test_state());

        let tag = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/tags")
                    .header("content-type", "application/json")
                    .body(Body::from(r##"{"name":"Work","color":"#ff0000"}"##))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(tag.status(), StatusCode::OK);

        let folder = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/folders")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"name":"Clients"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(folder.status(), StatusCode::OK);

        let list = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tags")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let body = read_json(list).await;
        assert!(!body["data"].as_array().unwrap().is_empty());
    }
}
