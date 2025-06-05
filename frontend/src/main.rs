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
fn send_message_to_backend(message: Message, callback: Callback<Message>) {
    let api = ApiEndpoints::new(get_api_base_url());
    spawn_local(async move {
        let client = Request::post(&api.messages_endpoint())
            .json(&message)
            .expect("Failed to serialize message");

        match client.send().await {
            Ok(response) => {
                log!("Message sent to backend successfully");

                // Parse the response
                match response.json::<Message>().await {
                    Ok(response_message) => {
                        log!(format!("Received response: {}", response_message.content));
                        println!("Backend response: {}", response_message.content); // Print to stdout
                        callback.emit(response_message);
                    }
                    Err(err) => {
                        log!(format!("Failed to parse response: {:?}", err));
                    }
                }
            }
            Err(err) => {
                log!(format!("Failed to send message to backend: {:?}", err));
            }
        }
    });
}

#[function_component]
fn App() -> Html {
    // State for storing messages
    let messages = use_state(Vec::new);

    // State for the current input
    let current_input = use_state(|| String::new());

    // State to track if we're waiting for a response
    let waiting_for_response = use_state(|| false);

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

    // Callback for when we receive a response from the backend
    let on_response_received = {
        let messages = messages.clone();
        let waiting_for_response = waiting_for_response.clone();
        Callback::from(move |response_message: Message| {
            // Add the response message to our messages
            let mut updated_messages = (*messages).clone();
            updated_messages.push(response_message);
            messages.set(updated_messages);

            // No longer waiting for response
            waiting_for_response.set(false);
        })
    };

    // Handler for form submission
    let on_submit = {
        let messages = messages.clone();
        let current_input = current_input.clone();
        let waiting_for_response = waiting_for_response.clone();
        let on_response_received = on_response_received.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            if current_input.is_empty() || *waiting_for_response {
                return;
            }

            // Create a new message
            let new_message = Message {
                content: (*current_input).clone(),
                timestamp: get_current_time(),
            };

            // Update local state with user message
            let mut updated_messages = (*messages).clone();
            updated_messages.push(new_message.clone());
            messages.set(updated_messages);

            // Set waiting state
            waiting_for_response.set(true);

            // Send to backend using the service function
            send_message_to_backend(new_message, on_response_received.clone());

            // Clear input
            current_input.set(String::new());
        })
    };

    html! {
        <div style="display: flex; flex-direction: column; height: 100vh; max-width: 800px; margin: 0 auto; font-family: system-ui, -apple-system, sans-serif;">
            <div style="flex: 1; overflow-y: auto; padding: 20px; background-color: #f5f5f5;">
                {
                    (*messages).iter().enumerate().map(|(index, message)| {
                        let is_user_message = index % 2 == 0;
                        let bg_color = if is_user_message { "#e3f2fd" } else { "#f1f8e9" };
                        let label = if is_user_message { "You:" } else { "Assistant:" };

                        html! {
                            <div style={format!("margin-bottom: 15px; padding: 15px; background-color: {}; border-radius: 8px; box-shadow: 0 1px 3px rgba(0,0,0,0.1);", bg_color)}>
                                <div style="font-size: 0.8rem; color: #666; margin-bottom: 5px; font-weight: bold;">{format!("{} {}", label, &message.timestamp)}</div>
                                <div style="font-size: 1rem; line-height: 1.5; white-space: pre-wrap; word-break: break-word;">{&message.content}</div>
                            </div>
                        }
                    }).collect::<Html>()
                }
                {
                    if *waiting_for_response {
                        html! {
                            <div style="margin-bottom: 15px; padding: 15px; background-color: #fff3e0; border-radius: 8px; box-shadow: 0 1px 3px rgba(0,0,0,0.1);">
                                <div style="font-size: 0.8rem; color: #666; margin-bottom: 5px; font-weight: bold;">{"Assistant: typing..."}</div>
                                <div style="font-size: 1rem; line-height: 1.5; font-style: italic; color: #888;">{"Thinking..."}</div>
                            </div>
                        }
                    } else {
                        html! {}
                    }
                }
            </div>
            <div style="padding: 20px; background-color: #ffffff; border-top: 1px solid #e0e0e0;">
                <form onsubmit={on_submit} style="display: flex; gap: 10px;">
                    <input
                        type="text"
                        value={(*current_input).clone()}
                        oninput={on_input_change}
                        placeholder="Type your message here..."
                        disabled={*waiting_for_response}
                        style={format!("flex: 1; padding: 10px; border: 1px solid #ddd; border-radius: 4px; font-size: 1rem; {}", if *waiting_for_response { "opacity: 0.7;" } else { "" })}
                    />
                    <button
                        type="submit"
                        disabled={*waiting_for_response}
                        style={format!("padding: 10px 20px; background-color: #2563eb; color: white; border: none; border-radius: 4px; cursor: pointer; font-size: 1rem; {}", if *waiting_for_response { "opacity: 0.7; cursor: not-allowed;" } else { "" })}
                    >
                        {if *waiting_for_response { "Sending..." } else { "Submit" }}
                    </button>
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
