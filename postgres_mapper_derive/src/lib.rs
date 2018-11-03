extern crate quote;
extern crate proc_macro;
#[macro_use]
extern crate syn;

use proc_macro::TokenStream;
use quote::Tokens;

use syn::DeriveInput;
use syn::Meta::{List, NameValue};
use syn::NestedMeta::{Literal, Meta};
use syn::Data::*;

use syn::{Fields, Ident};

#[proc_macro_derive(PostgresMapper, attributes(pg_mapper))]
pub fn postgres_mapper(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    impl_derive(&ast)
        .parse()
        .expect("Error parsing postgres mapper tokens")
}

fn impl_derive(ast: &DeriveInput) -> Tokens {
    #[allow(unused_mut)]
    let mut tokens = Tokens::new();



    #[allow(unused_variables)]
    let fields: &Fields = match ast.data {
        Struct(ref s) => {
            &s.fields
        },
        Enum(ref u) => {panic!("Enums can not be mapped")},
        Union(ref u) => {panic!("Unions can not be mapped")},
    };

    #[allow(unused_variables)]
    let table_name = parse_table_attr(&ast);

    #[cfg(feature = "postgres-support")]
    {
        impl_from_row(&mut tokens, &ast.ident, &fields);
        impl_from_borrowed_row(&mut tokens, &ast.ident, &fields);

        #[cfg(feature = "postgres-mapper")]
        {
            impl_postgres_mapper(&mut tokens, &ast.ident, &fields, &table_name);
        }
    }

    #[cfg(feature = "tokio-postgres-support")]
    {
        impl_tokio_from_row(&mut tokens, &ast.ident, &fields);
        impl_tokio_from_borrowed_row(&mut tokens, &ast.ident, &fields);

        #[cfg(feature = "postgres-mapper")]
        {
            impl_tokio_postgres_mapper(&mut tokens, &ast.ident, &fields, &table_name);
        }
    }

    tokens
}

