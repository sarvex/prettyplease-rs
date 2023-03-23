use crate::algorithm::Printer;
use crate::iter::IterDelimited;
use crate::path::PathKind;
use crate::INDENT;
use proc_macro2::TokenStream;
use syn::{
    Fields, FnArg, ForeignItem, ForeignItemFn, ForeignItemMacro, ForeignItemStatic,
    ForeignItemType, ImplItem, ImplItemConst, ImplItemFn, ImplItemMacro, ImplItemType, Item,
    ItemConst, ItemEnum, ItemExternCrate, ItemFn, ItemForeignMod, ItemImpl, ItemMacro, ItemMod,
    ItemStatic, ItemStruct, ItemTrait, ItemTraitAlias, ItemType, ItemUnion, ItemUse, Receiver,
    Signature, StaticMutability, Stmt, TraitItem, TraitItemConst, TraitItemFn, TraitItemMacro,
    TraitItemType, Type, UseGlob, UseGroup, UseName, UsePath, UseRename, UseTree, Variadic,
};

impl Printer {
    pub fn item(&mut self, item: &Item) {
        match item {
            Item::Const(item) => self.item_const(item),
            Item::Enum(item) => self.item_enum(item),
            Item::ExternCrate(item) => self.item_extern_crate(item),
            Item::Fn(item) => self.item_fn(item),
            Item::ForeignMod(item) => self.item_foreign_mod(item),
            Item::Impl(item) => self.item_impl(item),
            Item::Macro(item) => self.item_macro(item),
            Item::Mod(item) => self.item_mod(item),
            Item::Static(item) => self.item_static(item),
            Item::Struct(item) => self.item_struct(item),
            Item::Trait(item) => self.item_trait(item),
            Item::TraitAlias(item) => self.item_trait_alias(item),
            Item::Type(item) => self.item_type(item),
            Item::Union(item) => self.item_union(item),
            Item::Use(item) => self.item_use(item),
            Item::Verbatim(item) => self.item_verbatim(item),
            #[cfg_attr(all(test, exhaustive), deny(non_exhaustive_omitted_patterns))]
            _ => unimplemented!("unknown Item"),
        }
    }

    fn item_const(&mut self, item: &ItemConst) {
        self.outer_attrs(&item.attrs);
        self.cbox(0);
        self.visibility(&item.vis);
        self.word("const ");
        self.ident(&item.ident);
        self.generics(&item.generics);
        self.word(": ");
        self.ty(&item.ty);
        self.word(" = ");
        self.neverbreak();
        self.expr(&item.expr);
        self.word(";");
        self.end();
        self.hardbreak();
    }

    fn item_enum(&mut self, item: &ItemEnum) {
        self.outer_attrs(&item.attrs);
        self.cbox(INDENT);
        self.visibility(&item.vis);
        self.word("enum ");
        self.ident(&item.ident);
        self.generics(&item.generics);
        self.where_clause_for_body(&item.generics.where_clause);
        self.word("{");
        self.hardbreak_if_nonempty();
        for variant in &item.variants {
            self.variant(variant);
            self.word(",");
            self.hardbreak();
        }
        self.offset(-INDENT);
        self.end();
        self.word("}");
        self.hardbreak();
    }

    fn item_extern_crate(&mut self, item: &ItemExternCrate) {
        self.outer_attrs(&item.attrs);
        self.visibility(&item.vis);
        self.word("extern crate ");
        self.ident(&item.ident);
        if let Some((_as_token, rename)) = &item.rename {
            self.word(" as ");
            self.ident(rename);
        }
        self.word(";");
        self.hardbreak();
    }

