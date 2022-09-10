//! api 调用

#![allow(dead_code)]
use anyhow::Result;
use reqwest::blocking;
use reqwest::Client;
use std::os::windows::process::CommandExt;
use std::{
    process::{Command, Stdio},
    time::Duration,
};

use serde_json::Value;

use super::entity::{GameSession, LcuError, MatchHistory, SendInfo, Summoner};
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

    /// 获取房间信息
    pub async fn get_lobby(self) -> Result<Value> {
        let url = self.base_url + "/lol-lobby/v2/lobby";

        get_resp(url, self.client, self.token).await
    }
    /// 获取用户信息
    pub async fn get_summoners(self, id: &str) -> Result<Summoner> {
        let url = self.base_url + "/lol-summoner/v1/summoners/" + id;

        get_resp(url, self.client, self.token).await
    }
    /// 获取选人界面session
    pub async fn get_session(self) -> Result<GameSession> {
        let url = self.base_url + "/lol-champ-select/v1/session";

        get_resp(url, self.client, self.token).await
    }
    /// 英雄名称
    pub async fn get_hero_name(self, id: u8) -> Result<Value> {
        let url = format!(
            "{}/lol-game-data/assets/v1/champions/{}.json",
            self.base_url, id
        );

        get_resp(url, self.client, self.token).await
    }

    /// 游戏状态
    pub async fn get_gameflow_phase(self) -> Result<Value> {
        let url = self.base_url + "/lol-gameflow/v1/gameflow-phase";

        get_resp(url, self.client, self.token).await
    }
    /// 获取当前用户信息
    pub async fn get_current_summoner(self) -> Result<Summoner> {
        let url = self.base_url + "/lol-summoner/v1/current-summoner";
        get_resp(url, self.client, self.token).await
    }
    /// 获取用户对战记录
    pub async fn get_match_history(self, puuid: &str, beg_index: u32) -> Result<MatchHistory> {
        let url = format!(
            "{}/lol-match-history/v1/products/lol/{}/matches?begIndex={}&endIndex={}",
            self.base_url,
            puuid,
            (beg_index - 1) * 20,
            beg_index * 20
        );
        get_resp(url, self.client, self.token).await
       
        // get_resp(url, self.client, self.token).await
    }
    /// 选人界面发送信息
    pub async fn send_message(self, chat_room_id: &str, body: SendInfo) -> Result<Value> {
        let url = self.base_url + "/lol-chat/v1/conversations/" + chat_room_id + "/messages";

        match self.client {
            Some(cli) => {
                let resp = cli
                    .post(url)
                    .basic_auth("riot", Some(self.token.as_str()))
                    .json(&body)
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
}
async fn get_resp<T>(url: String, client: Option<Client>, token: String) -> Result<T>
where
    for<'de> T: serde::Deserialize<'de>,
{
    println!("{}", url);
    match client {
        Some(cli) => {
            let resp = cli
                .get(url)
                .basic_auth("riot", Some(token.as_str()))
                .timeout(Duration::new(2, 0))
                .send()
                .await?
                .json::<T>()
                .await?;
            Ok(resp)
        }
        None => Err(anyhow::anyhow!("not find client")),
    }
}

#[derive(Default, Clone)]
pub struct ApiClientBlock {
    client: Option<blocking::Client>,
    base_url: String,
    token: String,
    pub init_error: Option<LcuError>,
}

impl ApiClientBlock {
    pub fn init_client(&mut self) {
        let mut base_info = BaseInfo::default();
        match base_info.init() {
            Ok(_) => {
                let client = reqwest::blocking::Client::builder()
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

    pub fn get_lobby(self) -> Result<Value> {
        let url = self.base_url + "/lol-lobby/v2/lobby";

        get_resp_block(url, self.client, self.token)
    }

    pub fn get_gameflow_phase(self) -> Result<Value> {
        let url = self.base_url + "/lol-gameflow/v1/gameflow-phase";

        get_resp_block(url, self.client, self.token)
    }

    pub fn get_current_summoner(self) -> Result<Summoner> {
        let url = self.base_url + "/lol-summoner/v1/current-summoner";
        get_resp_block(url, self.client, self.token)
    }
}

fn get_resp_block<T>(url: String, client: Option<blocking::Client>, token: String) -> Result<T>
where
    for<'de> T: serde::Deserialize<'de>,
{
    println!("{}", url);
    match client {
        Some(cli) => {
            let resp = cli
                .get(url)
                .basic_auth("riot", Some(token.as_str()))
                .timeout(Duration::new(2, 0))
                .send()?
                .json::<T>()?;

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

        let r = tokio_test::block_on(api.get_current_summoner());
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
