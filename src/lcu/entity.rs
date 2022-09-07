//! 所有接口结构体定义

#![allow(non_snake_case)]

use serde::{Deserialize, Serialize};

use std::collections::HashMap;

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
    internalName: String,
    nameChangeFlag: bool,
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





/// 返回结果 json数据集或是错误数据集
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum LCUResult<T, B> {
    Ok(T),
    Err(B),
}

/// 枚举所有lcu的json数据集
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum LCUpackage {
    Summoner(Summoner),
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