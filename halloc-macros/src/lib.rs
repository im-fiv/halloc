extern crate proc_macro as pm;

use quote::quote;

use syn::{Ident, Type, WhereClause, Token, braced, parse_macro_input};
use syn::token;

#[derive(Debug)]
struct ImplAllocInput {
	pub trait_name: Ident,
	pub types: Vec<Type>,
	pub where_clause: Option<WhereClause>
}

fn parse_types(input: syn::parse::ParseStream) -> syn::Result<Vec<Type>> {
	// If implementing a single type
	if !input.peek(token::Brace) {
		let ty = input.parse::<Type>()?;
		return Ok(vec![ty]);
	}

	// Otherwise, parse all of them separated by a comma
	let content;
	braced!(content in input);

	let mut types = vec![];

	while !content.is_empty() {
		let ty = content.parse::<Type>()?;
		types.push(ty);

		// Parse the comma if it exists
		if content.peek(Token![,]) {
			let _comma = content.parse::<Token![,]>()?;
		}
	}

	Ok(types)
}

impl syn::parse::Parse for ImplAllocInput {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		let trait_name = input.parse::<Ident>()?;
		let _for_token = input.parse::<Token![for]>()?;
		let types = parse_types(input)?;

		let where_clause = if input.peek(Token![where]) {
			Some(input.parse::<WhereClause>()?)
		} else {
			None
		};

		Ok(Self {
			trait_name,
			types,
			where_clause
		})
	}
}

#[proc_macro]
pub fn impl_alloc(input: pm::TokenStream) -> pm::TokenStream {
	let ImplAllocInput {
		trait_name,
		types,
		where_clause
	} = parse_macro_input!(input as ImplAllocInput);

	let clause_predicates = where_clause.map(|w| w.predicates);
	
	let mut impls = vec![];

	for ty in types {
		impls.push(quote! {
			impl<#clause_predicates> #trait_name for #ty {}
		});
	}

	quote! {
		#( #impls )*
	}.into()
}