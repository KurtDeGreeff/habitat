// Copyright (c) 2016 Chef Software Inc. and/or applicable contributors
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

use std::collections::HashMap;
use std::fmt;
use std::io::Read;

use hyper::{self, Url};
use hyper::status::StatusCode;
use hyper::header::{Authorization, Accept, Bearer, UserAgent, qitem};
use hyper::mime::{Mime, TopLevel, SubLevel};
use protocol::sessionsrv;
use rustc_serialize::json;

use config;
use error::{Error, Result};

const USER_AGENT: &'static str = "Habitat-Builder";

pub struct GitHubClient {
    pub url: String,
    pub client_id: String,
    pub client_secret: String,
}

impl GitHubClient {
    pub fn new<T: config::GitHubOAuth>(config: &T) -> Self {
        GitHubClient {
            url: config.github_url().to_string(),
            client_id: config.github_client_id().to_string(),
            client_secret: config.github_client_secret().to_string(),
        }
    }

    pub fn authenticate(&self, code: &str) -> Result<String> {
        let url =
            Url::parse(&format!("https://github.\
                                 com/login/oauth/access_token?client_id={}&client_secret={}&code={}",
                                self.client_id,
                                self.client_secret,
                                code))
                .unwrap();
        let mut rep = try!(http_post(url));
        if rep.status.is_success() {
            let mut encoded = String::new();
            try!(rep.read_to_string(&mut encoded));
            match json::decode(&encoded) {
                Ok(msg @ AuthOk { .. }) => {
                    let scope = "user:email".to_string();
                    if msg.has_scope(&scope) {
                        Ok(msg.access_token)
                    } else {
                        Err(Error::MissingScope(scope))
                    }
                }
                Err(_) => {
                    let err: AuthErr = try!(json::decode(&encoded));
                    Err(Error::from(err))
                }
            }
        } else {
            Err(Error::HTTP(rep.status))
        }
    }

    pub fn user(&self, token: &str) -> Result<User> {
        let url = Url::parse(&format!("{}/user", self.url)).unwrap();
        let mut rep = try!(http_get(url, token));
        let mut body = String::new();
        try!(rep.read_to_string(&mut body));
        if rep.status != StatusCode::Ok {
            let err: HashMap<String, String> = try!(json::decode(&body));
            return Err(Error::GitHubAPI(err));
        }
        let user: User = json::decode(&body).unwrap();
        Ok(user)
    }

    pub fn emails(&self, token: &str) -> Result<Vec<Email>> {
        let url = Url::parse(&format!("{}/user/emails", self.url)).unwrap();
        let mut rep = try!(http_get(url, token));
        let mut body = String::new();
        try!(rep.read_to_string(&mut body));
        if rep.status != StatusCode::Ok {
            let err: HashMap<String, String> = try!(json::decode(&body));
            return Err(Error::GitHubAPI(err));
        }
        let emails: Vec<Email> = try!(json::decode(&body));
        Ok(emails)
    }
}

#[derive(Debug, RustcEncodable, RustcDecodable)]
pub struct User {
    pub login: String,
    pub id: u64,
    pub avatar_url: String,
    pub gravatar_id: String,
    pub url: String,
    pub html_url: String,
    pub followers_url: String,
    pub following_url: String,
    pub gists_url: String,
    pub starred_url: String,
    pub subscriptions_url: String,
    pub organizations_url: String,
    pub repos_url: String,
    pub events_url: String,
    pub received_events_url: String,
    pub site_admin: bool,
    pub name: Option<String>,
    pub company: Option<String>,
    pub blog: Option<String>,
    pub location: Option<String>,
    pub email: Option<String>,
    pub hireable: Option<bool>,
    pub bio: Option<String>,
    pub public_repos: u32,
    pub public_gists: u32,
    pub followers: u32,
    pub following: u32,
    pub created_at: String,
    pub updated_at: String,
}

impl From<User> for sessionsrv::Account {
    fn from(user: User) -> sessionsrv::Account {
        let mut account = sessionsrv::Account::new();
        account.set_name(user.login);
        if let Some(email) = user.email {
            account.set_email(email);
        }
        account
    }
}

#[derive(Debug, RustcEncodable, RustcDecodable)]
pub struct Email {
    pub email: String,
    pub primary: bool,
    pub verified: bool,
}

#[derive(Debug, RustcDecodable, RustcEncodable)]
pub struct AuthOk {
    pub access_token: String,
    pub scope: String,
    pub token_type: String,
}

impl AuthOk {
    pub fn has_scope(&self, grant: &str) -> bool {
        self.scope.split(",").collect::<Vec<&str>>().iter().any(|&p| p == grant)
    }
}

#[derive(RustcDecodable, RustcEncodable, Debug)]
pub struct AuthErr {
    pub error: String,
    pub error_description: String,
    pub error_uri: String,
}

impl fmt::Display for AuthErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "err={}, desc={}, uri={}",
               self.error,
               self.error_description,
               self.error_uri)
    }
}

#[derive(RustcDecodable, RustcEncodable)]
pub enum AuthResp {
    AuthOk,
    AuthErr,
}

fn http_get(url: Url, token: &str) -> Result<hyper::client::response::Response> {
    hyper::Client::new()
        .get(url)
        .header(Accept(vec![qitem(Mime(TopLevel::Application, SubLevel::Json, vec![]))]))
        .header(Authorization(Bearer { token: token.to_owned() }))
        .header(UserAgent(USER_AGENT.to_string()))
        .send()
        .map_err(|e| Error::from(e))
}

fn http_post(url: Url) -> Result<hyper::client::response::Response> {
    hyper::Client::new()
        .post(url)
        .header(Accept(vec![qitem(Mime(TopLevel::Application, SubLevel::Json, vec![]))]))
        .send()
        .map_err(|e| Error::from(e))
}
