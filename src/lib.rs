use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{spanned::Spanned, ItemStruct, Meta, Result};

mod field;

#[proc_macro_attribute]
pub fn clap_prefix(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    clap_prefix_inner(TokenStream::from(args), TokenStream::from(input))
        .unwrap()
        .into()
}

fn clap_prefix_inner(_args: TokenStream, input: TokenStream) -> Result<TokenStream> {
    let mut item: ItemStruct = syn::parse2(input)?;
    let prefix = item.ident.to_string().to_lowercase();

    for field in item.fields.iter_mut() {
        if let Some(field_ident) = &field.ident {
            for attribute in field.attrs.iter_mut() {
                if !attribute.path.is_ident("arg") {
                    continue;
                }

                if let Ok(Meta::List(mut meta)) = attribute.parse_meta() {
                    let new_attribute_value = field::Field {
                        ident: field_ident,
                        meta_items: &mut meta.nested,
                        prefix: &prefix,
                        span: attribute.span(),
                    }
                    .get_new_meta_items()
                    .to_token_stream();

                    attribute.tokens = quote! {(#new_attribute_value)};
                }
            }
        }
    }

    Ok(item.into_token_stream())
}

#[cfg(test)]
mod tests {
    use prettyplease;
    use proc_macro2::TokenStream;
    use std::str::FromStr;

    use crate::clap_prefix_inner;

    #[test]
    fn test() {
        let input_string = r#"
        pub struct Keycloak {
            #[arg()]
            pub server_url: String,
            #[arg(long, env, default_value="master")]
            pub realm: String,
            #[arg(long="clientid")]
            pub client_id: String,
            #[arg(long, value_name)]
            pub secret: String,
        }
        "#;

        let input_token_stream = TokenStream::from_str(input_string).unwrap();
        let output_token_stream =
            clap_prefix_inner(TokenStream::from_str("").unwrap(), input_token_stream);
        let file = syn::parse_file(&output_token_stream.unwrap().to_string()).unwrap();
        let result = prettyplease::unparse(&file);

        let expected = r#"
        pub struct Keycloak {
            #[arg(id = "keycloak_server_url", value_name = "KEYCLOAK_SERVER_URL")]
            pub server_url: String,
            #[arg(
                default_value = "master",
                id = "keycloak_realm",
                value_name = "KEYCLOAK_REALM",
                long = "keycloak-realm",
                env = "KEYCLOAK_REALM"
            )]
            pub realm: String,
            #[arg(
                id = "keycloak_client_id",
                value_name = "KEYCLOAK_CLIENT_ID",
                long = "clientid"
            )]
            pub client_id: String,
            #[arg(
                id = "keycloak_secret",
                value_name = "KEYCLOAK_SECRET",
                long = "keycloak-secret"
            )]
            pub secret: String,
        }
        "#;

        assert_eq!(
            result,
            prettyplease::unparse(&syn::parse_file(expected).unwrap())
        );
    }
}
