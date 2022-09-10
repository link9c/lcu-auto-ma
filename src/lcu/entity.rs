//! 所有接口结构体定义

#![allow(non_snake_case)]

use serde::{Deserialize, Serialize};

use std::fmt;

///
///
///
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SendInfo {
    #[serde(alias = "type")]
    pub _type: String,
    pub fromId: String,
    pub fromSummonerId: String,
    pub isHistorical: bool,
    pub timestamp: String,
    pub body: String,
}

impl Default for SendInfo {
    fn default() -> Self {
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        Self {
            _type: String::from("chat"),
            fromId: Default::default(),
            fromSummonerId: Default::default(),
            isHistorical: false,
            timestamp: now.to_string(),
            body: Default::default(),
        }
    }
}

///马的优良评价
///
///
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq, PartialOrd)]
pub struct HorseInfo {
    pub deaths: u32,
    pub kills: u32,
    pub assists: u32,
    pub win: u32,
    pub defeat: u32,
    pub favhero: String,
    pub user: String,

    pub summonerId: u64,
}
#[allow(clippy::derive_ord_xor_partial_ord)]
impl Ord for HorseInfo {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.win + self.defeat > 15 {
            if self.win_rate() > other.win_rate() {
                std::cmp::Ordering::Greater
            } else if self.win_rate() == other.win_rate() {
                if self.KDA() > other.KDA() {
                    std::cmp::Ordering::Greater
                } else {
                    std::cmp::Ordering::Less
                }
            } else {
                std::cmp::Ordering::Less
            }
        } else if self.KDA() > other.KDA() {
                std::cmp::Ordering::Greater
            } else {
                std::cmp::Ordering::Less
            }
        
    }
}

impl HorseInfo {
    fn kill_avg(&self) -> u32 {
        self.kills / (self.win + self.defeat)
    }

    fn deaths_avg(&self) -> u32 {
        self.deaths / (self.win + self.defeat)
    }

    fn assists_avg(&self) -> u32 {
        self.assists / (self.win + self.defeat)
    }
    pub fn win_rate(&self) -> f32 {
        self.win as f32 / (self.win as f32 + self.defeat as f32)
    }

    pub fn KDA(&self) -> f32 {
        (((self.kill_avg() + self.assists_avg()) / self.deaths_avg()) * 3) as f32
    }

    pub fn text(&self) -> String {
        format!(
            "{0} 胜{1}/输{2}(胜率{3:.2}) 场均击杀{6}/助攻{7}/死亡{8} KDA:{5:.2}  常用英雄:{4}",
            self.user,
            self.win,
            self.defeat,
            self.win_rate(),
            self.favhero,
            self.KDA(),
            self.kill_avg(),
            self.assists_avg(),
            self.deaths_avg()
        )
    }
}

///
///
///
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MatchHistory {
    pub accountId: u32,
    pub games: Games,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Games {
    pub games: Vec<Games2>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Games2 {
    pub participants: Vec<Participants>,
    pub queueId: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Participants {
    pub championId: u32,
    pub stats: Stats,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Stats {
    pub deaths: u32,
    pub kills: u32,
    pub assists: u32,
    pub win: bool,
}

/// 账号信息
///
/// 获取姓名,id,等级等信息
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Summoner {
    pub accountId: u32,
    pub displayName: String,
    pub internalName: String,
    // nameChangeFlag: bool,
    percentCompleteForNextLevel: u32,
    privacy: String,
    profileIconId: u32,
    pub puuid: String,
    pub summonerId: u64,
    pub summonerLevel: u32,
    unnamed: bool,
    xpSinceLastLevel: u32,
    xpUntilNextLevel: u32,
}

///
///
///
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GameSession {
    pub actions: Vec<Vec<Actions>>,
    pub myTeam: Vec<MyTeam>,
    pub chatDetails: ChatDetails,
    pub theirTeam: Vec<TheirTeam>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Actions {
    actorCellId: u32,
    championId: u32,
    completed: bool,
    pub id: u32,
    isAllyAction: bool,
    pub isInProgress: bool,
}
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct MyTeam {
    assignedPosition: String,
    cellId: u32,
    championId: u32,
    pub summonerId: u64,
    team: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct TheirTeam {
    assignedPosition: String,
    cellId: u32,
    championId: u32,
    pub summonerId: u64,
    team: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ChatDetails {
    pub chatRoomName: String,
    chatRoomPassword: Option<String>,
}

///
///
///
pub fn match_err_then(err: LcuError) -> String {
    match err {
        LcuError::JsonParseFailed => String::from("json解析错误"),
        LcuError::NoEntity => String::from("请开启游戏"),
        LcuError::Other => String::from("其他未知错误"),
        LcuError::NotAdmin => String::from("请用管理员权限运行"),
        LcuError::NotFind => String::from(""),
        LcuError::NotInitClient => String::from("未建立连接,请用管理员权限运行"),
    }
}

/// 返回结果 json数据集或是错误数据集
///
///
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum LcuResult {
    Ok(LcuPackage),
    Err(LcuError),
}

/// 枚举所有lcu的json数据集
///
///
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum LcuPackage {
    Summoner(Summoner),
    Status(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum LcuError {
    JsonParseFailed,
    NoEntity,
    Other,
    NotAdmin,
    NotFind,
    NotInitClient,
}

impl std::error::Error for LcuError {}
#[allow(clippy::recursive_format_impl)]
impl fmt::Display for LcuError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}
