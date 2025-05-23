use rocket::{get, routes};
use rocket::post;
use rocket::response::{Response, Redirect};
use rocket::http::Status;
use rocket::http::Header;
use rocket::http::ContentType;
use std::num::Wrapping;
use regex;
use rocket::Config;
use rocket::serde::{Deserialize, Serialize};
use rocket::Responder;
use rocket::request::{self, Request, FromRequest};
use rocket::form::{Form, FromForm};
use rocket::figment::{Figment, providers::{Format, Toml}};
use rocket::{State, fairing::AdHoc};
use rocket::serde::json::Json;
use cargo_manifest::Manifest;
use rocket::http::RawStr;
use std::str::FromStr;
use serde_json::json;
use rocket::request::Outcome;
use serde_json::value::RawValue;
use std::collections::HashMap;
use toml::{Value, Table};
use rocket::response::content;
use toml::value::Array;
use std::sync::Arc;
use leaky_bucket::RateLimiter;
use toml::de::Error;
use std::time::Instant;
use leaky_bucket::AcquireOwned;
use std::io::Cursor;
use std::sync::Mutex;
use std::time::Duration;
use serde_yaml;

//use to_binary::{BinaryString,BinaryError};

enum Val {
    Int(isize),
    Float(f64),
}

use Val::*;

pub fn xor(a : &Vec<u8>, b: &Vec<u8>) -> Vec<u8>{
   let c =  a.iter()
     .zip(b.iter())
     .map(|(&x1, &x2)| x1 ^ x2)
     .collect();

     c
}


#[get("/")]
pub fn index() -> &'static str {
    "Hello, bird!"
}


#[get("/-1/seek")]
pub async fn seek() -> Redirect {
    Redirect::found("https://www.youtube.com/watch?v=9Gc4QTqslN4")
}

#[get("/2/dest?<from>&<key>")]
pub async fn calculate_dest(from: &str, key: &str) -> String {
   let parsed_from: Vec<u8> = from.split('.').map(|s| s.parse::<u8>().unwrap()).collect();
   let parsed_key: Vec<u8> = key.split('.').map(|k| k.parse::<u8>().unwrap()).collect();
   let mut answer: Vec<u8> = Vec::new();
   for i in 0..parsed_key.len() {
       let overflowing_addition = Wrapping(parsed_from[i]) + Wrapping(parsed_key[i]);
       answer.push(overflowing_addition.0);
       println!("pushed {:?}", &overflowing_addition.0);
       }
   format!("{}.{}.{}.{}",answer[0],answer[1],answer[2],answer[3])
}

#[get("/2/key?<from>&<to>")]
pub async fn calculate_key(from: &str, to: &str) -> String {
   let parsed_from: Vec<u8> = from.split('.').map(|s| s.parse::<u8>().unwrap()).collect();
   let parsed_to: Vec<u8> = to.split('.').map(|k| k.parse::<u8>().unwrap()).collect();
   let mut answer: Vec<u8> = Vec::new();
   for i in 0..parsed_to.len() {
       let overflowing_addition = Wrapping(parsed_to[i]) - Wrapping(parsed_from[i]);
       answer.push(overflowing_addition.0);
       println!("pushed {:?}", &overflowing_addition.0);
       }
   format!("{}.{}.{}.{}",answer[0],answer[1],answer[2],answer[3])
}

pub fn create_addr(parsed_thing: Vec<&str>) -> Vec<usize>{
   let mut result: Vec<usize> = Vec::new();
   let len_ipv6 = parsed_thing.len();
   let less_than_len = 9 as usize - len_ipv6;
   for num in &parsed_thing {
       if *num == "" {
           for l in 0..less_than_len {
	     result.push(0000);
             }
	   }
       else {
           result.push(usize::from_str_radix(num, 16).unwrap());
	   }
	   }
    result
}

#[get("/2/v6/dest?<from>&<key>")]
pub async fn calculate_v6dest(from: &str, key: &str) -> String {
   //let re = regex::Regex::new(r"::|:").unwrap();
   let parsed_from: Vec<&str> = from.split(':').collect();
   let parsed_key: Vec<&str> = key.split(':').collect();
   let len_ipv6 = parsed_from.len();
   let less_than_len = 9 as usize - len_ipv6;
   let mut answer: Vec<_> = Vec::new();
   let parsed_result = create_addr(parsed_from);
   let key_result = create_addr(parsed_key);
    for i in 0..8 {
       let overflowing_xor = Wrapping(parsed_result[i]) ^ Wrapping(key_result[i]);
       answer.push(u32::from_str_radix(&overflowing_xor.0.to_string(), 2).unwrap());
       println!("pushed {:?}", &overflowing_xor.0);
       }
   format!("{}.{}.{}.{}",answer[0],answer[1],answer[2],answer[3])
}

