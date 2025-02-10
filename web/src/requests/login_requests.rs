use crate::components::gen_components::AdminSetupData;
use crate::components::gen_funcs::encode_password;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};

use yew_router::history::{BrowserHistory, History};
use yewdux::Dispatch;
// Add imports for your context modules
use crate::components::context::AppState;
use anyhow::{Context, Error};

#[derive(Serialize)]
pub struct LoginRequest {
    username: String,
    password: String,
    // api_key: String
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct LoginServerRequest {
    pub(crate) server_name: String,
    pub(crate) username: String,
    pub(crate) password: String,
    pub(crate) api_key: Option<String>,
}

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct LoginResponse {
    status: String,
    retrieved_key: String,
}

#[derive(Deserialize)]
pub struct PinepodsCheckResponse {
    pinepods_instance: Option<bool>,
}

pub async fn verify_pinepods_instance(
    server_name: &str,
) -> Result<PinepodsCheckResponse, anyhow::Error> {
    let url = format!("{}/api/pinepods_check", server_name);
    let response = Request::get(&url).send().await?;

    if response.ok() {
        let check_data: PinepodsCheckResponse = response.json().await?;
        if check_data.pinepods_instance.unwrap_or(false) {
            Ok(check_data)
        } else {
            Err(anyhow::Error::msg("Pinepods instance not found"))
        }
    } else {
        Err(anyhow::Error::msg("Failed to verify Pinepods instance"))
    }
}

#[derive(Deserialize, Debug)]
pub struct KeyVerification {
    // Add fields according to your API's JSON response
    pub status: String,
}

pub async fn call_verify_key(
    server_name: &str,
    api_key: &str,
) -> Result<crate::requests::login_requests::KeyVerification, anyhow::Error> {
    let url = format!("{}/api/data/verify_key", server_name);

    let response = Request::get(&url).header("Api-Key", api_key).send().await?;

    if response.ok() {
        let key_verify: crate::requests::login_requests::KeyVerification = response.json().await?;
        Ok(key_verify)
    } else {
        Err(anyhow::Error::msg("Failed to get user data"))
    }
}

#[derive(Deserialize, Debug)]
pub struct GetUserIdResponse {
    // Add fields according to your API's JSON response
    pub status: String,
    pub retrieved_id: Option<i32>,
}

pub async fn call_get_user_id(
    server_name: &str,
    api_key: &str,
) -> Result<GetUserIdResponse, anyhow::Error> {
    let url = format!("{}/api/data/get_user", server_name);

    let response = Request::get(&url).header("Api-Key", api_key).send().await?;

    if response.ok() {
        let user_id_data: GetUserIdResponse = response.json().await?;
        Ok(user_id_data)
    } else {
        Err(anyhow::Error::msg("Failed to get user ID"))
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
#[allow(non_snake_case)]
pub struct GetUserDetails {
    pub UserID: i32,
    pub Fullname: Option<String>,
    pub Username: Option<String>,
    pub Email: Option<String>,
    pub Hashed_PW: Option<String>,
    pub Salt: Option<String>,
    pub PreferredLanguage: Option<String>,
}

pub async fn call_get_user_details(
    server_name: &str,
    api_key: &str,
    user_id: &i32,
) -> Result<crate::requests::login_requests::GetUserDetails, anyhow::Error> {
    let url = format!("{}/api/data/user_details_id/{}", server_name, user_id);

    let response = Request::get(&url).header("Api-Key", api_key).send().await?;

    if response.ok() {
        let body_str = response.text().await?;

        let user_data: crate::requests::login_requests::GetUserDetails =
            serde_json::from_str(&body_str)?;
        Ok(user_data)
    } else {
        Err(anyhow::Error::msg("Failed to get user information"))
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct GetApiDetails {
    // Add fields according to your API's JSON response
    pub api_url: Option<String>,
    pub proxy_url: Option<String>,
    pub proxy_host: Option<String>,
    pub proxy_port: Option<String>,
    pub proxy_protocol: Option<String>,
    pub reverse_proxy: Option<String>,
    pub people_url: Option<String>,
}

pub async fn call_get_api_config(
    server_name: &str,
    api_key: &str,
) -> Result<crate::requests::login_requests::GetApiDetails, anyhow::Error> {
    let url = format!("{}/api/data/config", server_name);

    let response = Request::get(&url).header("Api-Key", api_key).send().await?;

    if response.ok() {
        let server_data: GetApiDetails = response.json().await?;
        Ok(server_data)
    } else {
        Err(anyhow::Error::msg("Failed to get user information"))
    }
}

pub async fn login_new_server(
    server_name: String,
    username: String,
    password: String,
) -> Result<(GetUserDetails, LoginServerRequest, GetApiDetails), anyhow::Error> {
    let credentials = STANDARD.encode(format!("{}:{}", username, password).as_bytes());
    let auth_header = format!("Basic {}", credentials);
    let url = format!("{}/api/data/get_key", server_name);

    // Step 1: Verify Server
    match verify_pinepods_instance(&server_name).await {
        Ok(check_data) => {
            if !check_data.pinepods_instance.unwrap_or(false) {
                return Err(anyhow::Error::msg(
                    "Pinepods instance not found at specified server",
                ));
            }
            // Step 2: Get API key
            let response = Request::get(&url)
                .header("Authorization", &auth_header)
                .send()
                .await?;

            if !response.ok() {
                return Err(anyhow::Error::msg(
                    "Failed to authenticate user. Incorrect credentials?",
                ));
            }

            let api_key = response.json::<LoginResponse>().await?.retrieved_key;

            // Step 2: Verify the API key
            let verify_response = call_verify_key(&server_name, &api_key).await?;
            if verify_response.status != "success" {
                return Err(anyhow::Error::msg("API key verification failed"));
            }

            // Step 3: Get user ID
            let user_id_response = call_get_user_id(&server_name, &api_key).await?;
            if user_id_response.status != "success" {
                return Err(anyhow::Error::msg("Failed to get user ID"));
            }

            let login_request = LoginServerRequest {
                server_name: server_name.clone(),
                username: username.clone(),
                password: password.clone(),
                api_key: Some(api_key.clone()), // or None, depending on the context
            };

            // Step 4: Get user details
            let user_details = call_get_user_details(
                &server_name,
                &api_key,
                &user_id_response.retrieved_id.unwrap(),
            )
            .await?;
            if user_details.Username.is_none() {
                return Err(anyhow::Error::msg("Failed to get user details"));
            }

            // Step 5: Get server details
            let server_details = call_get_api_config(&server_name, &api_key).await?;
            if server_details.api_url.is_none() {
                return Err(anyhow::Error::msg("Failed to get server details"));
            }

            Ok((user_details, login_request, server_details))
        }
        Err(e) => {
            // Directly propagate the error from verify_pinepods_instance
            return Err(e);
        }
    }
}

pub(crate) fn use_check_authentication(_dispatch: Dispatch<AppState>, current_route: &str) {
    let window = web_sys::window().expect("no global `window` exists");
    let session_storage = window.session_storage().unwrap().unwrap();
    let is_authenticated = session_storage.get_item("isAuthenticated").unwrap_or(None);
    session_storage
        .set_item("requested_route", &current_route)
        .unwrap();
    // If not authenticated or no information, redirect to login
    if is_authenticated != Some("true".to_string()) {
        session_storage
            .set_item("isAuthenticated", "false")
            .unwrap();
        let history = BrowserHistory::new();
        // let last_known_route = session_storage.get_item("requested_route").unwrap_or(Some(current_route.to_string())).unwrap_or_default();
        history.push("/");
    } else {
        // Already authenticated, continue as normal
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub struct AddUserRequest {
    pub(crate) fullname: String,
    pub(crate) username: String,
    pub(crate) email: String,
    pub(crate) hash_pw: String,
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct AddUserResponse {
    detail: String,
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct UserErrorResponse {
    pub detail: String,
}

#[allow(dead_code)]
pub async fn call_add_login_user(
    server_name: String,
    add_user: &Option<AddUserRequest>,
) -> Result<bool, Error> {
    let server = server_name.clone();
    let url = format!("{}/api/data/add_login_user", server);
    let add_user_req = add_user.as_ref().unwrap();

    // Serialize `add_user` into JSON
    let json_body = serde_json::to_string(&add_user_req)?;
    let response = Request::post(&url)
        .header("Content-Type", "application/json")
        .body(json_body)?
        .send()
        .await?;

    if response.ok() {
        Ok(true)
    } else {
        // Try to get the detailed error message from the response
        let error_text = response.text().await?;

        // Attempt to parse the error response as JSON
        match serde_json::from_str::<UserErrorResponse>(&error_text) {
            Ok(error_response) => {
                // Return the detailed error message
                Err(Error::msg(error_response.detail))
            }
            Err(_) => {
                // If we can't parse the error response, return a more user-friendly message
                if error_text.contains("duplicate key value") && error_text.contains("username") {
                    Err(Error::msg(
                        "This username is already taken. Please choose a different username.",
                    ))
                } else if error_text.contains("duplicate key value") && error_text.contains("email")
                {
                    Err(Error::msg(
                        "This email is already registered. Please use a different email address.",
                    ))
                } else {
                    Err(Error::msg(format!(
                        "Unable to create user account. Please try again or contact support if the problem persists. Error: {}",
                        error_text
                    )))
                }
            }
        }
    }
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
struct FirstLoginResponse {
    FirstLogin: bool,
}

pub async fn call_first_login_done(
    server_name: String,
    api_key: String,
    user_id: &i32,
) -> Result<bool, Error> {
    let url = format!("{}/api/data/first_login_done/{}", server_name, user_id);

    let response = Request::get(&url)
        .header("Content-Type", "application/json")
        .header("Api-Key", &api_key)
        .send()
        .await?;

    if response.ok() {
        let response_body = response.json::<FirstLoginResponse>().await?;
        Ok(response_body.FirstLogin) // Use the struct field to get the boolean value
    } else {
        Err(Error::msg(format!(
            "Error checking first login status: {}",
            response.status_text()
        )))
    }
}

#[derive(Serialize, Debug, Deserialize, PartialEq, Clone)]
pub struct TimeZoneInfo {
    pub user_id: i32,
    pub timezone: String,
    pub hour_pref: i32,
    pub date_format: String,
    pub preferred_language: String,
}

#[derive(Deserialize, Debug)]
pub struct SetupTimeZoneInfoResponse {
    pub success: bool,
}

pub async fn call_setup_timezone_info(
    server_name: String,
    api_key: String,
    time_zone_info: TimeZoneInfo,
) -> Result<SetupTimeZoneInfoResponse, anyhow::Error> {
    let url = format!("{}/api/data/setup_time_info", server_name);
    let request_body = serde_json::to_string(&time_zone_info).context("Serialization Error")?;

    let response = Request::post(&url)
        .header("Content-Type", "application/json")
        .header("Api-Key", api_key.as_str())
        .body(request_body)
        .context("Request Building Error")?
        .send()
        .await
        .context("Network Request Error")?;

    if response.ok() {
        response
            .json::<SetupTimeZoneInfoResponse>()
            .await
            .context("Response Parsing Error")
    } else {
        Err(anyhow::anyhow!(
            "Error setting up time info. Server Response: {}",
            response.status_text()
        ))
    }
}

#[derive(Deserialize, Debug)]
pub struct TimeInfoResponse {
    pub timezone: String,
    pub hour_pref: i16,
    pub date_format: String,
}

pub async fn call_get_time_info(
    server_name: String,
    api_key: String,
    user_id: &i32,
) -> Result<TimeInfoResponse, anyhow::Error> {
    let url = format!("{}/api/data/get_time_info?user_id={}", server_name, user_id);

    let response = Request::get(&url)
        .header("Content-Type", "application/json")
        .header("Api-Key", &api_key)
        .send()
        .await
        .context("Network Request Error")?;

    if response.ok() {
        response
            .json::<TimeInfoResponse>()
            .await
            .context("Response Parsing Error")
    } else {
        Err(anyhow::anyhow!(
            "Error fetching time info. Server Response: {}",
            response.status_text()
        ))
    }
}

#[derive(Deserialize, Debug)]
pub struct CheckMfaEnabledResponse {
    pub(crate) mfa_enabled: bool,
}

pub async fn call_check_mfa_enabled(
    server_name: String,
    api_key: String,
    user_id: &i32,
) -> Result<CheckMfaEnabledResponse, Error> {
    let url = format!("{}/api/data/check_mfa_enabled/{}", server_name, user_id);

    let response = Request::get(&url)
        .header("Content-Type", "application/json")
        .header("Api-Key", api_key.as_str())
        .send()
        .await
        .map_err(|e| Error::msg(format!("Network Request Error: {}", e)))?;

    if response.ok() {
        response
            .json::<CheckMfaEnabledResponse>()
            .await
            .map_err(|e| Error::msg(format!("Response Parsing Error: {}", e)))
    } else {
        let status_text = response.status_text();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| String::from("Failed to read error message"));
        Err(Error::msg(format!(
            "Error checking MFA enabled status: {} - {}",
            status_text, error_text
        )))
    }
}

#[derive(Serialize)]
pub struct VerifyMFABody {
    pub(crate) user_id: i32,
    pub(crate) mfa_code: String,
}

#[derive(Deserialize, Debug)]
pub struct VerifyMFAResponse {
    pub(crate) verified: bool,
}

pub async fn call_verify_mfa(
    server_name: &String,
    api_key: &String,
    user_id: i32,
    mfa_code: String,
) -> Result<VerifyMFAResponse, Error> {
    let url = format!("{}/api/data/verify_mfa", server_name);
    let body = VerifyMFABody { user_id, mfa_code };
    let request_body = serde_json::to_string(&body)?;

    let response = Request::post(&url)
        .header("Content-Type", "application/json")
        .header("Api-Key", api_key)
        .body(&request_body)?
        .send()
        .await?;

    if response.ok() {
        let response_body = response.json::<VerifyMFAResponse>().await?;
        Ok(response_body)
    } else {
        Err(anyhow::Error::msg(format!(
            "Error verifying MFA: {}",
            response.status_text()
        )))
    }
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct SelfServiceStatusResponse {
    pub status: bool,
    pub first_admin_created: bool,
}

pub async fn call_self_service_login_status(server_name: String) -> Result<(bool, bool), Error> {
    let server_name = server_name.trim_end_matches('/');
    let url = format!("{}/api/data/self_service_status", server_name);
    web_sys::console::log_1(&format!("Requesting URL: {}", url).into()); // Add logging
    let response = Request::get(&url)
        .send()
        .await
        .map_err(|e| Error::msg(format!("Network error: {}", e)))?;

    if response.ok() {
        let status_response: SelfServiceStatusResponse = response
            .json()
            .await
            .map_err(|e| Error::msg(format!("Error parsing JSON: {}", e)))?;

        Ok((status_response.status, status_response.first_admin_created))
    } else {
        Err(Error::msg(format!(
            "Error fetching self service status: {}",
            response.status_text()
        )))
    }
}

#[derive(Serialize, Debug)]
pub struct CreateFirstAdminRequest {
    pub username: String,
    pub password: String,
    pub email: String,
    pub fullname: String,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct CreateFirstAdminResponse {
    pub message: String,
    pub user_id: i32,
}

pub async fn call_create_first_admin(
    server_name: &str,
    request: AdminSetupData,
) -> Result<(), Error> {
    // Hash the password first
    let hashed_password = encode_password(&request.password)
        .map_err(|e| Error::msg(format!("Failed to hash password: {}", e)))?;

    // Create the request body with hashed password
    let api_request = CreateFirstAdminRequest {
        username: request.username,
        password: hashed_password, // Send the hashed password
        email: request.email,
        fullname: request.fullname,
    };

    let url = format!("{}/api/data/create_first", server_name);

    let response = Request::post(&url)
        .json(&api_request)?
        .send()
        .await
        .map_err(|e| Error::msg(format!("Network error: {}", e)))?;

    if response.ok() {
        Ok(())
    } else {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        Err(Error::msg(format!("Error creating admin: {}", error_text)))
    }
}

#[derive(Serialize)]
pub struct ResetCodePayload {
    pub(crate) email: String,
    pub(crate) username: String,
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct ResetCodeResponse {
    pub code_created: bool,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ErrorResponse {
    pub detail: String,
}

#[allow(dead_code)]
pub async fn call_reset_password_create_code(
    server_name: String,
    create_code: &ResetCodePayload,
) -> Result<bool, Error> {
    let url = format!("{}/api/data/reset_password_create_code", server_name);

    let json_body = serde_json::to_string(&create_code)?;

    let response = Request::post(&url)
        .header("Content-Type", "application/json")
        .body(json_body)?
        .send()
        .await?;

    if response.ok() {
        let response_body = response.json::<ResetCodeResponse>().await?;
        Ok(response_body.code_created)
    } else {
        let error_response: Result<ErrorResponse, _> = response.json().await;
        match error_response {
            Ok(err) => Err(Error::msg(err.detail)),
            Err(_) => {
                // If parsing the detailed error fails, fall back to the status text
                let status_text = response.status_text();
                Err(Error::msg(format!(
                    "Error creating reset code: {}",
                    status_text
                )))
            }
        }
    }
}

#[derive(Serialize)]
pub struct ResetForgotPasswordPayload {
    pub(crate) reset_code: String,
    pub(crate) email: String,
    pub(crate) new_password: String,
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct ForgotResetPasswordResponse {
    pub message: String,
}

#[allow(dead_code)]
pub async fn call_verify_and_reset_password(
    server_name: String,
    verify_and_reset: &ResetForgotPasswordPayload,
) -> Result<ForgotResetPasswordResponse, Error> {
    let url = format!("{}/api/data/verify_and_reset_password", server_name);

    let json_body = serde_json::to_string(&verify_and_reset)?;

    let response = Request::post(&url)
        .header("Content-Type", "application/json")
        .body(json_body)?
        .send()
        .await?;

    if response.ok() {
        let response_body = response.json::<ForgotResetPasswordResponse>().await?;
        Ok(response_body)
    } else {
        Err(Error::msg(format!(
            "Error creating reset code: {}",
            response.status_text()
        )))
    }
}
