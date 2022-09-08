//! 所有接口结构体定义

#![allow(non_snake_case)]

use serde::{Deserialize, Serialize};

use std::{collections::HashMap, fmt};

pub trait SetErrDefault {
    fn match_err_then(&mut self, err: LcuError);
}

/// 错误信息
/// {"errorCode": String("RPC_ERROR"), "httpStatus": Number(404), "implementationDetails": Object {}, "message": String("LOBBY_NOT_FOUND")}
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ErrorJson {
    errorCode: String,
    httpStatus: u32,
    implementationDetails: HashMap<String, String>,
    message: String,
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
    pub summonerId: u32,
    pub summonerLevel: u16,
    unnamed: bool,
    xpSinceLastLevel: u32,
    xpUntilNextLevel: u32,
    rerollPoints: RerollPoints,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct RerollPoints {
    currentPoints: u32,
    maxRolls: u8,
    numberOfRolls: u8,
    pointsCostToRoll: u32,
    pointsToReroll: u32,
}

impl SetErrDefault for Summoner {
    fn match_err_then(&mut self, err: LcuError) {
        match err {
            LcuError::JsonParseFailed => self.internalName = String::from("json解析错误"),
            LcuError::NoEntity => self.internalName = String::from("请开启游戏"),
            LcuError::Other => self.internalName = String::from("其他未知错误"),
            LcuError::NotAdmin => self.internalName = String::from("请用管理员权限运行"),
            LcuError::NotFind => self.internalName = String::from(""),
            LcuError::NotInitClient => self.internalName = String::from("未建立连接,请用管理员权限运行"),
        }
    }
}

/// 返回结果 json数据集或是错误数据集
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum LcuResult {
    Ok(LcuPackage),
    Err(LcuError),
}

/// 枚举所有lcu的json数据集
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum LcuPackage {
    Summoner(Summoner),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum LcuError {
    JsonParseFailed,
    NoEntity,
    Other,
    NotAdmin,
    NotFind,
    NotInitClient
}

impl std::error::Error for LcuError {}
#[allow(clippy::recursive_format_impl)]
impl fmt::Display for LcuError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

// pub fn parse_api_json<T, B>(resp: String) -> anyhow::Result<LCUResult<T, B>>
// where
//     for<'de> T: Deserialize<'de>,
//     for<'de> B: Deserialize<'de>, // T: Deserialize<'a>,
//                                   // B: Deserialize<'a>
// {
//     match serde_json::from_str::<T>(&resp) {
//         Ok(v) => Ok(LCUResult::Ok(v)),
//         Err(_) => match serde_json::from_str::<B>(&resp) {
//             Ok(v) => Ok(LCUResult::Err(v)),
//             Err(e) => Err(anyhow::Error::new(e)),
//         },
//     }
// }
