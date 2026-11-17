use mip_codex::{gomory::{self,Proof},mps};
use serde::Deserialize;
use std::io::{self,Read};
#[derive(Deserialize)]struct Input{model:String,proof:Proof}
fn main(){let args:Vec<_>=std::env::args().collect();if args.get(1).is_some_and(|v|v=="build-info"){println!("{}",serde_json::json!({"source_directory":env!("CARGO_MANIFEST_DIR"),"source_fingerprint":mip_codex::BUILD_SOURCE_FINGERPRINT,"library_source_directory":mip_codex::BUILD_SOURCE_DIRECTORY,"library_source_fingerprint":mip_codex::BUILD_SOURCE_FINGERPRINT}));return;}let mut text=String::new();io::stdin().read_to_string(&mut text).unwrap();let cases:Vec<Input>=serde_json::from_str(&text).unwrap();for i in cases{let m=mps::parse(std::io::Cursor::new(i.model)).unwrap();let row=gomory::derive(&m,&i.proof);if let Some(r)=&row{assert!(gomory::verify(&m,&i.proof,r));}println!("{}",serde_json::to_string(&row).unwrap());}}
