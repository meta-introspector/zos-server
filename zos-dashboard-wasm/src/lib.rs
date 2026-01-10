use wasm_bindgen::prelude::*;
use web_sys::{console, window, Document, Element, HtmlElement};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Service {
    name: String,
    icon: String,
    environment: String,
    port: u16,
    hostname: String,
    url: String,
    status: bool,
}

#[derive(Serialize, Deserialize)]
struct ServicesResponse {
    services: Vec<Service>,
    network_status: std::collections::HashMap<String, bool>,
}

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn init_dashboard() {
    console::log_1(&"🚀 ZOS Dashboard WASM initialized".into());

    // Set up event listeners
    if let Some(window) = window() {
        if let Some(document) = window.document() {
            setup_refresh_button(&document);
            setup_deploy_buttons(&document);

            // Initial refresh
            refresh_services();
        }
    }
}

fn setup_refresh_button(document: &Document) {
    if let Some(button) = document.get_element_by_id("refresh-btn") {
        let closure = Closure::wrap(Box::new(move || {
            refresh_services();
        }) as Box<dyn FnMut()>);

        button.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref()).unwrap();
        closure.forget();
    }
}

fn setup_deploy_buttons(document: &Document) {
    // QA Deploy button
    if let Some(button) = document.get_element_by_id("deploy-qa-btn") {
        let closure = Closure::wrap(Box::new(move || {
            deploy_qa();
        }) as Box<dyn FnMut()>);

        button.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref()).unwrap();
        closure.forget();
    }

    // Prod Deploy button
    if let Some(button) = document.get_element_by_id("deploy-prod-btn") {
        let closure = Closure::wrap(Box::new(move || {
            deploy_prod();
        }) as Box<dyn FnMut()>);

        button.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref()).unwrap();
        closure.forget();
    }
}

#[wasm_bindgen]
pub fn refresh_services() {
    console::log_1(&"🔍 Refreshing services via WASM...".into());

    wasm_bindgen_futures::spawn_local(async {
        match fetch_services().await {
            Ok(services) => {
                console::log_1(&format!("✅ Services fetched: {:?}", services.services.len()).into());
                update_services_display(&services);
            }
            Err(e) => {
                console::log_1(&format!("❌ Failed to fetch services: {:?}", e).into());
                log_to_activity(&format!("❌ Failed to refresh services: {}", e));
            }
        }
    });
}

async fn fetch_services() -> Result<ServicesResponse, JsValue> {
    let window = window().unwrap();
    let resp_value = wasm_bindgen_futures::JsFuture::from(
        window.fetch_with_str("/api/dashboard/services")
    ).await?;

    let resp: web_sys::Response = resp_value.dyn_into().unwrap();
    let json = wasm_bindgen_futures::JsFuture::from(resp.json()?).await?;

    let services: ServicesResponse = serde_wasm_bindgen::from_value(json)?;
    Ok(services)
}

fn update_services_display(services: &ServicesResponse) {
    if let Some(window) = window() {
        if let Some(document) = window.document() {
            if let Some(services_div) = document.get_element_by_id("services") {
                let html = format_services(services);
                services_div.set_inner_html(&html);
                log_to_activity("✅ Services refreshed via WASM");
            }
        }
    }
}

fn format_services(data: &ServicesResponse) -> String {
    let mut html = String::new();

    for service in &data.services {
        let status_class = if service.status { "healthy" } else { "unhealthy" };
        let status_text = if service.status { "✅ Healthy" } else { "❌ Down" };

        html.push_str(&format!(
            r#"<div class="status {}">
                {} {} ({})
                <br><small>{}:{}</small>
                <br><strong>{}</strong>
            </div>"#,
            status_class,
            service.icon,
            service.name,
            service.environment.to_uppercase(),
            service.hostname,
            service.port,
            status_text
        ));
    }

    html
}

#[wasm_bindgen]
pub fn deploy_qa() {
    console::log_1(&"🧪 Deploying QA via WASM...".into());
    log_to_activity("🧪 Starting QA deployment...");

    wasm_bindgen_futures::spawn_local(async {
        match deploy_service("qa", 8082).await {
            Ok(_) => {
                console::log_1(&"✅ QA deployment successful".into());
                log_to_activity("✅ QA deployment completed");
                refresh_services();
            }
            Err(e) => {
                console::log_1(&format!("❌ QA deployment failed: {:?}", e).into());
                log_to_activity(&format!("❌ QA deployment failed: {}", e));
            }
        }
    });
}

#[wasm_bindgen]
pub fn deploy_prod() {
    console::log_1(&"🏭 Deploying Production via WASM...".into());
    log_to_activity("🏭 Starting Production deployment...");

    wasm_bindgen_futures::spawn_local(async {
        match deploy_service("prod", 8081).await {
            Ok(_) => {
                console::log_1(&"✅ Production deployment successful".into());
                log_to_activity("✅ Production deployment completed");
                refresh_services();
            }
            Err(e) => {
                console::log_1(&format!("❌ Production deployment failed: {:?}", e).into());
                log_to_activity(&format!("❌ Production deployment failed: {}", e));
            }
        }
    });
}

async fn deploy_service(env: &str, port: u16) -> Result<JsValue, JsValue> {
    let window = window().unwrap();

    let mut opts = web_sys::RequestInit::new();
    opts.method("POST");
    opts.mode(web_sys::RequestMode::Cors);

    let headers = web_sys::Headers::new()?;
    headers.set("Content-Type", "application/json")?;
    opts.headers(&headers);

    let body = format!(r#"{{"env": "{}", "port": {}, "git_hash": "current"}}"#, env, port);
    opts.body(Some(&JsValue::from_str(&body)));

    let request = web_sys::Request::new_with_str_and_init("/api/dashboard/deploy", &opts)?;

    let resp_value = wasm_bindgen_futures::JsFuture::from(
        window.fetch_with_request(&request)
    ).await?;

    let resp: web_sys::Response = resp_value.dyn_into().unwrap();
    let json = wasm_bindgen_futures::JsFuture::from(resp.json()?).await?;

    Ok(json)
}

fn log_to_activity(message: &str) {
    if let Some(window) = window() {
        if let Some(document) = window.document() {
            if let Some(log_div) = document.get_element_by_id("activity-log") {
                let timestamp = js_sys::Date::new_0().to_locale_time_string("en-US", &JsValue::UNDEFINED);
                let current_html = log_div.inner_html();
                let new_entry = format!("[{}] {}\n", timestamp.as_string().unwrap_or_default(), message);
                log_div.set_inner_html(&format!("{}{}", current_html, new_entry));
                log_div.set_scroll_top(log_div.scroll_height());
            }
        }
    }
}
