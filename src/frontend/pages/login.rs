use crate::{
    common::LoginUserForm,
    frontend::{api::CLIENT, app::site, components::credentials::*},
};
use leptos::prelude::*;
use leptos_router::components::Redirect;

#[component]
pub fn Login() -> impl IntoView {
    let (login_response, set_login_response) = signal(None::<()>);
    let (login_error, set_login_error) = signal(None::<String>);
    let (wait_for_response, set_wait_for_response) = signal(false);

    let login_action = Action::new(move |(email, password): &(String, String)| {
        let username = email.to_string();
        let password = password.to_string();
        let credentials = LoginUserForm { username, password };
        async move {
            set_wait_for_response.update(|w| *w = true);
            let result = CLIENT.login(credentials).await;
            set_wait_for_response.update(|w| *w = false);
            match result {
                Ok(_res) => {
                    site().refetch();
                    set_login_response.update(|v| *v = Some(()));
                    set_login_error.update(|e| *e = None);
                }
                Err(err) => {
                    let msg = err.to_string();
                    log::warn!("Unable to login: {msg}");
                    set_login_error.update(|e| *e = Some(msg));
                }
            }
        }
    });

    let disabled = Signal::derive(move || wait_for_response.get());

    view! {
        <Show
            when=move || login_response.get().is_some()
            fallback=move || {
                view! {
                    <CredentialsForm
                        title="Login"
                        action_label="Login"
                        action=login_action
                        error=login_error.into()
                        disabled
                    />
                }
            }
        >

            <Redirect path="/" />
        </Show>
    }
}
