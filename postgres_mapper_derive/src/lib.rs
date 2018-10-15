extern crate quote;
extern crate proc_macro;
extern crate syn;

use proc_macro::TokenStream;
use quote::Tokens;
use syn::{Body, DeriveInput, VariantData};

#[cfg(any(feature = "postgres-support", "tokio-postgres-support")]
use syn::{Field, Ident};

#[proc_macro_derive(PostgresMapper)]
pub fn postgres_mapper(input: TokenStream) -> TokenStream {
    let s = input.to_string();
    let ast = syn::parse_derive_input(&s).unwrap();

    impl_derive(&ast)
        .parse()
        .expect("Error parsing postgres mapper tokens")
}

fn impl_derive(ast: &DeriveInput) -> Tokens {
    #[allow(unused_mut)]
    let mut tokens = Tokens::new();

    #[allow(unused_variables)]
    let fields = match ast.body {
        Body::Struct(VariantData::Struct(ref fields)) => fields,
        Body::Struct(VariantData::Tuple(_)) => panic!("Tuple-structs not supported"),
        Body::Struct(VariantData::Unit) => panic!("Unit structs not supported"),
        Body::Enum(_) => panic!("Enums can not be mapped"),
    };

    #[cfg(feature = "postgres-support")]
    {
        impl_from_row(&mut tokens, &ast.ident, &fields);
        impl_from_borrowed_row(&mut tokens, &ast.ident, &fields);

        #[cfg(feature = "postgres-mapper")]
        {
            impl_postgres_mapper(&mut tokens, &ast.ident, &fields);
        }
    }

    #[cfg(feature = "tokio-postgres-support")]
    {
        impl_tokio_from_row(&mut tokens, &ast.ident, &fields);
        impl_tokio_from_borrowed_row(&mut tokens, &ast.ident, &fields);

        #[cfg(feature = "postgres-mapper")]
        {
            impl_tokio_postgres_mapper(&mut tokens, &ast.ident, &fields);
        }
    }

    tokens
}

#[cfg(feature = "postgres-support")]
fn impl_from_row(t: &mut Tokens, struct_ident: &Ident, fields: &[Field]) {
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
fn impl_from_borrowed_row(t: &mut Tokens, struct_ident: &Ident, fields: &[Field]) {
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
fn impl_postgres_mapper(t: &mut Tokens, struct_ident: &Ident, fields: &[Field]) {
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
    }
}");
}

#[cfg(feature = "tokio-postgres-support")]
fn impl_tokio_from_row(t: &mut Tokens, struct_ident: &Ident, fields: &[Field]) {
    t.append(format!("
impl From<::tokio_postgres::Row> for {struct_name} {{
    fn from(row: ::tokio_postgres::Row) -> Self {{
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
fn impl_tokio_from_borrowed_row(t: &mut Tokens, struct_ident: &Ident, fields: &[Field]) {
    t.append(format!("
impl<'a> From<&'a ::tokio_postgres::Row> for {struct_name} {{
    fn from(row: &'a ::tokio_postgres::Row) -> Self {{
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
    fields: &[Field],
) {
    t.append(format!("
impl ::postgres_mapper::FromTokioPostgresRow for {struct_name} {{
    fn from_tokio_postgres_row(row: ::tokio_postgres::Row)
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

    fn from_tokio_postgres_row_ref(row: &::tokio_postgres::Row)
        -> Result<Self, ::postgres_mapper::Error> {
        Ok(Self {");

    for field in fields {
        let ident = field.ident.clone().expect("Expected structfield identifier");

        t.append(format!("
            {0}: row.try_get(\"{0}\")?.ok_or_else(|| ::postgres_mapper::Error::ColumnNotFound)?,", ident));
    }

    t.append("
        })
    }
}");
}
