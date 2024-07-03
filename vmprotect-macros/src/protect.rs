use darling::FromMeta;
use proc_macro2::{TokenStream};
use quote::quote;
use syn::{Block, parse_quote};

#[derive(Debug,FromMeta)]
pub struct ProtectArgs {
    #[darling(default)]
    lock: bool,
    #[darling(default)]
    virtualize: bool,
    #[darling(default)]
    mutate: bool,
    #[darling(default)]
    rename: Option<String>,
}

pub struct ProtectContext{
    args: ProtectArgs,
    body: syn::ItemFn,
}
impl ProtectContext{
    pub fn new(args: ProtectArgs, body: syn::ItemFn) -> Self{
        Self{
            args,
            body,
        }
    }

    pub fn render(mut self) -> syn::Result<TokenStream>{
        let mut body = self.body;
        let block = *body.block;
        if body.sig.asyncness.is_some(){
            return Err(syn::Error::new_spanned(body.sig.fn_token,"不支持异步函数"));
        }

        let name = self.args.rename.unwrap_or_else(||body.sig.ident.to_string());
        let name = name.as_bytes();
        let begin = match self.args {
            ProtectArgs { virtualize:true,mutate:false,lock:false,.. } => {
                //虚拟化
                quote!{
                    unsafe {vmprotect::vmprotect_sys::VMProtectBeginVirtualization(&[#(#name),*,0u8] as *const u8 as *const i8)};
                }
            },
            ProtectArgs { virtualize:true,mutate:false,lock:true,.. } => {
                //虚拟化+lock
                quote!{
                    unsafe {vmprotect::vmprotect_sys::VMProtectBeginVirtualizationLockByKey(&[#(#name),*,0u8] as *const u8 as *const i8)};
                }
            },
            ProtectArgs { virtualize:false,mutate:true,.. } => {
                //变异
                quote!{
                    unsafe {vmprotect::vmprotect_sys::VMProtectBeginMutation(&[#(#name),*,0u8] as *const u8 as *const i8)};
                }
            },
            ProtectArgs { virtualize:true,mutate:true,lock:false,.. } => {
                //变异+虚拟化
                quote!{
                    unsafe {vmprotect::vmprotect_sys::VMProtectBeginUltra(&[#(#name),*,0u8] as *const u8 as *const i8)};
                }
            },
            ProtectArgs { virtualize:true,mutate:true,lock:true,.. } => {
                //变异+虚拟化
                quote!{
                    unsafe {vmprotect::vmprotect_sys::VMProtectBeginUltraLockByKey(&[#(#name),*,0u8] as *const u8 as *const i8)};
                }
            },
            _=>{
                quote!{
                    unsafe {vmprotect::vmprotect_sys::VMProtectBegin(&[#(#name),*,0u8] as *const u8 as *const i8)};
                }
            }
        };
        let block:Block = parse_quote!({
            #begin
            unsafe { std::arch::asm!("nop") };

            let mut res  = #block;
            let ptr = &mut res as *mut _;

            unsafe { std::arch::asm!("nop {}", in(reg) ptr) };
            unsafe { vmprotect::vmprotect_sys::VMProtectEnd()};
            res
        });
        body.block = Box::new(block);
        let output = quote!{
            #body
        };
        Ok(output)
    }
}
