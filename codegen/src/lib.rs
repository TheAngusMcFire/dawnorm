use proc_macro::TokenStream;
use syn::DeriveInput;

#[proc_macro_derive(Entity, attributes(key, key_noinsert, key_noinsert, key_noinsert_noupdate, noinsert_noupdate, noupdate, noinsert))]
pub fn postgres_entity(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).expect("Couldn't parse item");
    //let table_name = parse_table_attr(&ast);
    dawnorm_codegen_lib::generate_entity_code(&ast).into()
}