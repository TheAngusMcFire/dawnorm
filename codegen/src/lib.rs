use proc_macro::TokenStream;
use syn::DeriveInput;

use syn::{
    DeriveInput, Ident,
    Meta::{List, NameValue},
    NestedMeta::Meta,
};
fn get_mapper_meta_items(attr: &syn::Attribute) -> Option<Vec<syn::NestedMeta>> {
    if attr.path.segments.len() == 1 && attr.path.segments[0].ident == "entity" {
        match attr.parse_meta() {
            Ok(List(ref meta)) => Some(meta.nested.iter().cloned().collect()),
            _ => {
                panic!("declare table name: #[pg_mapper(table = \"foo\")]");
            }
        }
    } else {
        None
    }
}

fn get_lit_str<'a>(
    attr_name: Option<&Ident>,
    lit: &'a syn::Lit,
) -> ::std::result::Result<&'a syn::LitStr, ()> {
    if let syn::Lit::Str(ref lit) = *lit {
        Ok(lit)
    } else {
        if let Some(val) = attr_name {
            panic!("expected pg_mapper {:?} attribute to be a string", val);
        } else {
            panic!("expected pg_mapper attribute to be a string");
        }
        #[allow(unreachable_code)]
        Err(())
    }
}

fn parse_table_attr(ast: &DeriveInput) -> String {
    // Parse `#[pg_mapper(table = "foo")]`
    let mut table_name: Option<String> = None;

    for meta_items in ast.attrs.iter().filter_map(get_mapper_meta_items) {
        for meta_item in meta_items {
            match meta_item {
                // Parse `#[pg_mapper(table = "foo")]`
                Meta(NameValue(ref m)) if m.path.is_ident("table") => {
                    if let Ok(s) = get_lit_str(m.path.get_ident(), &m.lit) {
                        table_name = Some(s.value());
                    }
                }
                Meta(_) => {
                    panic!("unknown pg_mapper container attribute")
                }
                _ => {
                    panic!("unexpected literal in pg_mapper container attribute");
                }
            }
        }
    }

    table_name.expect("declare table name: #[pg_mapper(table = \"foo\")]")
}


#[proc_macro_derive(Entity, attributes(entity))]
pub fn postgres_entity(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).expect("Couldn't parse item");
    //let table_name = parse_table_attr(&ast);
    dawnorm_codegen_lib::generate_entity_code(&ast).into()
}