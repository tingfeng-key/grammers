pub use grammers_mtsender::{AuthorizationError, InvocationError};
use grammers_tl_types as tl;

pub struct QRLogin {
    pub(crate) except_ids: Vec<i64>,
    pub(crate) login_token: tl::enums::auth::LoginToken,
    pub(crate) client: crate::Client,
}

impl QRLogin {
    pub(crate) async fn new(
        client: crate::Client,
        except_ids: Vec<i64>,
    ) -> Result<Self, AuthorizationError> {
        let login_token = client
            .invoke(&tl::functions::auth::ExportLoginToken {
                api_id: client.0.config.api_id,
                api_hash: client.0.config.api_hash.clone(),
                except_ids: except_ids.clone(),
            })
            .await?;
        Ok(Self {
            except_ids,
            login_token,
            client,
        })
    }

    pub fn token(&self) -> Vec<u8> {
        match self.login_token.clone() {
            tl::enums::auth::LoginToken::Token(x) => x.token,
            tl::enums::auth::LoginToken::MigrateTo(_x) => panic!("Unexpected result"),
            tl::enums::auth::LoginToken::Success(_x) => panic!("Unexpected result"),
        }
    }

    pub fn expires(&self) -> i32 {
        match self.login_token.clone() {
            tl::enums::auth::LoginToken::Token(x) => x.expires,
            tl::enums::auth::LoginToken::MigrateTo(_x) => panic!("Unexpected result"),
            tl::enums::auth::LoginToken::Success(_x) => panic!("Unexpected result"),
        }
    }

    pub async fn wait(&mut self) -> Result<super::User, AuthorizationError> {
        loop {
            if let Some(crate::types::Update::Raw(tl::enums::Update::LoginToken)) =
                self.client.next_update().await?
            {
                let request = tl::functions::auth::ExportLoginToken {
                    api_id: self.client.0.config.api_id,
                    api_hash: self.client.0.config.api_hash.clone(),
                    except_ids: self.except_ids.clone(),
                };

                match self.client.invoke(&request).await {
                    Ok(x) => match x {
                        tl::enums::auth::LoginToken::Token(_x) => {
                            panic!("should not have logged in yet")
                        }
                        tl::enums::auth::LoginToken::MigrateTo(x) => {
                            let token = self
                                .client
                                .invoke(&tl::functions::auth::ImportLoginToken { token: x.token })
                                .await?;
                            self.login_token = token;
                        }
                        tl::enums::auth::LoginToken::Success(x) => {
                            let user = match x.authorization {
                                tl::enums::auth::Authorization::Authorization(x) => {
                                    self.client.complete_login(x).await?
                                }
                                tl::enums::auth::Authorization::SignUpRequired(_) => {
                                    panic!("Unexpected result")
                                }
                            };
                            return Ok(user);
                        }
                    },
                    Err(e) => return Err(e.into()),
                }
            }
        }
    }
}
