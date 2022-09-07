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

use super::utils::parse_cmd_outoput;

#[derive(Default)]
struct BaseInfo {
    url: String,
    token: String,
}

impl BaseInfo {
    pub fn init(&mut self) -> Result<(), std::io::Error> {
        let out = Command::new("cmd")
            .creation_flags(0x08000000)
            .arg("/c")
            .arg("wmic PROCESS WHERE name='LeagueClientUx.exe' GET commandline")
            .stdout(Stdio::piped())
            .output();
        let res = match out {
            Ok(output) => {
                let stdout_txt = String::from_utf8_lossy(&output.stdout);
                let app = parse_cmd_outoput(&stdout_txt);
                let port = app.get("app-port");
                let token = app.get("remoting-auth-token");
                match (port, token) {
                    (Some(p), Some(t)) => {
                        println!("---{},{}---", p, t);
                        self.token = t.to_string();
                        self.url = format!("https://127.0.0.1:{}", p);
                    }
                    (_, _) => {
                        println!("---10086,100---");
                        self.token = "".to_string();
                        self.url = format!("https://127.0.0.1:{}", 10086);
                    }
                }

                Ok(())
            }
            Err(e) => Err(e),
        };
        res
    }
}

#[derive(Default, Clone)]
pub struct ApiClient {
    client: Option<Client>,
    base_url: String,
    token: String,
}

impl ApiClient {
    pub fn init_client(&mut self) {
        let mut base_info = BaseInfo::default();
        if base_info.init().is_ok() {
            let client = reqwest::Client::builder()
                .danger_accept_invalid_certs(true)
                .build()
                .unwrap();
            self.client = Some(client);
            self.base_url = base_info.url;
            self.token = base_info.token;
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
    let resp = client
        .unwrap()
        .get(url)
        .basic_auth("riot", Some(token.as_str()))
        .timeout(Duration::new(2, 0))
        .send()
        .await?
        .text()
        .await?;
    let res = serde_json::from_str::<Value>(&resp)?;
    Ok(res)
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
