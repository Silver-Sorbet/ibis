use crate::{
    activities::follow::Follow,
    generate_activity_id,
    objects::instance::InstanceWrapper,
    send_activity,
};
use activitypub_federation::{
    config::Data,
    fetch::object_id::ObjectId,
    kinds::activity::AcceptType,
    traits::{ActivityHandler, Actor},
};
use ibis_database::{
    common::instance::Instance,
    error::{BackendError, BackendResult},
    impls::IbisContext,
};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Accept {
    actor: ObjectId<InstanceWrapper>,
    object: Follow,
    #[serde(rename = "type")]
    kind: AcceptType,
    id: Url,
}

impl Accept {
    pub async fn send(
        local_instance: InstanceWrapper,
        object: Follow,
        context: &Data<IbisContext>,
    ) -> BackendResult<()> {
        let id = generate_activity_id(context)?;
        let follower = object.actor.dereference(context).await?;
        let accept = Accept {
            actor: local_instance.ap_id.clone().into(),
            object,
            kind: Default::default(),
            id,
        };
        send_activity(
            &local_instance,
            accept,
            vec![follower.shared_inbox_or_inbox()],
            context,
        )
        .await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl ActivityHandler for Accept {
    type DataType = IbisContext;
    type Error = BackendError;

    fn id(&self) -> &Url {
        &self.id
    }

    fn actor(&self) -> &Url {
        self.actor.inner()
    }

    async fn verify(&self, _context: &Data<Self::DataType>) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn receive(self, context: &Data<Self::DataType>) -> Result<(), Self::Error> {
        // add to follows
        let person = self.object.actor.dereference_local(context).await?;
        let instance = self.actor.dereference(context).await?;
        Instance::follow(&person, &instance, false, context)?;
        Ok(())
    }
}