    fn item_fn(&mut self, item: &ItemFn) {
        self.outer_attrs(&item.attrs);
        self.cbox(INDENT);
        self.visibility(&item.vis);
        self.signature(&item.sig);
        self.where_clause_for_body(&item.sig.generics.where_clause);
        self.word("{");
        self.hardbreak_if_nonempty();
        self.inner_attrs(&item.attrs);
        for stmt in &item.block.stmts {
            self.stmt(stmt);
        }
        self.offset(-INDENT);
        self.end();
        self.word("}");
        self.hardbreak();
    }

    fn item_foreign_mod(&mut self, item: &ItemForeignMod) {
        self.outer_attrs(&item.attrs);
        self.cbox(INDENT);
        if item.unsafety.is_some() {
            self.word("unsafe ");
        }
        self.abi(&item.abi);
        self.word("{");
        self.hardbreak_if_nonempty();
        self.inner_attrs(&item.attrs);
        for foreign_item in &item.items {
            self.foreign_item(foreign_item);
        }
        self.offset(-INDENT);
        self.end();
        self.word("}");
        self.hardbreak();
    }

    fn item_impl(&mut self, item: &ItemImpl) {
        self.outer_attrs(&item.attrs);
        self.cbox(INDENT);
        self.ibox(-INDENT);
        self.cbox(INDENT);
        if item.defaultness.is_some() {
            self.word("default ");
        }
        if item.unsafety.is_some() {
            self.word("unsafe ");
        }
        self.word("impl");
        self.generics(&item.generics);
        self.end();
        self.nbsp();
        if let Some((negative_polarity, path, _for_token)) = &item.trait_ {
            if negative_polarity.is_some() {
                self.word("!");
            }
            self.path(path, PathKind::Type);
            self.space();
            self.word("for ");
        }
        self.ty(&item.self_ty);
        self.end();
        self.where_clause_for_body(&item.generics.where_clause);
        self.word("{");
        self.hardbreak_if_nonempty();
        self.inner_attrs(&item.attrs);
        for impl_item in &item.items {
            self.impl_item(impl_item);
        }
        self.offset(-INDENT);
        self.end();
        self.word("}");
        self.hardbreak();
    }

    fn item_macro(&mut self, item: &ItemMacro) {
        self.outer_attrs(&item.attrs);
        self.mac(&item.mac, item.ident.as_ref());
        self.mac_semi_if_needed(&item.mac.delimiter);
        self.hardbreak();
    }

    fn item_mod(&mut self, item: &ItemMod) {
        self.outer_attrs(&item.attrs);
        self.cbox(INDENT);
        self.visibility(&item.vis);
        if item.unsafety.is_some() {
            self.word("unsafe ");
        }
        self.word("mod ");
        self.ident(&item.ident);
        if let Some((_brace, items)) = &item.content {
            self.word(" {");
            self.hardbreak_if_nonempty();
            self.inner_attrs(&item.attrs);
            for item in items {
                self.item(item);
            }
            self.offset(-INDENT);
            self.end();
            self.word("}");
        } else {
            self.word(";");
            self.end();
        }
        self.hardbreak();
    }

    fn item_static(&mut self, item: &ItemStatic) {
        self.outer_attrs(&item.attrs);
        self.cbox(0);
        self.visibility(&item.vis);
        self.word("static ");
        if let StaticMutability::Mut(_) = item.mutability {
            self.word("mut ");
        }
        self.ident(&item.ident);
        self.word(": ");
        self.ty(&item.ty);
        self.word(" = ");
        self.neverbreak();
        self.expr(&item.expr);
        self.word(";");
        self.end();
        self.hardbreak();
    }

    fn item_struct(&mut self, item: &ItemStruct) {
        self.outer_attrs(&item.attrs);
        self.cbox(INDENT);
        self.visibility(&item.vis);
        self.word("struct ");
        self.ident(&item.ident);
        self.generics(&item.generics);
        match &item.fields {
            Fields::Named(fields) => {
                self.where_clause_for_body(&item.generics.where_clause);
                self.word("{");
                self.hardbreak_if_nonempty();
                for field in &fields.named {
                    self.field(field);
                    self.word(",");
                    self.hardbreak();
                }
                self.offset(-INDENT);
                self.end();
                self.word("}");
            }
            Fields::Unnamed(fields) => {
                self.fields_unnamed(fields);
                self.where_clause_semi(&item.generics.where_clause);
                self.end();
            }
            Fields::Unit => {
                self.where_clause_semi(&item.generics.where_clause);
                self.end();
            }
        }
        self.hardbreak();
    }

