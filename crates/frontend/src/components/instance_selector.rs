use ibis_database::common::instance::InstanceWithArticles;
use ibis_api_client::CLIENT;
use super::suspense_error::SuspenseError;


use leptos::prelude::*;

#[component]
pub fn InstanceSelector(
    wait_for_response: ReadSignal<bool>,
    instance: ReadSignal<Option<i32>>,
    set_instance: WriteSignal<Option<i32>>,
) -> impl IntoView {
    let instances = Resource::new(move || (), |_| async move { CLIENT.list_instances().await });
    view!{
        <select
        class="select select-bordered"
        required
        on:change:target=move |ev| {
            let val = ev.target().value().parse::<i32>().unwrap_or(-1);
            set_instance.set(
                match val{
                    -1 => None,
                    i => Some(i)
                }
            )
        }
        prop:value=move || instance.get()
        prop:disabled=move || wait_for_response.get()
        >
            <SuspenseError result=instances>
                {move || Suspend::new(async move {
                    let instances_ = instances.await;
                    let is_empty = instances_.as_ref().map(|i| i.is_empty()).unwrap_or(true);
                    view!{
                    <Show
                        when=move || !is_empty
                        fallback=move || view! { Loading... }
                    >
                    
                    {instances_
                        .clone()
                        .ok()
                        .iter()
                        .flatten()
                        .map(instance_options)
                        .collect::<Vec<_>>()}
                    </Show>
                    }
                })}
                
            </SuspenseError>
        </select>
    }
}

pub fn instance_options(instance_view: &InstanceWithArticles) -> impl IntoView {
    println!("InstanceId at get: {}", instance_view.instance.id.0);
    view! {
        <option 
        value={instance_view.instance.id.0}>
        {instance_view.instance.domain.to_string()}
        </option>
        .into_any()
    }
}

