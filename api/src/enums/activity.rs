use serde::Deserialize;
use utoipa::ToSchema;

use crate::enums::Error;

#[repr(u8)]
#[derive(Clone,Debug,PartialEq,ToSchema,Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ActivityType {
    Paintball   = 1,
    Gellyball   = 2,
    GoKarts     = 3,
    WaterPark   = 4,
    LaserTag    = 5,
    BoardGames  = 6,
    AirSoft     = 7,
    Pickleball  = 8,
    PoolTable   = 9,
    MiniGolf    = 10,
    ArcadeGames = 11,
    PingPong    = 12
}

impl ActivityType {
    pub fn to_u8(self) -> u8 { self as u8 }

    pub fn from_u8(value: u8) -> Result<Self, Error> {
        Ok(match value {
            1  => Self::Paintball,
            2  => Self::Gellyball,
            3  => Self::GoKarts,
            4  => Self::WaterPark,
            5  => Self::LaserTag,
            6  => Self::BoardGames,
            7  => Self::AirSoft,
            8  => Self::Pickleball,
            9  => Self::PoolTable,
            10 => Self::MiniGolf,
            11 => Self::ArcadeGames,
            12 => Self::PingPong,
            _  => return Err(Error::ActivityTypeOutOfRange)
        })
    }
}