    fn item_trait(&mut self, item: &ItemTrait) {
        self.outer_attrs(&item.attrs);
        self.cbox(INDENT);
        self.visibility(&item.vis);
        if item.unsafety.is_some() {
            self.word("unsafe ");
        }
        if item.auto_token.is_some() {
            self.word("auto ");
        }
        self.word("trait ");
        self.ident(&item.ident);
        self.generics(&item.generics);
        for supertrait in item.supertraits.iter().delimited() {
            if supertrait.is_first {
                self.word(": ");
            } else {
                self.word(" + ");
            }
            self.type_param_bound(&supertrait);
        }
        self.where_clause_for_body(&item.generics.where_clause);
        self.word("{");
        self.hardbreak_if_nonempty();
        self.inner_attrs(&item.attrs);
        for trait_item in &item.items {
            self.trait_item(trait_item);
        }
        self.offset(-INDENT);
        self.end();
        self.word("}");
        self.hardbreak();
    }

    fn item_trait_alias(&mut self, item: &ItemTraitAlias) {
        self.outer_attrs(&item.attrs);
        self.cbox(INDENT);
        self.visibility(&item.vis);
        self.word("trait ");
        self.ident(&item.ident);
        self.generics(&item.generics);
        self.word(" = ");
        self.neverbreak();
        for bound in item.bounds.iter().delimited() {
            if !bound.is_first {
                self.space();
                self.word("+ ");
            }
            self.type_param_bound(&bound);
        }
        self.where_clause_semi(&item.generics.where_clause);
        self.end();
        self.hardbreak();
    }

    fn item_type(&mut self, item: &ItemType) {
        self.outer_attrs(&item.attrs);
        self.cbox(INDENT);
        self.visibility(&item.vis);
        self.word("type ");
        self.ident(&item.ident);
        self.generics(&item.generics);
        self.where_clause_oneline(&item.generics.where_clause);
        self.word("= ");
        self.neverbreak();
        self.ibox(-INDENT);
        self.ty(&item.ty);
        self.end();
        self.word(";");
        self.end();
        self.hardbreak();
    }

    fn item_union(&mut self, item: &ItemUnion) {
        self.outer_attrs(&item.attrs);
        self.cbox(INDENT);
        self.visibility(&item.vis);
        self.word("union ");
        self.ident(&item.ident);
        self.generics(&item.generics);
        self.where_clause_for_body(&item.generics.where_clause);
        self.word("{");
        self.hardbreak_if_nonempty();
        for field in &item.fields.named {
            self.field(field);
            self.word(",");
            self.hardbreak();
        }
        self.offset(-INDENT);
        self.end();
        self.word("}");
        self.hardbreak();
    }

    fn item_use(&mut self, item: &ItemUse) {
        self.outer_attrs(&item.attrs);
        self.visibility(&item.vis);
        self.word("use ");
        if item.leading_colon.is_some() {
            self.word("::");
        }
        self.use_tree(&item.tree);
        self.word(";");
        self.hardbreak();
    }

    #[cfg(not(feature = "verbatim"))]
    fn item_verbatim(&mut self, item: &TokenStream) {
        if !item.is_empty() {
            unimplemented!("Item::Verbatim `{}`", item);
        }
        self.hardbreak();
    }

