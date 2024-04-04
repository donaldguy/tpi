// Copyright 2023 Turing Machines
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Wrapper for `reqwest::Request` that asks for authentication if needed.

use std::ops::{Deref, DerefMut};

use anyhow::{bail, Result};
use reqwest::multipart::Form;
use reqwest::{Client, RequestBuilder, Response, StatusCode};
use url::Url;

use crate::api::Api;
use crate::auth::Auth;

pub struct Request<'a> {
    pub api: Api,
    auth:&'a Auth,
    pub client: reqwest::Client,
    inner: Option<reqwest::Request>,
    multipart: Option<Form>,
}


impl<'a> Request<'a> {
    pub fn new(
        api: Api,
        auth: &'a crate::auth::Auth,
        client: Client,
    ) -> Self {

        Self {
            api,
            auth,
            client,
            inner: None,
            multipart: None,
        }
    }

    pub fn get(&mut self) -> &mut Self {
        self.inner = Some(self.client.get(self.api.base_url.clone()).build().unwrap());

        self
    }

    pub fn post(&mut self) -> &mut Self {
        self.inner = Some(self.client.post(self.api.base_url.clone()).build().unwrap());

        self
    }

    pub fn set_multipart(&mut self, form: Form) {
        self.multipart = Some(form);
    }

    pub async fn send(&mut self) -> Result<Response> {
        let mut authenticated = cfg!(not(feature = "localhost"));

        let resp = loop {
            let mut builder =
                RequestBuilder::from_parts(self.client.clone(), self.inner.as_ref().unwrap().try_clone().unwrap());

            if authenticated {
                let token = self.get_bearer_token(self.client).await?;
                builder = builder.bearer_auth(token);
            }

            if let Some(form) = self.multipart.take() {
                builder = builder.multipart(form);
            }

            let resp = builder.send().await?;
            if resp.status() == StatusCode::UNAUTHORIZED {
                //BmcdBearerToken::delete_cache();
                authenticated = true;
            } else {
                break resp;
            }
        };

        Ok(resp)
    }

    async fn get_bearer_token(&mut self, client: &Client) -> Result<String> {

        // If it doesn't exist, ask on an interactive prompt
        request_token(&self.api, client).await
    }

    pub fn url(&self) -> &Url {
        self.inner.as_ref().unwrap().url()
    }

    pub fn url_mut(&mut self) -> &mut Url {
        self.inner.as_mut().unwrap().url_mut()
    }

    pub fn clone(&self) -> Self {
        let inner = match &self.inner {
            None => None,
            Some(r) => Some(r.try_clone().expect("request cannot be cloned: body is a stream"))
        };

        Self {
            api: self.api.clone(),
            auth: self.auth.clone(),
            client: self.client,
            inner,
            multipart: None,
        }
    }
}

impl <'a> Deref for Request<'a> {
    type Target = reqwest::Request;

    fn deref(&self) -> &Self::Target {
        &self.inner.as_ref().unwrap()
    }
}

impl <'a> DerefMut for Request <'a>{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.as_mut().unwrap()
    }
}


async fn request_token(
    _api: &Api,
    _client: &Client,
) -> Result<String> {
    unimplemented!();

    // let mut auth_url = url_from_host(host, ver.scheme())?;

    // auth_url
    //     .path_segments_mut()
    //     .expect("URL cannot be a base")
    //     .push("authenticate");


    // let body = serde_json::json!({
    //     "username": username,
    //     "password": password
    // });

    // let resp = client.post(auth_url).json(&body).send().await?;

    // match resp.status() {
    //     StatusCode::OK => {
    //         let json = resp.json::<serde_json::Value>().await?;
    //         let token = get_param(&json, "id");

    //         if save_token {
    //             if let Err(e) = cache_token(&token) {
    //                 let path = get_cache_file_location();
    //                 println!("Warning: failed to write to cache file {:?}: {}", path, e);
    //             }
    //         }

    //         Ok(token)
    //     }
    //     StatusCode::FORBIDDEN => bail!(
    //         "{}",
    //         resp.text()
    //             .await
    //             .unwrap_or("could not authenticate".to_string())
    //     ),
    //     x => bail!("Unexpected status code {x}"),
    // }
}

fn get_param(results: &serde_json::Value, key: &str) -> String {
    results
        .get(key)
        .unwrap_or_else(|| panic!("API error: Expected `{key}` attribute"))
        .as_str()
        .unwrap_or_else(|| panic!("API error: `{key}` value is not a string"))
        .to_owned()
}


