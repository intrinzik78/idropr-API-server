use actix_web::{HttpMessage, HttpRequest, Responder, web::{Data,Path,Query}};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    enums::{AuthContext,Error,Resource},
    traits::ToUser,
    types::{ApiErrorData, ApiResponse, AppState, business::{self, Location}, permissions::{UserPermissions, WereChecked}}};

type Result<T> = std::result::Result<T,Error>;

#[derive(Deserialize)]
pub struct GetReqPath {
    pub id:i64
}

#[derive(Deserialize)]
pub struct GetReqParams {
    pub nearest_zipcode:Option<String>,
    pub lat:Option<f32>,
    pub lon:Option<f32>
}

#[derive(Debug,Serialize,ToSchema)]
pub struct PrivateLocation {
    location_id:i64,
    business_id:i64,
    name:String,
    priority: u8,
    address_1:String,
    #[serde(skip_serializing_if = "Option::is_none")]
    address_2:Option<String>,
    city:String,
    state:String,
    zipcode:String,
    country:String,
}

impl PrivateLocation {
    pub fn transform(location:&Location) -> Self {
        // extract address from Location object
        let address= location.address();

        // extract the string from the option and take ownership
        let address_2 = match address.address_2() {
            Some(s) => Some(s.to_owned()),
            None => None
        };

        // format response
        PrivateLocation {
            location_id: location.id(),
            business_id: location.business_id(),
            name:        location.name().to_owned(),
            priority:    location.priority() as u8,
            address_1:   address.address_1().to_owned(),
            address_2:   address_2,
            city:        address.city().to_owned(),
            state:       address.state().to_owned(),
            zipcode:     address.zipcode().to_owned(),
            country:     address.country().to_owned()
        }
    }
}

#[derive(Debug,Serialize,ToSchema)]
pub struct PublicLocation {
    name:String,
    address_1:String,
    #[serde(skip_serializing_if = "Option::is_none")]
    address_2:Option<String>,
    city:String,
    state:String,
    zipcode:String,
    country:String,
}

impl PublicLocation {
    pub fn transform(location:&Location) -> Self {
        // extract address from Location object
        let address= location.address();
        
        let address_2 = match address.address_2() {
            Some(s) => Some(s.to_owned()),
            None => None
        };

        // format response
        PublicLocation {
            name: location.name().to_owned(),
            address_1: address.address_1().to_owned(),
            address_2: address_2,
            city:      address.city().to_owned(),
            state:     address.state().to_owned(),
            zipcode:   address.zipcode().to_owned(),
            country:   address.country().to_owned()
        }
    }
}

#[derive(ToSchema)]
pub struct LocationsGet;

impl LocationsGet {
    async fn private_logic(req: HttpRequest, path: Path<GetReqPath>, shared: Data<AppState>) -> Result<PrivateLocation> {
         // extract extensions
        let extensions = req.extensions();
        
        // extract auth_context from extensions
        let auth_context_opt:Option<&AuthContext> = extensions.get();

        // extract a user from auth context
        let user = auth_context_opt.to_user().ok_or(Error::MissingUserInAuthContext)?;

        let user_id = user.id();
        let permissions = user.permissions();
        
        // extract database
        let connection = shared.database();

        // extract location id from request path
        let location_id = path.id;

        // query database for location by id with private permissions
        let location = business::Location::read_self_by_id(location_id, user_id, &permissions, connection).await?;

        // format response as a PrivateLocation
        let private_location = PrivateLocation::transform(&location);

        Ok(private_location)
    }

    async fn public_logic(path: Path<GetReqPath>, shared: Data<AppState>) -> Result<PublicLocation> {
        // set permissions
        let resource = Resource::Locations;
        let permissions = UserPermissions::new().with_read_any(resource);

        // extract location id from request path
        let location_id = &path.id;
        
        // extract database connection
        let connection = shared.database();

        // query database for location by id with public permissions
        let location = business::Location::read_any_by_id(*location_id, &permissions, connection).await?;

        // format response as a PublicLocation
        let public_location = PublicLocation::transform(&location);

        Ok(public_location)
    }