    #[cfg(feature = "verbatim")]
    fn item_verbatim(&mut self, tokens: &TokenStream) {
        use proc_macro2::Ident;
        use syn::parse::{Parse, ParseStream, Result};
        use syn::punctuated::Punctuated;
        use syn::{braced, Attribute, Expr, Token, Visibility};

        enum ItemVerbatim {
            Empty,
            FnSignature(FnSignature),
            StaticUntyped(StaticUntyped),
            UseBrace(UseBrace),
        }

        struct FnSignature {
            attrs: Vec<Attribute>,
            vis: Visibility,
            sig: Signature,
        }

        struct StaticUntyped {
            attrs: Vec<Attribute>,
            vis: Visibility,
            mutability: StaticMutability,
            ident: Ident,
            expr: Expr,
        }

        struct UseBrace {
            attrs: Vec<Attribute>,
            vis: Visibility,
            trees: Punctuated<RootUseTree, Token![,]>,
        }

        struct RootUseTree {
            leading_colon: Option<Token![::]>,
            inner: UseTree,
        }

        impl Parse for RootUseTree {
            fn parse(input: ParseStream) -> Result<Self> {
                Ok(RootUseTree {
                    leading_colon: input.parse()?,
                    inner: input.parse()?,
                })
            }
        }

        impl Parse for ItemVerbatim {
            fn parse(input: ParseStream) -> Result<Self> {
                if input.is_empty() {
                    return Ok(ItemVerbatim::Empty);
                }

                let attrs = input.call(Attribute::parse_outer)?;
                let vis: Visibility = input.parse()?;

                let lookahead = input.lookahead1();
                if lookahead.peek(Token![const])
                    || lookahead.peek(Token![async])
                    || lookahead.peek(Token![unsafe])
                    || lookahead.peek(Token![extern])
                    || lookahead.peek(Token![fn])
                {
                    let sig: Signature = input.parse()?;
                    input.parse::<Token![;]>()?;
                    Ok(ItemVerbatim::FnSignature(FnSignature { attrs, vis, sig }))
                } else if lookahead.peek(Token![static]) {
                    input.parse::<Token![static]>()?;
                    let mutability: StaticMutability = input.parse()?;
                    let ident = input.parse()?;
                    input.parse::<Token![=]>()?;
                    let expr: Expr = input.parse()?;
                    input.parse::<Token![;]>()?;
                    Ok(ItemVerbatim::StaticUntyped(StaticUntyped {
                        attrs,
                        vis,
                        mutability,
                        ident,
                        expr,
                    }))
                } else if lookahead.peek(Token![use]) {
                    input.parse::<Token![use]>()?;
                    let content;
                    braced!(content in input);
                    let trees = content.parse_terminated(RootUseTree::parse, Token![,])?;
                    input.parse::<Token![;]>()?;
                    Ok(ItemVerbatim::UseBrace(UseBrace { attrs, vis, trees }))
                } else {
                    Err(lookahead.error())
                }
            }
        }

        let item: ItemVerbatim = match syn::parse2(tokens.clone()) {
            Ok(item) => item,
            Err(_) => unimplemented!("Item::Verbatim `{}`", tokens),
        };

        match item {
            ItemVerbatim::Empty => {}
            ItemVerbatim::FnSignature(item) => {
                self.outer_attrs(&item.attrs);
                self.cbox(INDENT);
                self.visibility(&item.vis);
                self.signature(&item.sig);
                self.where_clause_semi(&item.sig.generics.where_clause);
                self.end();
            }
            ItemVerbatim::StaticUntyped(item) => {
                self.outer_attrs(&item.attrs);
                self.cbox(0);
                self.visibility(&item.vis);
                self.word("static ");
                if let StaticMutability::Mut(_) = item.mutability {
                    self.word("mut ");
                }
                self.ident(&item.ident);
                self.word(" = ");
                self.neverbreak();
                self.expr(&item.expr);
                self.word(";");
                self.end();
            }
            ItemVerbatim::UseBrace(item) => {
                self.outer_attrs(&item.attrs);
                self.visibility(&item.vis);
                self.word("use ");
                if item.trees.len() == 1 {
                    self.word("::");
                    self.use_tree(&item.trees[0].inner);
                } else {
                    self.cbox(INDENT);
                    self.word("{");
                    self.zerobreak();
                    self.ibox(0);
                    for use_tree in item.trees.iter().delimited() {
                        if use_tree.leading_colon.is_some() {
                            self.word("::");
                        }
                        self.use_tree(&use_tree.inner);
                        if !use_tree.is_last {
                            self.word(",");
                            let mut use_tree = &use_tree.inner;
                            while let UseTree::Path(use_path) = use_tree {
                                use_tree = &use_path.tree;
                            }
                            if let UseTree::Group(_) = use_tree {
                                self.hardbreak();
                            } else {
                                self.space();
                            }
                        }
                    }
                    self.end();
                    self.trailing_comma(true);
                    self.offset(-INDENT);
                    self.word("}");
                    self.end();
                }
                self.word(";");
            }
        }

        self.hardbreak();
    }

