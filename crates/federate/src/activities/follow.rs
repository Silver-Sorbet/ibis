use crate::{
    activities::accept::Accept,
    generate_activity_id,
    objects::{instance::InstanceWrapper, user::PersonWrapper},
    send_activity,
};
use activitypub_federation::{
    config::Data,
    fetch::object_id::ObjectId,
    kinds::activity::FollowType,
    protocol::verification::verify_urls_match,
    traits::{ActivityHandler, Actor},
};
use ibis_database::{
    common::instance::Instance,
    error::{BackendError, BackendResult},
    impls::IbisContext,
};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Follow {
    pub actor: ObjectId<PersonWrapper>,
    pub object: ObjectId<InstanceWrapper>,
    #[serde(rename = "type")]
    kind: FollowType,
    id: Url,
}

impl Follow {
    pub fn new(
        actor: &PersonWrapper,
        to: &InstanceWrapper,
        context: &Data<IbisContext>,
    ) -> BackendResult<Self> {
        let id = generate_activity_id(context)?;
        Ok(Follow {
            actor: actor.ap_id.clone().into(),
            object: to.ap_id.clone().into(),
            kind: Default::default(),
            id,
        })
    }

    pub async fn send(
        actor: &PersonWrapper,
        to: &InstanceWrapper,
        context: &Data<IbisContext>,
    ) -> BackendResult<()> {
        let follow = Self::new(actor, to, context)?;
        send_activity(actor, follow, vec![to.shared_inbox_or_inbox()], context).await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl ActivityHandler for Follow {
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
        let actor = self.actor.dereference(context).await?;
        let local_instance: InstanceWrapper = Instance::read_local(context)?.into();
        verify_urls_match(self.object.inner(), local_instance.ap_id.inner())?;
        Instance::follow(&actor, &local_instance, false, context)?;

        // send back an accept
        Accept::send(local_instance, self, context).await?;
        Ok(())
    }
}
