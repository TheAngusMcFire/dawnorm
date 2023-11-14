use proc_macro2::{Span, TokenStream};
use syn::{Data, DataStruct, DeriveInput, Ident, ImplGenerics, Item, TypeGenerics, WhereClause, Attribute, Path, PathSegment};

/*
 * key_noinsert
 * key_noinsert_noupdate
 * key
 * noinsert
 * noupdate
 */


 #[derive(Default, Debug)]
struct EntityCodeGenData {
    key_fields: Vec<String>,
    insert_fields: Vec<String>,
    update_fields: Vec<String>,
    query_fields: Vec<String>
} 

fn get_codegen_data(ds: &DataStruct) -> EntityCodeGenData {
    let mut entity_data = EntityCodeGenData::default();

    for field in &ds.fields {
        let field_name = field.ident.as_ref().unwrap().to_string();
        let attrs = field.attrs.iter().filter_map(|x| 
            match x {
                Attribute {path: Path {segments, ..}, ..} => {
                    match &segments.first() {
                        Some(PathSegment {ident, .. }) => {
                            let ident_name = ident.to_string();
                            Some(ident_name)
                        },
                        _ => None
                    }
                }
            }
        ).collect::<Vec<String>>();

        entity_data.query_fields.push(field_name.clone());

        if attrs.iter().any(|x| x.contains("key")) {
            entity_data.key_fields.push(field_name.clone());
        }

        if !attrs.iter().any(|x| x.contains("noupdate")) {
            entity_data.update_fields.push(field_name.clone());
        }

        if !attrs.iter().any(|x| x.contains("noinsert")) {
            entity_data.insert_fields.push(field_name.clone());
        }
    }

    entity_data
}


pub fn generate_entity_code(ast: &DeriveInput) -> TokenStream {
    let name = &ast.ident;

    let (impl_generics, ty_generics, where_clause) = &ast.generics.split_for_impl();

    let s = match ast.data {
        Data::Struct(ref s) => s,
        _ => panic!("Enums or Unions can not be mapped"),
    };

    let codegen_data = get_codegen_data(&s);
    dbg!(&codegen_data);

    let entity_trait = impl_entity_trait(s, name, impl_generics, ty_generics, where_clause, &codegen_data);
    let entity_fields = impl_entity_fields(s, name, ty_generics, where_clause);

    quote::quote! {
        #entity_trait

        #entity_fields
    }
}

fn impl_entity_fields(
    s: &DataStruct,
    name: &Ident,
    ty_generics: &TypeGenerics,
    where_clause: &Option<&WhereClause>,
) -> TokenStream { 
    let new_name = Ident::new(&format!("{}Fields", name), Span::mixed_site());

    let fields = s.fields.iter().map(|field| {
        let ident = field.ident.as_ref().unwrap();

        let row_expr = format!(r##"{}"##, ident);
        quote::quote! {
            pub fn #ident() -> &'static str { #row_expr }
        }
    });

    let tokens = quote::quote! {
        pub struct #new_name #ty_generics #where_clause { }
        impl #new_name {
            #(#fields)*
        }
    };
    tokens
}

fn generate_args_list(len: usize, offset: usize) -> String {
    (offset..len+offset).map(|x| format!("${}", x + 1))
        .collect::<Vec<String>>().join(", ")
}

fn generate_key_constraint(keys: &[String], parm_offset: usize) -> String {
    format!("({}) = ({})", keys.join(", "), generate_args_list(keys.len(), parm_offset))
}

fn impl_entity_trait(
    s: &DataStruct,
    name: &Ident,
    impl_generics: &ImplGenerics,
    ty_generics: &TypeGenerics,
    where_clause: &Option<&WhereClause>,
    code_gen_data: &EntityCodeGenData
) -> Item {
    let fields = s.fields.iter().map(|field| {
        let ident = field.ident.as_ref().unwrap();
        let ty = &field.ty;
        let row_expr = format!(r##"{}"##, ident);
        quote::quote! {
            #ident:row.try_get::<&str,#ty>(#row_expr)?
        }
    });

    let insert_query = format!(
        "INSERT INTO {{}} ({}) VALUES ({}) RETURNING {};",
        code_gen_data.insert_fields.join(", "),
        generate_args_list(code_gen_data.insert_fields.len(), 0),
        code_gen_data.query_fields.join(", ")
    );
    let insert_parms = code_gen_data.insert_fields.iter()
        .map(|x| Ident::new(x, Span::mixed_site()))
        .map(|x| quote::quote!(self.#x));

    let update_query = format!(
        "UPDATE {{}} SET ({}) = ({}) WHERE {} RETURNING {};",
        code_gen_data.update_fields.join(", "),
        generate_args_list(code_gen_data.update_fields.len(), 0),
        generate_key_constraint(&code_gen_data.key_fields, code_gen_data.update_fields.len()),
        code_gen_data.query_fields.join(", ")
    );

    let update_parms = 
        [code_gen_data.update_fields.clone(), code_gen_data.key_fields.clone()].concat().into_iter()
        .map(|x| Ident::new(&x, Span::mixed_site()))
        .map(|x| quote::quote!(self.#x));

    let delete_query = format!(
        "DELETE FROM {{}} WHERE {}",
        generate_key_constraint(&code_gen_data.key_fields, 0)
    );

    let delete_parms = code_gen_data.key_fields.iter()
    .map(|x| Ident::new(x, Span::mixed_site()))
    .map(|x| quote::quote!(self.#x));

    let sql_fiels = code_gen_data.query_fields.join(", ");

    let tokens = quote::quote! {
        impl #impl_generics dawnorm::Entity for #name #ty_generics #where_clause {
            fn from_row(row: tokio_postgres::row::Row) -> ::std::result::Result<Self, dawnorm::Error> {
                Ok(Self {
                    #(#fields),*
                })
            }

            fn sql_fields() -> &'static str {
                #sql_fiels
            }

            fn get_insert_query(self, table_name: &str) -> (String, Vec<Box<dyn tokio_postgres::types::ToSql + Sync>>) {
                (format!(#insert_query, table_name), dawnorm::parms![#(#insert_parms),*])
            }

            fn get_update_query(self, table_name: &str) -> (String, Vec<Box<dyn tokio_postgres::types::ToSql + Sync>>) {
                (format!(#update_query, table_name), dawnorm::parms![#(#update_parms),*])
            }

            fn get_delete_query(&self, table_name: &str) -> (String, Vec<Box<dyn tokio_postgres::types::ToSql + Sync>>) {
                (format!(#delete_query, table_name), dawnorm::parms![#(#delete_parms),*])
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
            #[pg_mapper(table = "foo")]
            pub struct SomeEntity {
                #[key_noinsert_noupdate]
                id: i32,
                #[key]
                name: String
            }
        );

        let ast: syn::DeriveInput = syn::parse2(ts).expect("Couldn't parse item");

        dbg!(&ast.attrs);

        let out = generate_entity_code(&ast);

        std::fs::write("/tmp/test.rs", format!("{}", out)).unwrap();
    }
}