    async fn public_nearest_logic(params:Query<GetReqParams>, shared: Data<AppState>) -> Result<Vec<PublicLocation>> {
        
        // extract zipcode from query params and return error if missing
        let zipcode = params
            .into_inner()
            .nearest_zipcode
            .ok_or(Error::MissingLocationQueryParam(String::from("zipcode")))?;

        // set permissions for the query [read_any]
        let resource = Resource::Locations;
        let permissions = UserPermissions::new()
            .with_read_any(resource);
        

        // extract database connection
        let connection = shared.database();

        // retreive the list of locations nearest the zipcode parameter
        let list = Location::read_any_nearest_by_zipcode(&zipcode, &permissions, connection).await?;

        // alloc
        let mut public_locations:Vec<PublicLocation> = Vec::new();

        // transform the records into PublicLocations
        for item in list.iter() {
            let public_location = PublicLocation::transform(item);
            public_locations.push(public_location);
        }
        
        Ok(public_locations)
    }

    /// returns public data about a single location by id
    pub async fn public_response(path: Path<GetReqPath>, shared: Data<AppState>) -> impl Responder {
        type E = Error;

        let query = Self::public_logic(path, shared).await;

        match query {
            Ok(r) => ApiResponse::default().with_data(r).with_code(200).ok(),
            Err(e) => {
                let error_opt = e.to_api_error_message();

                if let Some(d) = error_opt {
                    match e {
                        E::LocationRecordNotFoundById       => ApiResponse::<ApiErrorData>::default().with_code(404).with_data(d).error(),
                        E::InsufficientLocationPermissions  => ApiResponse::<ApiErrorData>::default().with_code(403).with_data(d).error(),
                        _ => ApiResponse::server_error().error()
                    }
                } else {
                    ApiResponse::server_error().error()
                }
            }
        }
    }

    /// returns private data about a single location by id
    pub async fn private_response(_permission:WereChecked, req: HttpRequest, path: Path<GetReqPath>, shared: Data<AppState>) -> impl Responder {
        type E = Error;

        let query = Self::private_logic(req, path, shared).await;

        match query {
            Ok(r) => ApiResponse::default().with_data(r).with_code(200).ok(),
            Err(e) => {
                let error_opt = e.to_api_error_message();

                if let Some(d) = error_opt {
                    match e {
                        E::LocationRecordNotFoundById       => ApiResponse::<ApiErrorData>::default().with_code(404).with_data(d).error(),
                        E::InsufficientLocationPermissions  => ApiResponse::<ApiErrorData>::default().with_code(403).with_data(d).error(),
                        _ => ApiResponse::server_error().error()
                    }
                } else {
                    ApiResponse::server_error().error()
                }
            }
        }
    }

    /// returns a list of public locations nearest to a zipcode provided in the query parameters
    pub async fn public_nearest_zipcode_response(params:Query<GetReqParams>, shared: Data<AppState>) -> impl Responder {
        let location_list = match Self::public_nearest_logic(params,shared).await {
            Ok(l) => l,
            Err(e) => {
                // log error here
                println!("{e}");
                
                // return a helpful error message where possible
                type E = Error;
                
                let d_opt = e.to_api_error_message();

                if let Some(d) = d_opt {
                    let response = match e {
                        E::MissingLocationQueryParam(_)     => ApiResponse::default().with_code(400).with_data(d).error(),
                        E::InsufficientLocationPermissions  => ApiResponse::default().with_code(403).with_data(d).error(),
                        _ =>                                   ApiResponse::server_error().error()
                    };
                    
                    return response
                } else {
                    return ApiResponse::server_error().error()
                }
            }
        };

        ApiResponse::default()
            .with_code(200)
            .with_data(location_list)
            .ok()
    }

    /// returns a list of public locations nearest to a zipcode provided in the query parameters
    pub async fn public_nearest_point_response(_params:Query<GetReqParams>, _shared: Data<AppState>) -> impl Responder {
        // let lat = params.lat.unwrap();
        // let lon = params.lon.unwrap();

        // println!("{},{}",lat,lon);
        
        ApiResponse::bad_request().error()
    }
}