use rocket::{get, routes};
use rocket::post;
use rocket::response::Redirect;
use rocket::http::Status;
use rocket::http::Header;
use std::num::Wrapping;
use regex;
use rocket::Config;
use rocket::serde::{Deserialize, Serialize};
use rocket::Responder;
use rocket::form::{Form, FromForm};
use rocket::figment::{Figment, providers::{Format, Toml}};
use rocket::{State, fairing::AdHoc};
use rocket::serde::json::Json;
use cargo_manifest::Manifest;
use rocket::http::RawStr;
use std::str::FromStr;
use serde_json::json;
use serde_json::value::RawValue;
use std::collections::HashMap;
use toml::{Value, Table};
use rocket::response::content;
use toml::value::Array;
use toml::de::Error;

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

impl From<Vec<Orders>> for Metadata {
    fn from(metadata: Vec<Orders>) -> Self {
       Metadata {
          metadata
	  }
    }
}

#[derive(PartialEq, Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct Package {
    name: String,
    authors: String,
    keywords: String,
    metadata: Metadata,
}
        
#[derive(PartialEq, Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
#[derive(Responder)]
enum MapsErrorResponse {
    #[response(status = 204, content_type = "json")]
    A(String),
}

fn parse_string(s: Option<&str>) -> Option<Val> {
    println!("s is {:?}", &s);
    if let Ok(i) = s?.parse() {
       Some(Int(i))
    } else {
       None
    }
}


#[rocket::post("/5/manifest", format="application/toml", data="<manifest>")]
pub async fn gift_manifest(manifest: &str) -> Result<String, MapsErrorResponse> {
    let mani: Table = toml::from_str(manifest).unwrap();
    println!("heres the mani {:?}", &mani);
    let j: usize = 0;
    let mut i: usize = 0;
    let mut return_string = String::new();
    if let Some(Value::Array(orders)) = mani.get("package")
                             .and_then(|pkg| pkg.get("metadata"))
			     .and_then(|metadata| metadata.get("orders"))
			 {
			   for (index, order) in orders.iter().enumerate() {
			       if let Value::Table(order_table) = order {
			           let item = order_table.get("item").and_then(|v| v.as_str()).unwrap_or("");
				   let quantity = order_table.get("quantity").and_then(|v| v.as_integer()).unwrap_or(0);
				   if item.is_empty() {
				      println!("item is empty");
				      }
				   else {
				      println!("order is valid {:?}", &item);
				      }
				   if quantity == 0 {
				      println!("quantity is not valid {:?}", &quantity);
				      }
				   else {
				      println!("quantity is {:?}", &quantity);
				      }
                               }
			       }
			       }
	else {
	    return Err(MapsErrorResponse::A("error".to_string()));
	    }
        let mut newmanifest: Value = mani["package"]["metadata"]["orders"].clone();
	while newmanifest.clone().get_mut(i) != None {
              //println!("here it is {:?}", &manifest.unwrap().get_mut(i));
              let mut thing = toml::Value::try_from(&newmanifest.get_mut(i)).unwrap();
              //println!("get manu {}", &thing["item"].to_string());
              //println!("get quant {:?}", &thing["quantity"].as_number().unwrap());
	      let nums = thing.get("quantity").and_then(|val| val.as_integer()).unwrap_or(0);
              if nums == 0 {
	           ()
                   //return Err(MapsErrorResponse::A("error".to_string()));
	           }
              else {
                   let result = format!("{}: {}",&thing["item"].as_str().unwrap(),&nums);
                   return_string.push_str(&format!("{}\n",&result));
	       }
	      i+=1;
      	      }
    let answer = return_string.trim().to_string();
    if answer == "" {
       return Err(MapsErrorResponse::A("error".to_string()));
       }
    println!("this is exact return string {:?}", &return_string.trim());
    Ok(answer)
}

#[shuttle_runtime::main]
pub async fn main() -> shuttle_rocket::ShuttleRocket {
    let rocket = rocket::build().mount("/", routes![gift_manifest, index, calculate_dest, calculate_key, calculate_v6dest, calculate_v6key, seek]);
    Ok(rocket.into())
}