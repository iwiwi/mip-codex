//! 元モデルを変更せず、非交差行の証明を単独で出力する検証用入口。
use mip_codex::{model::Model,row_packing};
fn main(){let a:Vec<_>=std::env::args().collect();let m:Model=if a[1].ends_with(".json"){serde_json::from_reader(std::fs::File::open(&a[1]).unwrap()).unwrap()}else{mip_codex::mps::read(&a[1]).unwrap()};let end=std::time::Instant::now()+std::time::Duration::from_secs(1);println!("{}",serde_json::to_string(&row_packing::compute(&m,end)).unwrap());}
