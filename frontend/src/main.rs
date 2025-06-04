use gloo::console::log;
use gloo::net::http::Request;
use js_sys::Date;
use shared::api::{ApiEndpoints, get_api_base_url};
use shared::models::Message;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use web_sys::wasm_bindgen::JsCast;
use yew::prelude::*;

// Reusable message service function
fn send_message_to_backend(message: Message) {
    let api = ApiEndpoints::new(get_api_base_url());
    spawn_local(async move {
        let client = Request::post(&api.messages_endpoint())
            .json(&message)
            .expect("Failed to serialize message");

        match client.send().await {
            Ok(_) => log!("Message sent to backend successfully"),
            Err(err) => log!(format!("Failed to send message to backend: {:?}", err)),
        }
    });
}

#[function_component]
fn App() -> Html {
    // State for storing messages
    let messages = use_state(Vec::new);

    // State for the current input
    let current_input = use_state(|| String::new());

    // Handler for input changes
    let on_input_change = {
        let current_input = current_input.clone();
        Callback::from(move |e: InputEvent| {
            let target = e.target();
            let input = target.and_then(|t| t.dyn_into::<HtmlInputElement>().ok());

            if let Some(input) = input {
                current_input.set(input.value());
            }
        })
    };

    // Handler for form submission
    let on_submit = {
        let messages = messages.clone();
        let current_input = current_input.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            if current_input.is_empty() {
                return;
            }

            // Create a new message
            let new_message = Message {
                content: (*current_input).clone(),
                timestamp: get_current_time(),
            };

            // Update local state
            let mut updated_messages = (*messages).clone();
            updated_messages.push(new_message.clone());
            messages.set(updated_messages);

            // Send to backend using the service function
            send_message_to_backend(new_message);

            // Clear input
            current_input.set(String::new());
        })
    };

    html! {
        <div style="display: flex; flex-direction: column; height: 100vh; max-width: 800px; margin: 0 auto; font-family: system-ui, -apple-system, sans-serif;">
            <div style="flex: 1; overflow-y: auto; padding: 20px; background-color: #f5f5f5;">
                {
                    (*messages).iter().map(|message| {
                        html! {
                            <div style="margin-bottom: 15px; padding: 15px; background-color: white; border-radius: 8px; box-shadow: 0 1px 3px rgba(0,0,0,0.1);">
                                <div style="font-size: 0.8rem; color: #666; margin-bottom: 5px;">{&message.timestamp}</div>
                                <div style="font-size: 1rem; line-height: 1.5; white-space: pre-wrap; word-break: break-word;">{&message.content}</div>
                            </div>
                        }
                    }).collect::<Html>()
                }
            </div>
            <div style="padding: 20px; background-color: #ffffff; border-top: 1px solid #e0e0e0;">
                <form onsubmit={on_submit} style="display: flex; gap: 10px;">
                    <input
                        type="text"
                        value={(*current_input).clone()}
                        oninput={on_input_change}
                        placeholder="Type your message here..."
                        style="flex: 1; padding: 10px; border: 1px solid #ddd; border-radius: 4px; font-size: 1rem;"
                    />
                    <button type="submit" style="padding: 10px 20px; background-color: #2563eb; color: white; border: none; border-radius: 4px; cursor: pointer; font-size: 1rem;">{"Submit"}</button>
                </form>
            </div>
        </div>
    }
}

// Helper function to get current time
fn get_current_time() -> String {
    let date = Date::new_0();
    format!(
        "{:02}:{:02}:{:02}",
        date.get_hours(),
        date.get_minutes(),
        date.get_seconds()
    )
}

fn main() {
    yew::Renderer::<App>::new().render();
}