    fn use_tree(&mut self, use_tree: &UseTree) {
        match use_tree {
            UseTree::Path(use_path) => self.use_path(use_path),
            UseTree::Name(use_name) => self.use_name(use_name),
            UseTree::Rename(use_rename) => self.use_rename(use_rename),
            UseTree::Glob(use_glob) => self.use_glob(use_glob),
            UseTree::Group(use_group) => self.use_group(use_group),
        }
    }

    fn use_path(&mut self, use_path: &UsePath) {
        self.ident(&use_path.ident);
        self.word("::");
        self.use_tree(&use_path.tree);
    }

    fn use_name(&mut self, use_name: &UseName) {
        self.ident(&use_name.ident);
    }

    fn use_rename(&mut self, use_rename: &UseRename) {
        self.ident(&use_rename.ident);
        self.word(" as ");
        self.ident(&use_rename.rename);
    }

    fn use_glob(&mut self, use_glob: &UseGlob) {
        let _ = use_glob;
        self.word("*");
    }

    fn use_group(&mut self, use_group: &UseGroup) {
        if use_group.items.is_empty() {
            self.word("{}");
        } else if use_group.items.len() == 1 {
            self.use_tree(&use_group.items[0]);
        } else {
            self.cbox(INDENT);
            self.word("{");
            self.zerobreak();
            self.ibox(0);
            for use_tree in use_group.items.iter().delimited() {
                self.use_tree(&use_tree);
                if !use_tree.is_last {
                    self.word(",");
                    let mut use_tree = *use_tree;
                    while let UseTree::Path(use_path) = use_tree {
                        use_tree = &use_path.tree;
                    }
                    if let UseTree::Group(_) = use_tree {
                        self.hardbreak();
                    } else {
                        self.space();
                    }
                }
            }
            self.end();
            self.trailing_comma(true);
            self.offset(-INDENT);
            self.word("}");
            self.end();
        }
    }

    fn foreign_item(&mut self, foreign_item: &ForeignItem) {
        match foreign_item {
            ForeignItem::Fn(item) => self.foreign_item_fn(item),
            ForeignItem::Static(item) => self.foreign_item_static(item),
            ForeignItem::Type(item) => self.foreign_item_type(item),
            ForeignItem::Macro(item) => self.foreign_item_macro(item),
            ForeignItem::Verbatim(item) => self.foreign_item_verbatim(item),
            #[cfg_attr(all(test, exhaustive), deny(non_exhaustive_omitted_patterns))]
            _ => unimplemented!("unknown ForeignItem"),
        }
    }

    fn foreign_item_fn(&mut self, foreign_item: &ForeignItemFn) {
        self.outer_attrs(&foreign_item.attrs);
        self.cbox(INDENT);
        self.visibility(&foreign_item.vis);
        self.signature(&foreign_item.sig);
        self.where_clause_semi(&foreign_item.sig.generics.where_clause);
        self.end();
        self.hardbreak();
    }