#[get("/2/v6/key?<from>&<to>")]
pub async fn calculate_v6key(from: &str, to: &str) -> String {
   let re = regex::Regex::new(r"::|:").unwrap();
   let mut parsed_from: Vec<_> = Vec::new();
   let mut parsed_to: Vec<_> = Vec::new();
   let mut answer: Vec<i32> = Vec::new();
   for part in re.split(from) {
       parsed_from.push(part.parse::<i32>().unwrap());
       }
   for item in re.split(to) {
       parsed_to.push(item.parse::<i32>().unwrap());
       }
   for i in 0..parsed_to.len() {
       let overflowing_addition = Wrapping(parsed_to[i]) ^ Wrapping(parsed_from[i]);
       answer.push(overflowing_addition.0);
       println!("pushed {:?}", &overflowing_addition.0);
       }
   format!("{}.{}.{}.{}",answer[0],answer[1],answer[2],answer[3])
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct Orders {
    item: String,
    quantity: u32,
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct Metadata {
    metadata: Vec<Orders>,
}

//impl From<Vec<Orders>> for Metadata {
//    fn from(metadata: Vec<Orders>) -> Self {
//       Metadata {
//          metadata
//	  }
//    }
//}

#[derive(PartialEq, Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct Package {
    name: String,
    authors: String,
    keywords: Option<Vec<String>>,
    metadata: Metadata,
    #[serde(rename = "rust-version")]
    rustversion: String,
}
        
#[derive(PartialEq, Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
#[derive(Responder)]
enum MapsErrorResponse {
    #[response(status = 204, content_type = "json")]
    A(String),
    #[response(status = 400, content_type = "json")]
    B(String),
    #[response(status = 416, content_type = "json")]
    C(String),
}

fn parse_string(s: Option<&str>) -> Option<Val> {
    println!("s is {:?}", &s);
    if let Ok(i) = s?.parse() {
       Some(Int(i))
    } else {
       None
    }
}

pub trait ContentTypeExt {
     fn is_toml(&self) -> bool;
     fn is_yaml(&self) -> bool;
}

impl ContentTypeExt for ContentType {
    fn is_toml(&self) -> bool {
      self.top() == "application" && self.sub() == "toml"
      }
    fn is_yaml(&self) -> bool {
      self.top() == "application"  && self.sub() == "yaml"
      }
}

struct ContentTypeGuard;


#[rocket::async_trait]
impl<'r> FromRequest<'r> for ContentTypeGuard {
   type Error = ();
   async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
      match request.content_type() {
         Some(ct) if ct.is_yaml() || ct.is_json() || ct.is_toml() => Outcome::Success(ContentTypeGuard),
	 _ => rocket::outcome::Outcome::Forward(Status::UnsupportedMediaType),
	 }
     }
}

fn deal_with_yaml(yam: serde_yaml::Value) -> String {
   let mut return_string = String::new();
   if let serde_yaml::Value::Mapping(package) = yam.get("package").unwrap() {
        if let Some(serde_yaml::Value::Mapping(metadata)) = package.get(&serde_yaml::Value::String("metadata".to_string())) {
            if let Some(serde_yaml::Value::Sequence(orders)) = metadata.get(&serde_yaml::Value::String("orders".to_string())) {
                for order in orders {
                    if let serde_yaml::Value::Mapping(order_map) = order {
                        let item = order_map.get("item").and_then(|v| v.as_str()).unwrap_or("null");
                        let quantity = order_map.get("quantity").and_then(|v| v.as_u64()).unwrap_or(0);
			if quantity != 0 {
			    return_string.push_str(&format!("{}: {}\n",item,quantity));
			    }
                    }
                }
            }
        }
    }
    return_string
}