#[cfg(feature = "postgres-support")]
fn impl_from_row(t: &mut Tokens, struct_ident: &Ident, fields: &Fields) {
    t.append(format!("
impl<'a> From<::postgres::rows::Row<'a>> for {struct_name} {{
    fn from(row: ::postgres::rows::Row<'a>) -> Self {{
        Self {{", struct_name=struct_ident));

    for field in fields {
        let ident = field.ident.clone().expect("Expected structfield identifier");

        t.append(format!("
            {0}: row.get(\"{0}\"),", ident));
    }

    t.append("
        }
    }
}");
}

#[cfg(feature = "postgres-support")]
fn impl_from_borrowed_row(t: &mut Tokens, struct_ident: &Ident, fields: &Fields) {
    t.append(format!("
impl<'a> From<&'a ::postgres::rows::Row<'a>> for {struct_name} {{
    fn from(row: &::postgres::rows::Row<'a>) -> Self {{
        Self {{", struct_name=struct_ident));

    for field in fields {
        let ident = field.ident.clone().expect("Expected structfield identifier");

        t.append(format!("
            {0}: row.get(\"{0}\"),", ident));
    }

    t.append("
        }
    }
}");
}

#[cfg(all(feature = "postgres-support", feature = "postgres-mapper"))]
fn impl_postgres_mapper(t: &mut Tokens, struct_ident: &Ident, fields: &Fields, table_name: &str) {
    t.append(format!("
impl ::postgres_mapper::FromPostgresRow for {struct_name} {{
    fn from_postgres_row(row: ::postgres::rows::Row)
        -> Result<Self, ::postgres_mapper::Error> {{
        Ok(Self {{", struct_name=struct_ident));

    for field in fields {
        let ident = field.ident.clone().expect("Expected structfield identifier");

        t.append(format!("
            {0}: row.get_opt(\"{0}\").ok_or_else(|| ::postgres_mapper::Error::ColumnNotFound)??,", ident));
    }

    t.append("
        })
    }

    fn from_postgres_row_ref(row: &::postgres::rows::Row)
        -> Result<Self, ::postgres_mapper::Error> {
        Ok(Self {");

    for field in fields {
        let ident = field.ident.clone().expect("Expected structfield identifier");

        t.append(format!("
            {0}: row.get_opt(\"{0}\").ok_or_else(|| ::postgres_mapper::Error::ColumnNotFound)??,", ident));
    }

    t.append("
        })
    }");

    t.append(format!(
    "fn sql_table() -> String {{
        \" {0} \".to_string()
    }}"
    , table_name));

    t.append(
    format!(
    "fn sql_fields() -> String {{")
    );

    let field_name = fields.iter().map(|field| {
        let ident = field.ident.clone().expect("Expected structfield identifier");
        format!("{0}.{1}", table_name, ident)
    }).collect::<Vec<String>>().join(", ");

    t.append(format!("\" {0} \".to_string()", field_name));

    t.append(
    "}"
    );

    t.append("
}");
}

#[cfg(feature = "tokio-postgres-support")]
fn impl_tokio_from_row(t: &mut Tokens, struct_ident: &Ident, fields: &Fields) {
    t.append(format!("
impl From<::tokio_postgres::rows::Row> for {struct_name} {{
    fn from(row: ::tokio_postgres::rows::Row) -> Self {{
        Self {{", struct_name=struct_ident));

    for field in fields {
        let ident = field.ident.clone().expect("Expected structfield identifier");

        t.append(format!("
            {0}: row.get(\"{0}\"),", ident));
    }

    t.append("
        }
    }
}");
}

#[cfg(feature = "tokio-postgres-support")]
fn impl_tokio_from_borrowed_row(t: &mut Tokens, struct_ident: &Ident, fields: &Fields) {
    t.append(format!("
impl<'a> From<&'a ::tokio_postgres::rows::Row> for {struct_name} {{
    fn from(row: &'a ::tokio_postgres::rows::Row) -> Self {{
        Self {{", struct_name=struct_ident));

    for field in fields {
        let ident = field.ident.clone().expect("Expected structfield identifier");

        t.append(format!("
            {0}: row.get(\"{0}\"),", ident));
    }

    t.append("
        }
    }
}");
}


#[cfg(all(feature = "tokio-postgres-support", feature = "postgres-mapper"))]
fn impl_tokio_postgres_mapper(
    t: &mut Tokens,
    struct_ident: &Ident,
    fields: &Fields,
    table_name: &str,
) {
    t.append(format!("
impl ::postgres_mapper::FromTokioPostgresRow for {struct_name} {{
    fn from_tokio_postgres_row(row: ::tokio_postgres::rows::Row)
        -> Result<Self, ::postgres_mapper::Error> {{
        Ok(Self {{", struct_name=struct_ident));

    for field in fields {
        let ident = field.ident.clone().expect("Expected structfield identifier");

        t.append(format!("
            {0}: row.try_get(\"{0}\")?.ok_or_else(|| ::postgres_mapper::Error::ColumnNotFound)?,", ident));
    }

    t.append("
        })
    }

    fn from_tokio_postgres_row_ref(row: &::tokio_postgres::rows::Row)
        -> Result<Self, ::postgres_mapper::Error> {
        Ok(Self {");

    for field in fields {
        let ident = field.ident.clone().expect("Expected structfield identifier");

        t.append(format!("
            {0}: row.try_get(\"{0}\")?.ok_or_else(|| ::postgres_mapper::Error::ColumnNotFound)?,", ident));
    }

    t.append("
        })
    }");

    t.append(format!(
    "fn sql_table() -> String {{
        \" {0} \".to_string()
    }}"
    , table_name));

    t.append(
    format!(
    "fn sql_fields() -> String {{")
    );

    let field_name = fields.iter().map(|field| {
        let ident = field.ident.clone().expect("Expected structfield identifier");
        format!("{0}.{1}", table_name, ident)
    }).collect::<Vec<String>>().join(", ");

    t.append(format!("\" {0} \".to_string()", field_name));

    t.append(
    "}"
    );

    t.append("
}");
}

fn get_mapper_meta_items(attr: &syn::Attribute) -> Option<Vec<syn::NestedMeta>> {
    if attr.path.segments.len() == 1 && attr.path.segments[0].ident == "pg_mapper" {
        match attr.interpret_meta() {
            Some(List(ref meta)) => Some(meta.nested.iter().cloned().collect()),
            _ => {
                panic!("declare table name: #[pg_mapper(table = \"foo\")]");
            }
        }
    } else {
        None
    }
}

fn get_lit_str<'a>(
    attr_name: &Ident,
    meta_item_name: &Ident,
    lit: &'a syn::Lit,
) -> Result<&'a syn::LitStr, ()> {
    if let syn::Lit::Str(ref lit) = *lit {
        Ok(lit)
    } else {
        panic!(format!(
            "expected pg_mapper {} attribute to be a string: `{} = \"...\"`",
            attr_name, meta_item_name
        ));
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
                Meta(NameValue(ref m)) if m.ident == "table" => {
                    if let Ok(s) = get_lit_str(&m.ident, &m.ident, &m.lit) {
                        table_name = Some(s.value());
                    }
                }

                Meta(ref meta_item) => {
                    panic!(format!(
                        "unknown pg_mapper container attribute `{}`",
                        meta_item.name()
                    ))
                }

                Literal(_) => {
                    panic!("unexpected literal in pg_mapper container attribute");
                }
            }
        }
    }

    table_name.expect("declare table name: #[pg_mapper(table = \"foo\")]")
}