    fn foreign_item_static(&mut self, foreign_item: &ForeignItemStatic) {
        self.outer_attrs(&foreign_item.attrs);
        self.cbox(0);
        self.visibility(&foreign_item.vis);
        self.word("static ");
        if let StaticMutability::Mut(_) = foreign_item.mutability {
            self.word("mut ");
        }
        self.ident(&foreign_item.ident);
        self.word(": ");
        self.ty(&foreign_item.ty);
        self.word(";");
        self.end();
        self.hardbreak();
    }

    fn foreign_item_type(&mut self, foreign_item: &ForeignItemType) {
        self.outer_attrs(&foreign_item.attrs);
        self.cbox(0);
        self.visibility(&foreign_item.vis);
        self.word("type ");
        self.ident(&foreign_item.ident);
        self.generics(&foreign_item.generics);
        self.word(";");
        self.end();
        self.hardbreak();
    }

    fn foreign_item_macro(&mut self, foreign_item: &ForeignItemMacro) {
        self.outer_attrs(&foreign_item.attrs);
        self.mac(&foreign_item.mac, None);
        self.mac_semi_if_needed(&foreign_item.mac.delimiter);
        self.hardbreak();
    }

    #[cfg(not(feature = "verbatim"))]
    fn foreign_item_verbatim(&mut self, foreign_item: &TokenStream) {
        if !foreign_item.is_empty() {
            unimplemented!("ForeignItem::Verbatim `{}`", foreign_item);
        }
        self.hardbreak();
    }

    #[cfg(feature = "verbatim")]
    fn foreign_item_verbatim(&mut self, tokens: &TokenStream) {
        use syn::parse::{Parse, ParseStream, Result};

        enum ForeignItemVerbatim {
            TypeAlias(ItemType),
        }

        impl Parse for ForeignItemVerbatim {
            fn parse(input: ParseStream) -> Result<Self> {
                input.parse().map(ForeignItemVerbatim::TypeAlias)
            }
        }

        let foreign_item: ForeignItemVerbatim = match syn::parse2(tokens.clone()) {
            Ok(foreign_item) => foreign_item,
            Err(_) => unimplemented!("ForeignItem::Verbatim `{}`", tokens),
        };

        match foreign_item {
            ForeignItemVerbatim::TypeAlias(item) => self.item_type(&item),
        }
    }

    fn trait_item(&mut self, trait_item: &TraitItem) {
        match trait_item {
            TraitItem::Const(item) => self.trait_item_const(item),
            TraitItem::Fn(item) => self.trait_item_fn(item),
            TraitItem::Type(item) => self.trait_item_type(item),
            TraitItem::Macro(item) => self.trait_item_macro(item),
            TraitItem::Verbatim(item) => self.trait_item_verbatim(item),
            #[cfg_attr(all(test, exhaustive), deny(non_exhaustive_omitted_patterns))]
            _ => unimplemented!("unknown TraitItem"),
        }
    }

    fn trait_item_const(&mut self, trait_item: &TraitItemConst) {
        self.outer_attrs(&trait_item.attrs);
        self.cbox(0);
        self.word("const ");
        self.ident(&trait_item.ident);
        self.generics(&trait_item.generics);
        self.word(": ");
        self.ty(&trait_item.ty);
        if let Some((_eq_token, default)) = &trait_item.default {
            self.word(" = ");
            self.neverbreak();
            self.expr(default);
        }
        self.word(";");
        self.end();
        self.hardbreak();
    }

