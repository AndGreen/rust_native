use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{braced, parenthesized, Expr, Ident, Path, Result, Token};

pub struct UiRoot {
    pub nodes: Vec<WidgetNode>,
}

impl UiRoot {
    pub fn expand(&self) -> TokenStream {
        match self.nodes.len() {
            0 => quote! { mf_core::View::fragment(::std::vec::Vec::<mf_core::View>::new()) },
            1 => self.nodes[0].expand(),
            _ => {
                let pushes = self.nodes.iter().map(|node| {
                    let expanded = node.expand();
                    quote! { __children.push(#expanded); }
                });
                quote! {{
                    let mut __children: ::std::vec::Vec<mf_core::View> = ::std::vec::Vec::new();
                    #(#pushes)*
                    mf_core::View::fragment(__children)
                }}
            }
        }
    }
}

impl Parse for UiRoot {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mut nodes = Vec::new();
        while !input.is_empty() {
            nodes.push(input.parse()?);
            while input.peek(Token![,]) || input.peek(Token![;]) {
                if input.peek(Token![,]) {
                    let _ = input.parse::<Token![,]>();
                } else {
                    let _ = input.parse::<Token![;]>();
                }
            }
        }
        Ok(Self { nodes })
    }
}

pub struct WidgetNode {
    head: WidgetHead,
    modifiers: Vec<Modifier>,
    children: Option<UiRoot>,
}

impl Parse for WidgetNode {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let path: Path = input.parse()?;
        let args = if input.peek(syn::token::Paren) {
            let content;
            parenthesized!(content in input);
            Some(Arguments::parse(&content)?)
        } else {
            None
        };
        let mut head = WidgetHead {
            path,
            args,
            named: Vec::new(),
        };
        if let Some(arguments) = &head.args {
            if !arguments.named.is_empty() {
                head.named = arguments.named.clone();
            }
        }
        let mut modifiers = Vec::new();
        WidgetNode::consume_modifiers(input, &mut modifiers)?;
        let children = if input.peek(syn::token::Brace) {
            let content;
            braced!(content in input);
            let parsed = content.parse()?;
            WidgetNode::consume_modifiers(input, &mut modifiers)?;
            Some(parsed)
        } else {
            None
        };
        Ok(Self {
            head,
            modifiers,
            children,
        })
    }
}

impl WidgetNode {
    fn consume_modifiers(input: ParseStream<'_>, modifiers: &mut Vec<Modifier>) -> Result<()> {
        while input.peek(Token![.]) {
            input.parse::<Token![.]>()?;
            let name: Ident = input.parse()?;
            let args = if input.peek(syn::token::Paren) {
                let content;
                parenthesized!(content in input);
                content.parse_terminated(Expr::parse, Token![,])?
            } else {
                Punctuated::new()
            };
            modifiers.push(Modifier { name, args });
        }
        Ok(())
    }

    pub fn expand(&self) -> TokenStream {
        let mut expr = self.head.expand_base();
        for named in &self.head.named {
            expr = named.apply(expr);
        }
        for modifier in &self.modifiers {
            expr = modifier.apply(expr);
        }
        if let Some(children) = &self.children {
            let pushes = children.nodes.iter().map(|child| {
                let expanded = child.expand();
                quote! { __children.push(#expanded); }
            });
            quote! {{
                let mut __children: ::std::vec::Vec<mf_core::View> = ::std::vec::Vec::new();
                #(#pushes)*
                mf_core::dsl::WithChildren::with_children(#expr, __children)
            }}
        } else {
            quote! { mf_core::dsl::IntoView::into_view(#expr) }
        }
    }
}

struct WidgetHead {
    path: Path,
    args: Option<Arguments>,
    named: Vec<NamedArgument>,
}

impl WidgetHead {
    fn expand_base(&self) -> TokenStream {
        let path = &self.path;
        if let Some(args) = &self.args {
            args.expand(path)
        } else {
            quote! { #path() }
        }
    }
}

#[derive(Clone)]
struct NamedArgument {
    label: Ident,
    value: Expr,
}

impl NamedArgument {
    fn apply(&self, expr: TokenStream) -> TokenStream {
        let label = &self.label;
        let value = &self.value;
        quote! { (#expr).#label(#value) }
    }
}

struct Modifier {
    name: Ident,
    args: Punctuated<Expr, Token![,]>,
}

impl Modifier {
    fn apply(&self, expr: TokenStream) -> TokenStream {
        let name = &self.name;
        let args = &self.args;
        if args.is_empty() {
            quote! { (#expr).#name() }
        } else {
            quote! { (#expr).#name(#args) }
        }
    }
}

struct Arguments {
    positional: Vec<Expr>,
    named: Vec<NamedArgument>,
}

impl Arguments {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mut positional = Vec::new();
        let mut named = Vec::new();
        while !input.is_empty() {
            if input.peek(Ident) && input.peek2(Token![=]) {
                let label: Ident = input.parse()?;
                input.parse::<Token![=]>()?;
                let value: Expr = input.parse()?;
                named.push(NamedArgument { label, value });
            } else {
                positional.push(input.parse()?);
            }
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }
        Ok(Self { positional, named })
    }

    fn expand(&self, path: &Path) -> TokenStream {
        if self.positional.is_empty() {
            quote! { #path() }
        } else {
            let positional = &self.positional;
            quote! { #path(#(#positional),*) }
        }
    }
}
