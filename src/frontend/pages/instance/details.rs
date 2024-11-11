use crate::{
    common::{utils::http_protocol_str, DbInstance, ListArticlesForm},
    frontend::{
        app::GlobalState,
        article_link,
        article_title,
        components::instance_follow_button::InstanceFollowButton,
    },
};
use leptos::*;
use leptos_router::use_params_map;
use url::Url;

#[component]
pub fn InstanceDetails() -> impl IntoView {
    let params = use_params_map();
    let hostname = move || params.get().get("hostname").cloned().unwrap();
    let instance_profile = create_resource(hostname, move |hostname| async move {
        let url = Url::parse(&format!("{}://{hostname}", http_protocol_str())).unwrap();
        GlobalState::api_client()
            .resolve_instance(url)
            .await
            .unwrap()
    });

    view! {
        <Suspense fallback=|| {
            view! { "Loading..." }
        }>
            {move || {
                instance_profile
                    .get()
                    .map(|instance: DbInstance| {
                        let articles = create_resource(
                            move || instance.id,
                            |instance_id| async move {
                                GlobalState::api_client()
                                    .list_articles(ListArticlesForm {
                                        only_local: None,
                                        instance_id: Some(instance_id),
                                    })
                                    .await
                                    .unwrap()
                            },
                        );
                        let instance_ = instance.clone();
                        view! {
                            <div class="grid gap-3 mt-4">
                                <div class="flex flex-row items-center">
                                    <h1 class="text-4xl font-bold font-serif w-full">
                                        {instance.domain}
                                    </h1>
                                    <InstanceFollowButton instance=instance_.clone() />
                                </div>

                                <div class="divider"></div>
                                <div>{instance.description}</div>
                                <h2 class="text-xl font-bold font-serif">Articles</h2>
                                <ul class="list-none">
                                    {move || {
                                        articles
                                            .get()
                                            .map(|a| {
                                                a.into_iter()
                                                    .map(|a| {
                                                        view! {
                                                            <li>
                                                                <a class="link text-lg" href=article_link(&a)>
                                                                    {article_title(&a)}
                                                                </a>
                                                            </li>
                                                        }
                                                    })
                                                    .collect::<Vec<_>>()
                                            })
                                    }}

                                </ul>
                            </div>
                        }
                    })
            }}

        </Suspense>
    }
}