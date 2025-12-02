use serde::Deserialize;
use utoipa::ToSchema;

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
}