use proc_macro2::{Ident, Span};
use std::collections::HashMap;
use syn::{
    punctuated::Punctuated, token::Comma, Lit, LitStr, Meta, MetaNameValue, NestedMeta, Path, Token,
};

pub struct Field<'a> {
    pub ident: &'a Ident,
    pub prefix: &'a str,
    pub meta_items: &'a Punctuated<NestedMeta, Comma>,
    pub span: Span,
}

impl Field<'_> {
    pub fn get_new_meta_items(&mut self) -> Punctuated<NestedMeta, Comma> {
        let mut visited_args: HashMap<String, String> = HashMap::new();
        let mut new_meta_items = Punctuated::new();

        for item in self.meta_items.iter() {
            if let NestedMeta::Meta(meta) = item {
                if let Some(ident) = meta.path().get_ident() {
                    let ident = ident.to_string();

                    match ident.as_str() {
                        "id" | "long" | "env" | "value_name" => match meta {
                            Meta::NameValue(MetaNameValue {
                                path: _,
                                lit: Lit::Str(lit_str),
                                eq_token: _,
                            }) => {
                                visited_args.insert(ident, lit_str.value());
                            }
                            _ => {
                                if ident != "id" && ident != "value_name" {
                                    visited_args.insert(ident, String::new());
                                }
                            }
                        },
                        _ => {
                            new_meta_items.push(item.clone());
                        }
                    }
                }
            }
        }

        let snake_case_value = format!("{}_{}", self.prefix, self.ident);
        let screaming_case_value = snake_case_value.to_uppercase();
        let kebab_case_value = snake_case_value.replace('_', "-");

        new_meta_items
            .push(self.create_meta_item("id", visited_args.get("id").unwrap_or(&snake_case_value)));

        new_meta_items.push(
            self.create_meta_item(
                "value_name",
                visited_args
                    .get("value_name")
                    .unwrap_or(&screaming_case_value),
            ),
        );

        if let Some(value) = visited_args.get("long") {
            new_meta_items.push(self.create_meta_item(
                "long",
                if value.is_empty() {
                    &kebab_case_value
                } else {
                    value
                },
            ));
        }

        if let Some(value) = visited_args.get("env") {
            new_meta_items.push(self.create_meta_item(
                "env",
                if value.is_empty() {
                    &screaming_case_value
                } else {
                    value
                },
            ));
        }

        new_meta_items
    }

    fn create_meta_item(&self, path: &str, value: &str) -> NestedMeta {
        NestedMeta::Meta(Meta::NameValue(syn::MetaNameValue {
            path: Path::from(Ident::new(path, self.span)),
            eq_token: Token![=]([self.span]),
            lit: Lit::Str(LitStr::new(value, self.span)),
        }))
    }
}
