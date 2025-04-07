use serde::Deserialize;
use serde_json::Value;
use std::time::Duration;
use uuid::Uuid;

static REQWEST_TIMEOUT: u64 = 10;

#[derive(Debug)]
pub struct SS14AuthClientService {
    inner: reqwest::Client,
    discord_auth_uri: String,
    discord_auth_token: String,
    ss14_auth_uri: String,
}

impl SS14AuthClientService {
    pub fn new(
        discord_auth_uri: String,
        discord_auth_token: String,
        ss14_auth_uri: String,
    ) -> Result<Self, crate::error::Error> {
        let client = reqwest::ClientBuilder::new()
            .timeout(Duration::from_secs(REQWEST_TIMEOUT))
            .build()?;

        Ok(Self {
            inner: client,
            discord_auth_token,
            discord_auth_uri,
            ss14_auth_uri,
        })
    }

    pub async fn get_user_id(&self, login: String) -> Result<Option<Uuid>, crate::error::Error> {
        #[derive(Deserialize)]
        struct JsonResponseBody {
            #[serde(rename = "userId")]
            user_id: String,
        }

        let result = self
            .inner
            .get(format!("{}/api/query/name", self.ss14_auth_uri))
            .query(&[("name", login)])
            .send()
            .await?;

        if result.status() == 404 {
            return Ok(None);
        }

        let body = result.json::<JsonResponseBody>().await?;

        Ok(body.user_id.parse::<Uuid>().ok())
    }

    pub async fn get_discord_id(&self, uuid: Uuid) -> Result<Option<String>, crate::error::Error> {
        #[derive(Deserialize)]
        struct JsonResponseBody {
            id: String,
        }

        let result = self
            .inner
            .get(format!("{}/api/identify", self.discord_auth_uri))
            .bearer_auth(self.discord_auth_token.as_str())
            .query(&[("id", uuid.to_string()), ("method", "uid".to_string())])
            .send()
            .await?;

        if result.status() == 404 {
            return Ok(None);
        }

        let body = result.json::<JsonResponseBody>().await?;
        Ok(Some(body.id))
    }

    pub async fn get_user_id_from_discord(
        &self,
        discord_id: String,
    ) -> Result<Option<Uuid>, crate::error::Error> {
        #[derive(Deserialize)]
        struct JsonResponseBody {
            #[serde(rename = "uuid")]
            user_id: String,
        }

        let result = self
            .inner
            .get(format!("{}/api/uuid", self.discord_auth_uri))
            .bearer_auth(self.discord_auth_token.as_str())
            .query(&[("method", "discord".to_string()), ("id", discord_id)])
            .send()
            .await?;

        match result.status().as_u16() {
            404 => Ok(None),
            200 => {
                let body = result.json::<JsonResponseBody>().await?;
                Ok(Some(body.user_id.parse::<Uuid>()?))
            }
            _ => Err(crate::error::Error::auth("Error occurred at auth service")),
        }
    }

    pub async fn get_extra_data(
        &self,
        discord_id: String,
    ) -> Result<Option<Value>, crate::error::Error> {
        let result = self
            .inner
            .get(format!("{}/api/extra", self.discord_auth_uri))
            .bearer_auth(self.discord_auth_token.as_str())
            .query(&[("method", "discord".to_string()), ("id", discord_id)])
            .send()
            .await?;

        match result.status().as_u16() {
            404 => Ok(None),
            200 => {
                let body = result.json::<Value>().await?;

                Ok(Some(body))
            }
            _ => Err(crate::error::Error::auth("Error occurred at auth service")),
        }
    }
}
