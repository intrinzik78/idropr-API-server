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

#[cfg(test)]
pub mod test {
    use super::*;

    #[test]
    fn bounds_check() {
        type A = ActivityType;

        let paintball = A::Paintball;
        let gellyball = A::Gellyball;
        let go_karts = A::GoKarts;
        let water_park = A::WaterPark;
        let laser_tag = A::LaserTag;
        let board_games = A::BoardGames;
        let air_soft = A::AirSoft;
        let pickleball = A::Pickleball;
        let pool_table = A::PoolTable;
        let mini_golf = A::MiniGolf;
        let arcade_games = A::ArcadeGames;
        let ping_pong = A::PingPong;

        let paintball_test = A::from_u8(1).unwrap();
        let gellyball_test = A::from_u8(2).unwrap();
        let go_karts_test = A::from_u8(3).unwrap();
        let water_park_test = A::from_u8(4).unwrap();
        let laser_tag_test = A::from_u8(5).unwrap();
        let board_games_test = A::from_u8(6).unwrap();
        let air_soft_test = A::from_u8(7).unwrap();
        let pickleball_test = A::from_u8(8).unwrap();
        let pool_table_test = A::from_u8(9).unwrap();
        let mini_golf_test = A::from_u8(10).unwrap();
        let arcade_games_test = A::from_u8(11).unwrap();
        let ping_pong_test = A::from_u8(12).unwrap();
        let fail_low = A::from_u8(0);
        let fail_high = A::from_u8(13);

        assert_eq!(paintball, paintball_test);
        assert_eq!(gellyball, gellyball_test);
        assert_eq!(go_karts, go_karts_test);
        assert_eq!(water_park, water_park_test);
        assert_eq!(laser_tag, laser_tag_test);
        assert_eq!(board_games, board_games_test);
        assert_eq!(air_soft, air_soft_test);
        assert_eq!(pickleball, pickleball_test);
        assert_eq!(pool_table, pool_table_test);
        assert_eq!(mini_golf, mini_golf_test);
        assert_eq!(arcade_games, arcade_games_test);
        assert_eq!(ping_pong, ping_pong_test);

        assert!(fail_low.is_err());
        assert!(fail_high.is_err());
    }
}