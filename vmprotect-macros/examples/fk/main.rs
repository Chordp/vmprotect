use std::fmt::Debug;
use vmprotect_macros::protected;
#[protected]
fn sss<T:Debug>(sb:T){
    println!("{:#?}",sb);
}
fn main(){
    sss(1);
}