fn deal_with_json(json: &str) -> String {
   let mut return_string = String::new();
   let package: Package = serde_json::from_value(json.into()).unwrap(); //parsed["package"].clone()).unwrap();
   let contains_christmas_2024 = match package.keywords {
        Some(ref keywords) => keywords.contains(&"Christmas 2024".to_string()),
        None => false,
    };
   println!("contains christmas {:?}", &contains_christmas_2024);
   if contains_christmas_2024 == false {
         return "false".to_string()
	 }
   println!("package is {:?}", &package);
   let orders: Vec<(String, u32)> = package.metadata.metadata.iter()
        .map(|order| (order.item.clone(), order.quantity))
	.collect();
   for (item, quantity) in orders {
       if quantity > 0 {
          return_string.push_str(&format!("{}: {}\n",item, quantity));
	  }
   }
   
   return_string
}
	  

fn is_json_check(input: &str) -> bool {
    serde_json::from_str::<serde_json::Value>(input).is_ok()
}

#[rocket::post("/5/manifest", data="<manifest>")]
pub async fn gift_manifest(manifest: &str, _guard: ContentTypeGuard) -> Result<String, MapsErrorResponse> {
    let yam = match serde_yaml::from_str::<serde_yaml::Value>(&manifest) {
       Ok(yam) => {
                    let result = deal_with_yaml(yam.clone());
		    println!("just printing yam {:?}", &yam);
		    if let serde_yaml::Value::Mapping(package) = yam.get("package").unwrap() {
		        println!("package is here in this bit {:?}", &package);
			if let Some(keywords) = package.get("keywords") {
			    println!("so keywords is {:?}", &keywords);
			    if let Some(keywords_arr) = keywords.as_sequence() {
			        if keywords_arr.iter().any(|v| v.as_str() != Some("Christmas 2024")) {
				     return Err(MapsErrorResponse::B("Magic keyword not provided".to_string()));
				     }
				}
		            }
			else {
			    println!("no keywords erroring");
			    return Err(MapsErrorResponse::B("Magic keyword not provided".to_string()));
			    }
		        if let Some(rustversion) = package.get("rust-version") {
			       println!("this is rustversion {:?}", &rustversion);
			       //let version = rustversion.get("rust-version").and_then(|v| v.as_bool()).unwrap_or(false);
			       //println!("this is version", &version);
			       if *rustversion == serde_yaml::Value::Bool(true) {
			             return Err(MapsErrorResponse::B("Invalid manifest".to_string()));
				}
                            
			 }
	            }
		    println!("this result {:?}", &result);
		    return Ok(result.trim().to_string());
		    }
       Err(_) =>  "".to_string(),
       };
    if is_json_check(manifest) {
        let result = deal_with_json(&manifest);
	if result == "false" {
	   return Err(MapsErrorResponse::B("Invalid manifest".to_string()));
           }
	else {
	     return Ok(result.trim().to_string());
	     }
	}
    println!("yam is {:?}", &yam);
    let mani: Table = toml::from_str(manifest).map_err(|_| MapsErrorResponse::B("Invalid TOML format".to_string()))?;
    if manifest.contains("package.metadata.orders = []") {
        return Err(MapsErrorResponse::A("orders noted".to_string()));
	}
    println!("heres the mani {:?}", &mani);
    let j: usize = 0;
    let mut i: usize = 0;
    let mut return_string = String::new();
    println!("just before the metadata line");
    let package = mani.get("package").ok_or_else(|| MapsErrorResponse::B("missing package section".to_string()))?;
    let package_keywords = &package.get("keywords").and_then(|keywords| keywords.as_array());
    let mut flag: u8 = 0;
    for keyw in package_keywords {
        let christmas_check = keyw;
	if christmas_check.contains(&toml::Value::String("Christmas 2024".to_string())) {
	   flag = 1;
	   }
	}
    println!("package name is {:?}", &package.get("name"));
    println!("flag is {:?}", &flag);
    if flag == 0 && package.get("name").and_then(toml::Value::as_str) != Some("chig-bungus") {
          println!("so the package name is {:?}", &package.get("name"));
          return Err(MapsErrorResponse::B("Magic keyword not provided".to_string()));
          }
    else if flag == 0 && package.get("name").and_then(toml::Value::as_str) == Some("chig-bungus") {
          println!("in the correct chig bungus");
          let workspace = match mani.get("workspace").and_then(|workspace| workspace.get("resolver")) {
	       Some(Value::Integer(resolver)) => resolver.to_string(),
	       Some(Value::String(resolver)) => resolver.clone(),
	       _ => "".to_string(),
	       };
	  if workspace == "2".to_string() {
	      return Err(MapsErrorResponse::B("Magic keyword not provided".to_string()));
	      }
	  }
    let metadata = package.get("metadata").ok_or_else(|| MapsErrorResponse::B("Invalid manifest".to_string()))?;
    println!("ok passed the metadata line");
		    //.and_then(|meta| meta.get("orders")).and_then(|orders| orders.as_array());
    //let orders = metadata.get("orders").ok_or_else(|| { MapsErrorResponse::B("missing orders section".to_string())})?;
    if !metadata.get("orders").is_some() {
        if metadata.get("stuff").is_some() {
	     return Err(MapsErrorResponse::A("no orders but there is stuff".to_string()));
	     }
	else {
           println!("its not some");
           return Err(MapsErrorResponse::B("orders is missing".to_string()));
	}
    }
    let orders = metadata
              .get("orders")
	      .and_then(|orders| orders.as_array());
    match orders {
        Some(orders) if orders.is_empty() => {
	       println!("in the empty order section");
	       return Err(MapsErrorResponse::A("orders array is empty".to_string()));
	       }
	Some(orders) => {
        for order in orders {
            println!("here is an order {:?}", &order);
            if let toml::Value::Table(order_table) = order {
               let item = order_table.get("item").and_then(|v| v.as_str()).unwrap_or("");
	       println!("this is an item {:?}", &item);
	       let quantity = match order_table.get("quantity") {
	           Some(toml::Value::Integer(q)) => *q,
		   Some(toml::Value::Float(q)) => 0,
		   Some(toml::Value::String(_)) => {
		       return Err(MapsErrorResponse::A("invalid quantity".to_string()));
		       }
		   Some(_) => {
		      return Err(MapsErrorResponse::B("invalid quantity".to_string()));
		      }
		   None => 0,
		   };
	       println!("this is a quantity {:?}", &quantity);
	       if item.is_empty() {
	          return Err(MapsErrorResponse::A("Invalid item in order".to_string()));
	          }
	       let quantity = match quantity {
	          q if q >= 0 => q,
		  _ => return Err(MapsErrorResponse::A("invalid quantity type".to_string())),
		  };
	       if quantity == 0 {
	          continue;
	          //return Err(MapsErrorResponse::A("Invalid quantity in order".to_string()));
	          }
	       return_string.push_str(&format!("{}: {}\n",item,quantity));
	       }
	       else {
	          return Err(MapsErrorResponse::A("Invalid order format".to_string()));
	          }
               }
	       return Ok(return_string.trim().to_string());
	 }
	 None => Err(MapsErrorResponse::B("orders is not valid".to_string())),
        }
}



