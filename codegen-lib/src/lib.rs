use proc_macro2::TokenStream;
use syn::{DataStruct, Item, DeriveInput, Data, ImplGenerics, TypeGenerics, WhereClause, Ident};


pub fn generate_entity_code(ast: &DeriveInput) -> TokenStream {
    let name = &ast.ident;

    let (impl_generics, ty_generics, where_clause) = &ast.generics.split_for_impl();

    let s = match ast.data {
        Data::Struct(ref s) => s,
        _ => panic!("Enums or Unions can not be mapped"),
    };

    let entity_trait = impl_entity_trait(s, name, impl_generics, ty_generics, where_clause);

    quote::quote! {
        #entity_trait
    }
}

fn impl_entity_trait(
    s: &DataStruct,
    name: &Ident,
    impl_generics: &ImplGenerics,
    ty_generics: &TypeGenerics,
    where_clause: &Option<&WhereClause>) -> Item {
    
        let fields = s.fields.iter().map(|field| {
            let ident = field.ident.as_ref().unwrap();
            let ty = &field.ty;
    
            let row_expr = format!(r##"{}"##, ident);
            quote::quote! {
                #ident:row.try_get::<&str,#ty>(#row_expr)?
            }
        });
    
        // **** todo use fallback attribute for pk **** \\
        let primary_key_name = "id";        
    
        let table_columns = s
            .fields
            .iter()
            .map(|field| {
                let ident = field
                    .ident
                    .as_ref()
                    .expect("Expected structfield identifier");
                format!(" {{table_name}}.{} ", ident)
            })
            .collect::<Vec<String>>()
            .join(", ");
    
        let columns = s
            .fields
            .iter()
            .map(|field| {
                let ident = field
                    .ident
                    .as_ref()
                    .expect("Expected structfield identifier");
                format!(" {} ", ident)
            })
            .collect::<Vec<String>>()
            .join(", ");
    
    let tokens = quote::quote! {
        impl #impl_generics dawnorm::Entity for #name #ty_generics #where_clause {
            fn from_row(row: tokio_postgres::row::Row) -> ::std::result::Result<Self, dawnorm::Error> {
                Ok(Self {
                    #(#fields),*
                })
            }

            fn sql_table_fields(table_name: &str) -> String {
                format!(#table_columns)
            }

            fn sql_fields() -> &'static str {
                #columns
            }
            fn entity_fields() -> Vec<dawnorm::EntityFieldDefinition> {
                todo!()
            }
            fn primary_key_name() -> &'static str {
                #primary_key_name
            }
        }
    };

    syn::parse_quote!(#tokens)
}


#[cfg(test)]
mod tests {
    use crate::generate_entity_code;

    
    #[test]
    pub fn test() {
        let ts = quote::quote!(
            pub struct SomeEntity {
                id: i32, 
                name: String,
                optiion: Option<String>
            }
        );

        let mut ast: syn::DeriveInput = syn::parse2(ts).expect("Couldn't parse item");

        let out = generate_entity_code(&ast);

        std::fs::write("/tmp/test.rs", format!("{}", out)).unwrap();
    }
}