    fn trait_item_fn(&mut self, trait_item: &TraitItemFn) {
        self.outer_attrs(&trait_item.attrs);
        self.cbox(INDENT);
        self.signature(&trait_item.sig);
        if let Some(block) = &trait_item.default {
            self.where_clause_for_body(&trait_item.sig.generics.where_clause);
            self.word("{");
            self.hardbreak_if_nonempty();
            self.inner_attrs(&trait_item.attrs);
            for stmt in &block.stmts {
                self.stmt(stmt);
            }
            self.offset(-INDENT);
            self.end();
            self.word("}");
        } else {
            self.where_clause_semi(&trait_item.sig.generics.where_clause);
            self.end();
        }
        self.hardbreak();
    }

    fn trait_item_type(&mut self, trait_item: &TraitItemType) {
        self.outer_attrs(&trait_item.attrs);
        self.cbox(INDENT);
        self.word("type ");
        self.ident(&trait_item.ident);
        self.generics(&trait_item.generics);
        for bound in trait_item.bounds.iter().delimited() {
            if bound.is_first {
                self.word(": ");
            } else {
                self.space();
                self.word("+ ");
            }
            self.type_param_bound(&bound);
        }
        if let Some((_eq_token, default)) = &trait_item.default {
            self.word(" = ");
            self.neverbreak();
            self.ty(default);
        }
        self.where_clause_oneline_semi(&trait_item.generics.where_clause);
        self.end();
        self.hardbreak();
    }

    fn trait_item_macro(&mut self, trait_item: &TraitItemMacro) {
        self.outer_attrs(&trait_item.attrs);
        self.mac(&trait_item.mac, None);
        self.mac_semi_if_needed(&trait_item.mac.delimiter);
        self.hardbreak();
    }

    fn trait_item_verbatim(&mut self, trait_item: &TokenStream) {
        if !trait_item.is_empty() {
            unimplemented!("TraitItem::Verbatim `{}`", trait_item);
        }
        self.hardbreak();
    }

    fn impl_item(&mut self, impl_item: &ImplItem) {
        match impl_item {
            ImplItem::Const(item) => self.impl_item_const(item),
            ImplItem::Fn(item) => self.impl_item_fn(item),
            ImplItem::Type(item) => self.impl_item_type(item),
            ImplItem::Macro(item) => self.impl_item_macro(item),
            ImplItem::Verbatim(item) => self.impl_item_verbatim(item),
            #[cfg_attr(all(test, exhaustive), deny(non_exhaustive_omitted_patterns))]
            _ => unimplemented!("unknown ImplItem"),
        }
    }

    fn impl_item_const(&mut self, impl_item: &ImplItemConst) {
        self.outer_attrs(&impl_item.attrs);
        self.cbox(0);
        self.visibility(&impl_item.vis);
        if impl_item.defaultness.is_some() {
            self.word("default ");
        }
        self.word("const ");
        self.ident(&impl_item.ident);
        self.generics(&impl_item.generics);
        self.word(": ");
        self.ty(&impl_item.ty);
        self.word(" = ");
        self.neverbreak();
        self.expr(&impl_item.expr);
        self.word(";");
        self.end();
        self.hardbreak();
    }

    fn impl_item_fn(&mut self, impl_item: &ImplItemFn) {
        self.outer_attrs(&impl_item.attrs);
        self.cbox(INDENT);
        self.visibility(&impl_item.vis);
        if impl_item.defaultness.is_some() {
            self.word("default ");
        }
        self.signature(&impl_item.sig);
        if impl_item.block.stmts.len() == 1 {
            if let Stmt::Item(Item::Verbatim(verbatim)) = &impl_item.block.stmts[0] {
                if verbatim.to_string() == ";" {
                    self.where_clause_semi(&impl_item.sig.generics.where_clause);
                    self.end();
                    self.hardbreak();
                    return;
                }
            }
        }
        self.where_clause_for_body(&impl_item.sig.generics.where_clause);
        self.word("{");
        self.hardbreak_if_nonempty();
        self.inner_attrs(&impl_item.attrs);
        for stmt in &impl_item.block.stmts {
            self.stmt(stmt);
        }
        self.offset(-INDENT);
        self.end();
        self.word("}");
        self.hardbreak();
    }