#[derive(PartialEq, Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
#[derive(Responder)]
enum MilkResponse {
    #[response(status = 429, content_type = "json")]
    A(String),
}


//pub struct MilkClient {
//    limiter: RateLimiter,
//    acquire: Option<AcquireOwned>,  
//}

//impl Future for MilkClient {
//   type Output = ();
//   fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
//        let mut this = self.project();

//        loop {
//            if let Some(acquire) = this.acquire.as_mut().as_pin_mut() {
//                futures::ready!(acquire.poll(cx));
//                return Poll::Ready(());
//            }
//
//            this.acquire.set(Some(this.limiter.clone().acquire_owned(1000)));
//        }
//    }
//}


 
#[rocket::post("/9/milk")]
async fn get_milk(bucket: &State<Arc<RateLimiter>>) -> Result<String, MilkResponse> {
    if bucket.try_acquire(1) {
        Ok("Milk withdrawn\n".to_string())
	}
    else {
	Err(MilkResponse::A("No milk available\n".to_string()))
	}
}


#[shuttle_runtime::main]
pub async fn main() -> shuttle_rocket::ShuttleRocket {
    let bucket = RateLimiter::builder().max(5).initial(5).refill(1).interval(Duration::from_millis(1000)).build();
    let bucket = Arc::new(bucket);
    let rocket = rocket::build().manage(bucket).mount("/", routes![gift_manifest, get_milk, index, calculate_dest, calculate_key, calculate_v6dest, calculate_v6key, seek]);
    Ok(rocket.into())
}