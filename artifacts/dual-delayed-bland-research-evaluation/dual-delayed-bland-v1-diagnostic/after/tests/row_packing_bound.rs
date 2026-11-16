use mip_codex::{model::Model,row_packing};
use std::time::{Instant,Duration};
#[test]
fn duplicate_column_cannot_be_counted_as_one_cover(){
 let m:Model=serde_json::from_value(serde_json::json!({"name":"duplicate","maximize":false,"offset":0.,"offset_enclosure":[0.,0.],"variables":[{"name":"x","lower":0.,"upper":1.,"integer":false,"objective":1.,"objective_enclosure":[1.,1.],"lower_enclosure":[0.,0.],"upper_enclosure":[1.,1.]}],"rows":[{"name":"twice","lower":1.,"upper":null,"lower_enclosure":[1.,1.],"exact_integral":true,"coefficients":[[0,1.],[0,1.]]}]})).unwrap();
 assert!(m.check(&[0.5],false,1e-9).is_ok());
 assert!(row_packing::compute(&m,Instant::now()+Duration::from_secs(1)).is_none());
}
