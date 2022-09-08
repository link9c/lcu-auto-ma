//! api 调用

// #![allow(dead_code)]
use anyhow::Result;
use reqwest::Client;
use std::os::windows::process::CommandExt;
use std::{
    process::{Command, Stdio},
    time::Duration,
};

use serde_json::Value;

use super::entity::LcuError;
use super::utils::parse_cmd_outoput;

#[derive(Default)]
struct BaseInfo {
    url: String,
    token: String,
}

impl BaseInfo {
    pub fn init(&mut self) -> Result<(), LcuError> {
        let out = Command::new("cmd")
            .creation_flags(0x08000000)
            .arg("/c")
            .arg("wmic PROCESS WHERE name='LeagueClientUx.exe' GET commandline")
            .stdout(Stdio::piped())
            .output();

        match out {
            Ok(output) => {
                let stdout_txt = String::from_utf8_lossy(&output.stdout)
                    .replace('\r', "")
                    .replace('\n', "")
                    .replace(' ', "");
                println!("{}", stdout_txt);
                if stdout_txt.contains("没有") {
                    Err(LcuError::NoEntity)
                } else if stdout_txt == "CommandLine" {
                    Err(LcuError::NotAdmin)
                } else {
                    let app = parse_cmd_outoput(&stdout_txt);
                    let port = app.get("app-port");
                    let token = app.get("remoting-auth-token");
                    match (port, token) {
                        (Some(p), Some(t)) => {
                            println!("---{},{}---", p, t);
                            self.token = t.to_string();
                            self.url = format!("https://127.0.0.1:{}", p);
                            Ok(())
                        }
                        (_, _) => Err(LcuError::NoEntity),
                    }
                }
            }
            Err(_) => Err(LcuError::Other),
        }
    }
}

#[derive(Default, Clone)]
pub struct ApiClient {
    client: Option<Client>,
    base_url: String,
    token: String,
    pub init_error: Option<LcuError>,
}

impl ApiClient {
    pub fn init_client(&mut self) {
        let mut base_info = BaseInfo::default();
        match base_info.init() {
            Ok(_) => {
                let client = reqwest::Client::builder()
                    .danger_accept_invalid_certs(true)
                    .build()
                    .unwrap();
                self.client = Some(client);
                self.base_url = base_info.url;
                self.token = base_info.token;
            }
            Err(e) => {
                self.init_error = Some(e);
            }
        }
    }

    pub async fn get_lobby(self) -> Result<Value> {
        let url = self.base_url + "/lol-lobby/v2/lobby";

        get_resp(url, self.client, self.token).await
    }

    pub async fn get_gameflow_phase(self) -> Result<Value> {
        let url = self.base_url + "/lol-gameflow/v1/gameflow-phase";

        get_resp(url, self.client, self.token).await
    }

    pub async fn get_summoner(self) -> Result<Value> {
        let url = self.base_url + "/lol-summoner/v1/current-summoner";
        get_resp(url, self.client, self.token).await
    }
}
async fn get_resp(url: String, client: Option<Client>, token: String) -> Result<Value> {
    println!("{}", url);
    match client {
        Some(cli) => {
            let resp = cli
                .get(url)
                .basic_auth("riot", Some(token.as_str()))
                .timeout(Duration::new(2, 0))
                .send()
                .await?
                .json::<Value>()
                .await?;
            Ok(resp)
        }
        None => Err(anyhow::anyhow!("not find client")),
    }
}
mod tests {

    #[test]
    fn test_summoner() {
        let mut api = super::ApiClient::default();
        api.init_client();

        let r = tokio_test::block_on(api.get_summoner());
        println!("summoner:{:?}", r);
        assert_eq!(true, true);
    }
    #[test]
    fn test_lobby() {
        let mut api = super::ApiClient::default();
        api.init_client();

        let r = tokio_test::block_on(api.get_lobby());
        println!("lobby:{:?}", r);
        assert_eq!(true, true);
    }

    #[test]
    fn test_gameflow_phase() {
        let mut api = super::ApiClient::default();
        api.init_client();

        let r = tokio_test::block_on(api.get_gameflow_phase());
        println!("gameflow_phase:{:?}", r);
        assert_eq!(true, true);
    }
}
