// 从lcu获取port token 地址等 返回 结构体ClientInfo
#![allow(non_snake_case)]
#![allow(dead_code)]
use std::process::{Command, Stdio};

use anyhow::Result;
use reqwest::Client;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Default)]
struct BaseInfo {
    url: String,
    token: String,
}

impl BaseInfo {
    pub fn init(&mut self) -> Result<(), std::io::Error> {
        let out = Command::new("cmd")
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
                // let port = "61445".to_string();

                // self.token="4uyEUBeXbCBP0Lq3fW2eWQ".to_string();

                Ok(())
            }
            Err(e) => Err(e),
        };
        res
    }
}

fn parse_cmd_outoput(r: &str) -> HashMap<String, String> {
    // let r = "agueClient/LeagueClientUx.exe \"--riotclient-auth-token=psm0ORo2keEAcoOaMcw8ww\" \"--riotclient-app-port=63316\" \"--riotclient-tencent\" \"--no-rads\" \"--disable-self-update\" \"--region=TENCENT\" \"--locale=zh_CN\" \"--t.lcdshost=hn1-sz-feapp.lol.qq.com\" \"--t.chathost=hn1-sz-ejabberd.lol.qq.com\" \"--t.lq=https://hn1-sz-login.lol.qq.com:8443\" \"--t.storeurl=https://hn1-sr.lol.qq.com:8443\" \"--t.rmsurl=wss://sz-rms-bcs.lol.qq.com:443\" \"--rso-auth.url=https://prod-rso.lol.qq.com:3000\" \"--rso_platform_id=HN1\" \"--rso-auth.client=lol\" \"--t.location=loltencent.sz.HN1\" \"--tglog-endpoint=https://tglogsz.datamore.qq.com/lolcli/report/\" \"--t.league_edge_url=https://ledge-hn1.lol.qq.com:22019\" \"--ccs=https://cc-hn1.lol.qq.com:8093\" \"--dradis-endpoint=http://some.url\" \"--remoting-auth-token=tzwyGAFfukloNZyEgUR02w\" \"--app-port=63481\" \"--install-directory=e:\\game\\英雄联盟\\LeagueClient\" \"--app-name=LeagueClient\" \"--ux-name=LeagueClientUx\" \"--ux-helper-name=LeagueClientUxHelper\" \"--log-dir=LeagueClient Logs\" \"--crash-reporting=\" \"--crash-environment=HN1\" \"--app-log-file-path=e:/game/Ӣ������/LeagueClient/../Game/Logs/LeagueClient Logs/2022-09-04T14-52-04_20576_LeagueClient.log\" \"--app-pid=20576\" \"--output-base-dir=e:/game/英雄联盟/LeagueClient/../Game\" \"--no-proxy-server\"  \r\r\n\r\r\n";
    let mut result: HashMap<String, String> = HashMap::new();
    for args in r.replace('\"', "").replace(' ', "").split("--") {
        let arg_vec = args.split('=').collect::<Vec<&str>>();

        if arg_vec[0] == "app-port" || arg_vec[0] == "remoting-auth-token" {
            result.insert(arg_vec[0].to_string(), arg_vec[1].to_string());
        }
    }
    result
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
        let resp = self
            .client
            .unwrap()
            .get(url)
            // .bearer_auth("cmlvdDp0end5R0FGZnVrbG9OWnlFZ1VSMDJ3")
            .basic_auth("riot", Some(self.token.as_str()))
            .send()
            .await?
            // .json::<HashMap<String, String>>()
            .text()
            .await?;

        let v: Value = serde_json::from_str(&resp)?;
        Ok(v)
    }

    pub async fn summoner(self) -> Result<Summoner> {
        let url = self.base_url + "/lol-summoner/v1/current-summoner";
        let resp = self
            .client
            .unwrap()
            .get(url)
            // .bearer_auth("cmlvdDp0end5R0FGZnVrbG9OWnlFZ1VSMDJ3")
            .basic_auth("riot", Some(self.token.as_str()))
            .send()
            .await?
            // .json::<HashMap<String, String>>()
            .text()
            .await?;

        let v: Summoner = serde_json::from_str(&resp)?;
        Ok(v)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Summoner {
    pub accountId: u32,
    pub displayName: String,
    internalName: String,
    nameChangeFlag: bool,
    percentCompleteForNextLevel: u32,
    privacy: String,
    profileIconId: u32,
    puuid: String,
    pub summonerId: u32,
    pub summonerLevel: u16,
    unnamed: bool,
    xpSinceLastLevel: u32,
    xpUntilNextLevel: u32,
    rerollPoints: RerollPoints,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct RerollPoints {
    currentPoints: u32,
    maxRolls: u8,
    numberOfRolls: u8,
    pointsCostToRoll: u32,
    pointsToReroll: u32,
}