    fn impl_item_type(&mut self, impl_item: &ImplItemType) {
        self.outer_attrs(&impl_item.attrs);
        self.cbox(INDENT);
        self.visibility(&impl_item.vis);
        if impl_item.defaultness.is_some() {
            self.word("default ");
        }
        self.word("type ");
        self.ident(&impl_item.ident);
        self.generics(&impl_item.generics);
        self.word(" = ");
        self.neverbreak();
        self.ibox(-INDENT);
        self.ty(&impl_item.ty);
        self.end();
        self.where_clause_oneline_semi(&impl_item.generics.where_clause);
        self.end();
        self.hardbreak();
    }

    fn impl_item_macro(&mut self, impl_item: &ImplItemMacro) {
        self.outer_attrs(&impl_item.attrs);
        self.mac(&impl_item.mac, None);
        self.mac_semi_if_needed(&impl_item.mac.delimiter);
        self.hardbreak();
    }

    fn impl_item_verbatim(&mut self, impl_item: &TokenStream) {
        if !impl_item.is_empty() {
            unimplemented!("ImplItem::Verbatim `{}`", impl_item);
        }
        self.hardbreak();
    }

    fn signature(&mut self, signature: &Signature) {
        if signature.constness.is_some() {
            self.word("const ");
        }
        if signature.asyncness.is_some() {
            self.word("async ");
        }
        if signature.unsafety.is_some() {
            self.word("unsafe ");
        }
        if let Some(abi) = &signature.abi {
            self.abi(abi);
        }
        self.word("fn ");
        self.ident(&signature.ident);
        self.generics(&signature.generics);
        self.word("(");
        self.neverbreak();
        self.cbox(0);
        self.zerobreak();
        for input in signature.inputs.iter().delimited() {
            self.fn_arg(&input);
            let is_last = input.is_last && signature.variadic.is_none();
            self.trailing_comma(is_last);
        }
        if let Some(variadic) = &signature.variadic {
            self.variadic(variadic);
            self.zerobreak();
        }
        self.offset(-INDENT);
        self.end();
        self.word(")");
        self.cbox(-INDENT);
        self.return_type(&signature.output);
        self.end();
    }

    fn fn_arg(&mut self, fn_arg: &FnArg) {
        match fn_arg {
            FnArg::Receiver(receiver) => self.receiver(receiver),
            FnArg::Typed(pat_type) => self.pat_type(pat_type),
        }
    }

    fn receiver(&mut self, receiver: &Receiver) {
        self.outer_attrs(&receiver.attrs);
        if let Some((_ampersand, lifetime)) = &receiver.reference {
            self.word("&");
            if let Some(lifetime) = lifetime {
                self.lifetime(lifetime);
                self.nbsp();
            }
        }
        if receiver.mutability.is_some() {
            self.word("mut ");
        }
        self.word("self");
        if receiver.colon_token.is_some() {
            self.word(": ");
            self.ty(&receiver.ty);
        } else {
            let consistent = match (&receiver.reference, &receiver.mutability, &*receiver.ty) {
                (Some(_), mutability, Type::Reference(ty)) => {
                    mutability.is_some() == ty.mutability.is_some()
                        && match &*ty.elem {
                            Type::Path(ty) => ty.qself.is_none() && ty.path.is_ident("Self"),
                            _ => false,
                        }
                }
                (None, _, Type::Path(ty)) => ty.qself.is_none() && ty.path.is_ident("Self"),
                _ => false,
            };
            if !consistent {
                self.word(": ");
                self.ty(&receiver.ty);
            }
        }
    }

    fn variadic(&mut self, variadic: &Variadic) {
        self.outer_attrs(&variadic.attrs);
        if let Some((pat, _colon)) = &variadic.pat {
            self.pat(pat);
            self.word(": ");
        }
        self.word("...");
    }
}
