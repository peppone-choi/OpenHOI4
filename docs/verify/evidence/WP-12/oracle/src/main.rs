use std::io::{self,BufRead};
fn main(){
 for line in io::stdin().lock().lines(){
  let text=line.unwrap();
  let bytes=(0..text.len()).step_by(2).map(|n|u8::from_str_radix(&text[n..n+2],16).unwrap()).collect::<Vec<_>>();
  println!("{}",oh_proto::decode_server(&bytes).is_ok());
 }
}
