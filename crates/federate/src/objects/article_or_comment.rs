use super::{
    article::{ApubArticle, ArticleWrapper},
    comment::{ApubComment, CommentWrapper},
};
use activitypub_federation::{config::Data, traits::Object};
use chrono::{DateTime, Utc};
use ibis_database::{
    error::{BackendError, BackendResult},
    impls::IbisContext,
};
use serde::Deserialize;
use url::Url;

#[derive(Clone, Debug)]
pub enum DbArticleOrComment {
    Article(ArticleWrapper),
    Comment(CommentWrapper),
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum ApubArticleOrComment {
    Article(Box<ApubArticle>),
    Comment(Box<ApubComment>),
}

#[async_trait::async_trait]
impl Object for DbArticleOrComment {
    type DataType = IbisContext;
    type Kind = ApubArticleOrComment;
    type Error = BackendError;

    fn last_refreshed_at(&self) -> Option<DateTime<Utc>> {
        None
    }

    async fn read_from_id(
        object_id: Url,
        context: &Data<Self::DataType>,
    ) -> BackendResult<Option<Self>> {
        let post = ArticleWrapper::read_from_id(object_id.clone(), context).await?;
        Ok(match post {
            Some(o) => Some(Self::Article(o)),
            None => CommentWrapper::read_from_id(object_id, context)
                .await?
                .map(Self::Comment),
        })
    }

    async fn delete(self, context: &Data<Self::DataType>) -> BackendResult<()> {
        match self {
            Self::Article(p) => p.delete(context).await,
            Self::Comment(c) => c.delete(context).await,
        }
    }

    async fn into_json(self, context: &Data<Self::DataType>) -> BackendResult<Self::Kind> {
        Ok(match self {
            Self::Article(p) => Self::Kind::Article(Box::new(p.into_json(context).await?)),
            Self::Comment(c) => Self::Kind::Comment(Box::new(c.into_json(context).await?)),
        })
    }

    async fn verify(
        apub: &Self::Kind,
        expected_domain: &Url,
        context: &Data<Self::DataType>,
    ) -> BackendResult<()> {
        match apub {
            Self::Kind::Article(a) => ArticleWrapper::verify(a, expected_domain, context).await,
            Self::Kind::Comment(a) => CommentWrapper::verify(a, expected_domain, context).await,
        }
    }

    async fn from_json(apub: Self::Kind, context: &Data<Self::DataType>) -> BackendResult<Self> {
        Ok(match apub {
            Self::Kind::Article(p) => Self::Article(ArticleWrapper::from_json(*p, context).await?),
            Self::Kind::Comment(n) => Self::Comment(CommentWrapper::from_json(*n, context).await?),
        })
    }
}
