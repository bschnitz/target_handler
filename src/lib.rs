use darling::FromDeriveInput;
use proc_macro::TokenStream;
use quote::quote;
use syn::{self, Ident, Data, Variant, Fields, FieldsNamed, DataEnum};

type TokenStream2 = proc_macro2::TokenStream;

#[derive(FromDeriveInput, Default)]
#[darling(attributes(handler))]
struct HandlerOpts {
    returns: Option<String>,
    trait_name: Option<String>,
    method: Option<String>
}

impl HandlerOpts {
    fn get_returns(&self) -> TokenStream2 {
        str_to_tok(&self.returns.as_ref().map_or("()", |s| { s.as_str() }))
    }

    fn get_trait_name(&self, ast: &syn::DeriveInput) -> TokenStream2 {
        if let Some(name) = &self.trait_name {
            return str_to_tok(name);
        }
        let name = &ast.ident;
        str_to_tok(&format!("{name}Handler"))
    }

    fn get_handler_method(&self, ast: &syn::DeriveInput) -> TokenStream2 {
        if let Some(method) = &self.method {
            return str_to_tok(method);
        }
        let name = lower_name(ast.ident.to_string());
        str_to_tok(&format!("handle_{name}"))
    }
}

fn lower_name(name: String) -> String {
    name.to_lowercase()
}

fn str_to_tok(arg: &str) -> TokenStream2 {
    arg.parse().unwrap()
}

fn enum_variant_to_handle_ident(var: &Variant) -> Ident {
    let ident = &var.ident;
    let name = lower_name(ident.to_string());
    Ident::new(&name, ident.span())
} 

fn enum_variant_to_handle_arguments(var: &Variant) -> TokenStream2 {
    if let Fields::Named(fields) = &var.fields {
        return arguments_from_named_fields(fields);
    }
    quote! { &self }
}

fn arguments_from_named_fields(fields: &FieldsNamed) -> TokenStream2 {
    let args = fields.named.iter().filter_map(|field| {
        let ident = field.ident.clone()?;
        let ty = &field.ty;
        Some(quote! {#ident: #ty})
    });
    quote! { &self, #(#args),* }
}

fn get_field_name_list(fields: &Fields) -> TokenStream2
{
    match &fields {
        Fields::Named(fields) => get_named_fields_name_list(fields),
        _                     => TokenStream2::new()
    }
}

fn get_named_fields_name_list(fields: &FieldsNamed) -> TokenStream2 {
    let names = get_idents_of_named_fields(fields);
    quote! { #(#names),* }
}

fn get_idents_of_named_fields<'a>(fields: &'a FieldsNamed) -> impl Iterator<Item=&Ident> + 'a {
    fields.named.iter().filter_map(|field| { field.ident.as_ref() })
}

struct TargetMacroGenerator {
    opts:      HandlerOpts,
    ast:       syn::DeriveInput,
}

impl<'a> TargetMacroGenerator {
    fn new(ast: syn::DeriveInput, opts: HandlerOpts) -> TargetMacroGenerator {
        TargetMacroGenerator { ast, opts }
    }

    fn get_data_enum(&self) -> &DataEnum {
        if let Data::Enum(data) = &self.ast.data {
            return data;
        }
        panic!("Target must be an enum.");
    }

    fn generate(&self) -> TokenStream {
        let trait_name = self.opts.get_trait_name(&self.ast);
        let handles = self.get_handles();
        let handler_function = self.get_handler_function();
        quote! {
            trait #trait_name {
                #(#handles)*

                #handler_function
            }
        }.into()
    }

    fn get_handles(&self) -> impl Iterator<Item=TokenStream2> + '_ {
        self.get_data_enum().variants
            .iter()
            .map(|var| { self.enum_variant_to_handle(var) })
    }

    fn enum_variant_to_handle(&self, var: &Variant) -> TokenStream2 {
        let ident = enum_variant_to_handle_ident(var);
        let arguments = enum_variant_to_handle_arguments(var);
        let returns = self.opts.get_returns();
        quote! { fn #ident(#arguments) -> #returns; }
    }

    fn get_handler_function(&self) -> TokenStream2 {
        let handler_method = self.opts.get_handler_method(&self.ast);
        let enum_name      = &self.ast.ident;
        let handler_arms   = self.get_handler_arms();
        let returns        = self.opts.get_returns();

        quote! {
            fn #handler_method(&self, handled_enum: #enum_name) -> #returns {
                match handled_enum {
                    #(#handler_arms)*
                }
            }
        }
    }

    fn get_handler_arms(&self) -> impl Iterator<Item=TokenStream2> + '_ {
        self.get_data_enum().variants
            .iter()
            .map(|var| { self.enum_variant_to_match_arm(var) })
    }

    fn enum_variant_to_match_arm(&self, variant: &Variant) -> TokenStream2 {
        let enum_name = &self.ast.ident;
        let variant_name = &variant.ident;
        let variant_handle_name = enum_variant_to_handle_ident(variant);
        let field_name_list = get_field_name_list(&variant.fields);

        quote! {
            #enum_name::#variant_name { #field_name_list } => {
                self.#variant_handle_name(#field_name_list)
            }
        }
    }
}

#[proc_macro_derive(Target, attributes(handler))]
pub fn targets_derive(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    let opts = HandlerOpts::from_derive_input(&ast).expect("Wrong options for 'handler'.");
    TargetMacroGenerator::new(ast, opts).generate()
}
