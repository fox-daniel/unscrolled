use gloo::console::log;
use gloo::net::http::Request;
use js_sys::Date;
use shared::api::{ApiEndpoints, get_api_base_url};
use shared::models::{Message, MessageRole};
use std::collections::HashMap;
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
    let messages = use_state(Vec::<Message>::new);

    // State for the current input
    let current_input = use_state(|| String::new());

    // State to track if we're waiting for a response
    let waiting_for_response = use_state(|| false);

    // State to track which assistant messages are collapsed (using message index)
    let collapsed_responses = use_state(|| HashMap::<usize, bool>::new());

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
        let waiting_for_response = waiting_for_response.clone();
        let collapsed_responses = collapsed_responses.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            if current_input.is_empty() || *waiting_for_response {
                return;
            }

            // Collapse all previous assistant responses when submitting a new message
            let current_messages = (*messages).clone();
            let mut new_collapsed_state = (*collapsed_responses).clone();

            for (index, message) in current_messages.iter().enumerate() {
                if matches!(message.role, MessageRole::Assistant) {
                    new_collapsed_state.insert(index, true);
                }
            }
            collapsed_responses.set(new_collapsed_state);

            // Create a new user message
            let new_message = Message {
                content: (*current_input).clone(),
                timestamp: get_current_time(),
                role: MessageRole::User,
            };

            log!(format!("SUBMIT: Adding user message: {:?}", new_message));

            // Update local state with user message
            let mut updated_messages_with_user = current_messages;
            updated_messages_with_user.push(new_message.clone());

            log!(format!(
                "SUBMIT: Final messages being set: {:?}",
                updated_messages_with_user
            ));
            messages.set(updated_messages_with_user.clone());

            // Set waiting state
            waiting_for_response.set(true);

            // Create a fresh callback that uses the updated messages directly
            let messages_for_callback = messages.clone();
            let waiting_for_callback = waiting_for_response.clone();
            let updated_messages_for_callback = updated_messages_with_user; // Use the updated vector directly

            let on_response_received = Callback::from(move |assistant_message: Message| {
                log!(format!(
                    "FRESH CALLBACK: Using updated messages directly: {:?}",
                    updated_messages_for_callback
                ));
                log!(format!(
                    "FRESH CALLBACK: Adding assistant message: {:?}",
                    assistant_message
                ));

                let mut final_messages = updated_messages_for_callback.clone();
                final_messages.push(assistant_message);

                log!(format!(
                    "FRESH CALLBACK: Final messages being set: {:?}",
                    final_messages
                ));
                messages_for_callback.set(final_messages);
                waiting_for_callback.set(false);
            });

            // Send to backend using the fresh callback
            send_message_to_backend(new_message, on_response_received);

            // Clear input
            current_input.set(String::new());
        })
    };

    // Handler for toggling collapse state
    let on_toggle_collapse = {
        let collapsed_responses = collapsed_responses.clone();
        Callback::from(move |index: usize| {
            let mut current_state = (*collapsed_responses).clone();
            let is_collapsed = current_state.get(&index).unwrap_or(&false);
            current_state.insert(index, !is_collapsed);
            collapsed_responses.set(current_state);
        })
    };

    html! {
        <div style="display: flex; flex-direction: column; height: 100vh; max-width: 800px; margin: 0 auto; font-family: system-ui, -apple-system, sans-serif;">
            <div style="flex: 1; overflow-y: auto; padding: 20px; background-color: #f5f5f5;">
                {
                    (*messages).iter().enumerate().map(|(index, message)| {
                        let (bg_color, label) = match message.role {
                            MessageRole::User => ("#e3f2fd", "You:"),
                            MessageRole::Assistant => ("#f1f8e9", "Assistant:"),
                        };

                        let is_assistant = matches!(message.role, MessageRole::Assistant);
                        let is_collapsed = is_assistant && *collapsed_responses.get(&index).unwrap_or(&false);

                        let toggle_callback = {
                            let on_toggle = on_toggle_collapse.clone();
                            Callback::from(move |_: MouseEvent| {
                                on_toggle.emit(index);
                            })
                        };

                        html! {
                            <div style={format!("margin-bottom: 15px; padding: 15px; background-color: {}; border-radius: 8px; box-shadow: 0 1px 3px rgba(0,0,0,0.1);", bg_color)}>
                                <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 5px;">
                                    <div style="font-size: 0.8rem; color: #666; font-weight: bold;">{format!("{} {}", label, &message.timestamp)}</div>
                                    {
                                        if is_assistant {
                                            html! {
                                                <button
                                                    onclick={toggle_callback}
                                                    style="background: none; border: 1px solid #ccc; border-radius: 4px; padding: 4px 8px; cursor: pointer; font-size: 0.75rem; color: #666; hover: background-color: #f0f0f0;"
                                                    title={if is_collapsed { "Expand response" } else { "Collapse response" }}
                                                >
                                                    {if is_collapsed { "▶" } else { "▼" }}
                                                </button>
                                            }
                                        } else {
                                            html! {}
                                        }
                                    }
                                </div>
                                {
                                    if !is_collapsed {
                                        html! {
                                            <div style="font-size: 1rem; line-height: 1.5; white-space: pre-wrap; word-break: break-word;">{&message.content}</div>
                                        }
                                    } else {
                                        html! {
                                            <div style="font-size: 0.9rem; color: #888; font-style: italic;">{"Response collapsed"}</div>
                                        }
                                    }
                                }
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
