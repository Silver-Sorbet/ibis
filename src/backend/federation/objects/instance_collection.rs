use super::instance::ApubInstance;
use crate::{
    backend::{
        database::IbisData,
        error::{Error, MyResult},
    },
    common::{utils::http_protocol_str, DbInstance},
};
use activitypub_federation::{
    config::Data,
    fetch::collection_id::CollectionId,
    kinds::collection::CollectionType,
    protocol::verification::verify_domains_match,
    traits::{Collection, Object},
};
use futures::future::{self, join_all};
use log::warn;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceCollection {
    pub r#type: CollectionType,
    pub id: Url,
    pub total_items: i32,
    pub items: Vec<ApubInstance>,
}

#[derive(Clone, Debug)]
pub struct DbInstanceCollection(());

pub fn linked_instances_url(domain: &str) -> MyResult<CollectionId<DbInstanceCollection>> {
    Ok(CollectionId::parse(&format!(
        "{}://{domain}/linked_instances",
        http_protocol_str()
    ))?)
}

#[async_trait::async_trait]
impl Collection for DbInstanceCollection {
    type Owner = ();
    type DataType = IbisData;
    type Kind = InstanceCollection;
    type Error = Error;

    async fn read_local(
        _owner: &Self::Owner,
        data: &Data<Self::DataType>,
    ) -> Result<Self::Kind, Self::Error> {
        let instances = DbInstance::read_remote(data)?;
        let instances = future::try_join_all(
            instances
                .into_iter()
                .map(|i| i.into_json(data))
                .collect::<Vec<_>>(),
        )
        .await?;
        let collection = InstanceCollection {
            r#type: Default::default(),
            id: linked_instances_url(&data.config.federation.domain)?.into(),
            total_items: instances.len() as i32,
            items: instances,
        };
        Ok(collection)
    }

    async fn verify(
        json: &Self::Kind,
        expected_domain: &Url,
        _data: &Data<Self::DataType>,
    ) -> Result<(), Self::Error> {
        verify_domains_match(&json.id, expected_domain)?;
        Ok(())
    }

    async fn from_json(
        apub: Self::Kind,
        _owner: &Self::Owner,
        data: &Data<Self::DataType>,
    ) -> Result<Self, Self::Error> {
        let instances =
            apub.items
                .into_iter()
                .filter(|i| !i.id.is_local(data))
                .map(|instance| async {
                    let id = instance.id.clone();
                    let res = DbInstance::from_json(instance, data).await;
                    if let Err(e) = &res {
                        warn!("Failed to synchronize article {id}: {e}");
                    }
                    res
                });
        join_all(instances).await;

        Ok(DbInstanceCollection(()))
    }
}
