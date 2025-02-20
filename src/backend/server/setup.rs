use crate::{
    backend::{
        database::{article::DbArticleForm, instance::DbInstanceForm, IbisContext},
        federation::{
            activities::submit_article_update,
            objects::{
                articles_collection::local_articles_url,
                instance_collection::linked_instances_url,
            },
        },
        utils::{error::BackendError, generate_keypair},
    },
    common::{
        article::{Article, EditVersion},
        instance::Instance,
        user::Person,
        utils::http_protocol_str,
        MAIN_PAGE_NAME,
    },
};
use activitypub_federation::{config::Data, fetch::object_id::ObjectId};
use chrono::Utc;

const MAIN_PAGE_DEFAULT_TEXT: &str = "Welcome to Ibis, the federated Wikipedia alternative!

This main page can only be edited by the admin. Use it as an introduction for new users, \
and to list interesting articles.
";

pub async fn setup(context: &Data<IbisContext>) -> Result<(), BackendError> {
    let domain = &context.config.federation.domain;
    let ap_id = ObjectId::parse(&format!("{}://{domain}", http_protocol_str()))?;
    let inbox_url = format!("{}://{domain}/inbox", http_protocol_str());
    let keypair = generate_keypair()?;
    let form = DbInstanceForm {
        domain: domain.to_string(),
        ap_id,
        articles_url: Some(local_articles_url(domain)?),
        instances_url: Some(linked_instances_url(domain)?),
        inbox_url,
        public_key: keypair.public_key,
        private_key: Some(keypair.private_key),
        last_refreshed_at: Utc::now(),
        local: true,
        topic: None,
        name: None,
    };
    let instance = Instance::create(&form, context)?;

    let person = Person::create_local(
        context.config.setup.admin_username.clone(),
        context.config.setup.admin_password.clone(),
        true,
        context,
    )?;

    // Create the main page which is shown by default
    let form = DbArticleForm {
        title: MAIN_PAGE_NAME.to_string(),
        text: String::new(),
        ap_id: ObjectId::parse(&format!(
            "{}://{domain}/article/{MAIN_PAGE_NAME}",
            http_protocol_str()
        ))?,
        instance_id: instance.id,
        local: true,
        protected: true,
        approved: true,
    };
    let article = Article::create(form, context)?;
    // also create an article so its included in most recently edited list
    submit_article_update(
        MAIN_PAGE_DEFAULT_TEXT.to_string(),
        "Default main page".to_string(),
        EditVersion::default(),
        &article,
        person.person.id,
        context,
    )
    .await?;

    // create ghost user
    Person::ghost(context)?;

    Ok(())
}
