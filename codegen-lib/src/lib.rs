use proc_macro2::{Span, TokenStream};
use syn::{Data, DataStruct, DeriveInput, Ident, ImplGenerics, Item, TypeGenerics, WhereClause};

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
    where_clause: &Option<&WhereClause>,
) -> Item {
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
    let primary_key_name_ident = Ident::new(primary_key_name, Span::mixed_site());

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

    let all_columns = s
        .fields
        .iter()
        .map(|field| {
            let ident = field
                .ident
                .as_ref()
                .expect("Expected structfield identifier");
            format!("{}", ident)
        })
        .collect::<Vec<String>>();

    let columns = all_columns.join(", ");

    let cols_without_pk = all_columns
        .iter()
        .filter(|x| *x != primary_key_name)
        .map(|x| x.into())
        .collect::<Vec<String>>();

    let insert_args = (0..cols_without_pk.len())
        .map(|x| format!("${}", x + 1))
        .collect::<Vec<String>>()
        .join(", ");

    let combined_cols_without_pk = cols_without_pk.iter().map(|x| {
        let i = Ident::new(x, Span::mixed_site());
        quote::quote!(self.#i)
    });

    let combined_cols_without_pk_for_update = combined_cols_without_pk.clone();

    let insert_query = format!(
        "INSERT INTO {{}} ({}) VALUES ({}) RETURNING {};",
        cols_without_pk.join(", "),
        insert_args,
        columns
    );

    let update_query = format!(
        "UPDATE {{}} SET ({}) = ({}) WHERE {} = ${} RETURNING {};",
        cols_without_pk.join(", "),
        insert_args,
        primary_key_name,
        cols_without_pk.len() + 1,
        columns
    );

    let delete_query = format!(
        "DELETE FROM {{}} WHERE {} = $1;",
        primary_key_name
    );

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

            fn get_insert_query(self, table_name: &str) -> (String, Vec<Box<dyn tokio_postgres::types::ToSql + Sync>>) {
                (format!(#insert_query, table_name), dawnorm::parms![#(#combined_cols_without_pk),*])
            }

            fn get_update_query(self, table_name: &str) -> (String, Vec<Box<dyn tokio_postgres::types::ToSql + Sync>>) {
                (format!(#update_query, table_name), dawnorm::parms![#(#combined_cols_without_pk_for_update),* , self.#primary_key_name_ident])
            }

            fn get_delete_query(&self, table_name: &str) -> (String, Vec<Box<dyn tokio_postgres::types::ToSql + Sync>>) {
                (format!(#delete_query, table_name), dawnorm::parms![self.#primary_key_name_ident.clone()])
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
                optiion: Option<String>,
            }
        );

        let ast: syn::DeriveInput = syn::parse2(ts).expect("Couldn't parse item");

        let out = generate_entity_code(&ast);

        std::fs::write("/tmp/test.rs", format!("{}", out)).unwrap();
    }
}
