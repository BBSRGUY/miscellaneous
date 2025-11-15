//! Item (declaration) parsing.

use crate::error::{ParseError, ParseErrorKind, ParseResult};
use crate::parser::Parser;
use blang_ast::*;
use blang_lexer::TokenKind;

impl<'a> Parser<'a> {
    /// Parse an item (top-level declaration).
    pub(crate) fn parse_item(&mut self) -> ParseResult<Item> {
        let start = self.stream.current_span().start;

        // Parse attributes
        let attrs = self.parse_attrs()?;

        // Check for visibility
        let public = self.stream.eat(TokenKind::Pub);

        let kind = match self.stream.peek_kind() {
            TokenKind::Fn => self.parse_function(attrs, public)?,
            TokenKind::Struct => self.parse_struct(attrs, public)?,
            TokenKind::Enum => self.parse_enum(attrs, public)?,
            TokenKind::Trait => self.parse_trait(attrs, public)?,
            TokenKind::Impl => self.parse_impl(attrs)?,
            TokenKind::Type => self.parse_type_alias(attrs, public)?,
            TokenKind::Const => self.parse_const(attrs, public)?,
            TokenKind::Static => self.parse_static(attrs, public)?,
            TokenKind::Module => self.parse_module(attrs, public)?,
            TokenKind::Use => self.parse_use(attrs)?,
            TokenKind::Component => self.parse_component(attrs)?,
            TokenKind::Script => self.parse_script(attrs)?,
            _ => {
                return Err(ParseError::new(
                    ParseErrorKind::ExpectedItem,
                    self.stream.current_span(),
                ));
            }
        };

        let end = self.stream.last_span().end;
        Ok(Item::new(kind, start.to(end)))
    }

    /// Parse attributes.
    fn parse_attrs(&mut self) -> ParseResult<Vec<Attr>> {
        let mut attrs = Vec::new();

        while self.stream.at(TokenKind::Hash) {
            attrs.push(self.parse_attr()?);
        }

        Ok(attrs)
    }

    /// Parse a single attribute.
    fn parse_attr(&mut self) -> ParseResult<Attr> {
        let start = self.stream.current_span().start;

        self.stream.expect(TokenKind::Hash).map_err(|_| {
            ParseError::missing_token(TokenKind::Hash, self.stream.current_span())
        })?;

        self.stream.expect(TokenKind::OpenBracket).map_err(|_| {
            ParseError::missing_token(TokenKind::OpenBracket, self.stream.current_span())
        })?;

        let ident = self.parse_ident()?;
        let path = AttrPath::Simple(ident);

        let args = if self.stream.eat(TokenKind::OpenParen) {
            let mut arg_list = Vec::new();

            while !self.stream.at(TokenKind::CloseParen) {
                let ident = self.parse_ident()?;
                arg_list.push(AttrArg::Ident(ident));

                if !self.stream.eat(TokenKind::Comma) {
                    break;
                }
            }

            self.stream.expect(TokenKind::CloseParen).map_err(|_| {
                ParseError::missing_token(TokenKind::CloseParen, self.stream.current_span())
            })?;

            Some(AttrArgs::List(arg_list))
        } else {
            None
        };

        self.stream.expect(TokenKind::CloseBracket).map_err(|_| {
            ParseError::missing_token(TokenKind::CloseBracket, self.stream.current_span())
        })?;

        let end = self.stream.last_span().end;
        Ok(Attr::new(path, args, start.to(end)))
    }

    /// Parse a function declaration.
    fn parse_function(&mut self, attrs: Vec<Attr>, public: bool) -> ParseResult<ItemKind> {
        self.stream.expect(TokenKind::Fn).map_err(|_| {
            ParseError::missing_token(TokenKind::Fn, self.stream.current_span())
        })?;

        let unsafe_ = self.stream.eat(TokenKind::Unsafe);
        let name = self.parse_ident()?;
        let generic_params = self.parse_generic_params()?;

        self.stream.expect(TokenKind::OpenParen).map_err(|_| {
            ParseError::missing_token(TokenKind::OpenParen, self.stream.current_span())
        })?;

        let params = self.parse_params()?;

        self.stream.expect(TokenKind::CloseParen).map_err(|_| {
            ParseError::missing_token(TokenKind::CloseParen, self.stream.current_span())
        })?;

        let return_ty = if self.stream.eat(TokenKind::Arrow) {
            Some(self.parse_ty()?)
        } else {
            None
        };

        let body = if self.stream.at(TokenKind::OpenBrace) {
            Some(self.parse_block()?)
        } else {
            self.expect_semi()?;
            None
        };

        Ok(ItemKind::Function(FunctionDecl {
            attrs,
            sig: FunctionSig {
                public,
                unsafe_,
                name,
                generic_params,
                params,
                return_ty,
            },
            body,
        }))
    }

    /// Parse generic parameters.
    fn parse_generic_params(&mut self) -> ParseResult<Vec<GenericParam>> {
        if !self.stream.eat(TokenKind::Lt) {
            return Ok(Vec::new());
        }

        let mut params = Vec::new();

        loop {
            let start = self.stream.current_span().start;
            let name = self.parse_ident()?;

            let bounds = if self.stream.eat(TokenKind::Colon) {
                self.parse_trait_bounds()?
            } else {
                Vec::new()
            };

            let end = self.stream.last_span().end;
            params.push(GenericParam {
                name,
                bounds,
                span: start.to(end),
            });

            if !self.stream.eat(TokenKind::Comma) {
                break;
            }
        }

        self.stream.expect(TokenKind::Gt).map_err(|_| {
            ParseError::missing_token(TokenKind::Gt, self.stream.current_span())
        })?;

        Ok(params)
    }

    /// Parse trait bounds.
    fn parse_trait_bounds(&mut self) -> ParseResult<Vec<TraitBound>> {
        let mut bounds = Vec::new();

        loop {
            let start = self.stream.current_span().start;
            let trait_path = self.parse_path()?;
            let end = self.stream.last_span().end;

            bounds.push(TraitBound {
                trait_path,
                span: start.to(end),
            });

            if !self.stream.eat(TokenKind::Plus) {
                break;
            }
        }

        Ok(bounds)
    }

    /// Parse function parameters.
    fn parse_params(&mut self) -> ParseResult<Vec<Param>> {
        let mut params = Vec::new();

        if self.stream.at(TokenKind::CloseParen) {
            return Ok(params);
        }

        loop {
            let pat = self.parse_pat()?;

            self.stream.expect(TokenKind::Colon).map_err(|_| {
                ParseError::missing_token(TokenKind::Colon, self.stream.current_span())
            })?;

            let ty = self.parse_ty()?;

            params.push(Param { pat, ty });

            if !self.stream.eat(TokenKind::Comma) {
                break;
            }

            if self.stream.at(TokenKind::CloseParen) {
                break;
            }
        }

        Ok(params)
    }

    /// Parse a struct declaration.
    fn parse_struct(&mut self, attrs: Vec<Attr>, public: bool) -> ParseResult<ItemKind> {
        self.stream.expect(TokenKind::Struct).map_err(|_| {
            ParseError::missing_token(TokenKind::Struct, self.stream.current_span())
        })?;

        let name = self.parse_ident()?;
        let generic_params = self.parse_generic_params()?;

        let body = match self.stream.peek_kind() {
            TokenKind::OpenBrace => {
                self.stream.next();
                let fields = self.parse_struct_fields()?;
                self.stream.expect(TokenKind::CloseBrace).map_err(|_| {
                    ParseError::missing_token(TokenKind::CloseBrace, self.stream.current_span())
                })?;
                StructBody::Named(fields)
            }

            TokenKind::OpenParen => {
                self.stream.next();
                let fields = self.parse_tuple_fields()?;
                self.stream.expect(TokenKind::CloseParen).map_err(|_| {
                    ParseError::missing_token(TokenKind::CloseParen, self.stream.current_span())
                })?;
                self.expect_semi()?;
                StructBody::Tuple(fields)
            }

            TokenKind::Semi => {
                self.stream.next();
                StructBody::Unit
            }

            _ => {
                return Err(ParseError::new(
                    ParseErrorKind::Expected {
                        message: "struct body".to_string(),
                    },
                    self.stream.current_span(),
                ));
            }
        };

        Ok(ItemKind::Struct(StructDecl {
            attrs,
            public,
            name,
            generic_params,
            body,
        }))
    }

    /// Parse struct fields.
    fn parse_struct_fields(&mut self) -> ParseResult<Vec<StructField>> {
        let mut fields = Vec::new();

        while !self.stream.at(TokenKind::CloseBrace) {
            let attrs = self.parse_attrs()?;
            let public = self.stream.eat(TokenKind::Pub);
            let name = self.parse_ident()?;

            self.stream.expect(TokenKind::Colon).map_err(|_| {
                ParseError::missing_token(TokenKind::Colon, self.stream.current_span())
            })?;

            let ty = self.parse_ty()?;

            fields.push(StructField {
                attrs,
                public,
                name,
                ty,
            });

            if !self.stream.eat(TokenKind::Comma) {
                break;
            }
        }

        Ok(fields)
    }

    /// Parse tuple fields.
    fn parse_tuple_fields(&mut self) -> ParseResult<Vec<TupleField>> {
        let mut fields = Vec::new();

        while !self.stream.at(TokenKind::CloseParen) {
            let attrs = self.parse_attrs()?;
            let public = self.stream.eat(TokenKind::Pub);
            let ty = self.parse_ty()?;

            fields.push(TupleField { attrs, public, ty });

            if !self.stream.eat(TokenKind::Comma) {
                break;
            }
        }

        Ok(fields)
    }

    /// Parse an enum declaration.
    fn parse_enum(&mut self, attrs: Vec<Attr>, public: bool) -> ParseResult<ItemKind> {
        self.stream.expect(TokenKind::Enum).map_err(|_| {
            ParseError::missing_token(TokenKind::Enum, self.stream.current_span())
        })?;

        let name = self.parse_ident()?;
        let generic_params = self.parse_generic_params()?;

        self.stream.expect(TokenKind::OpenBrace).map_err(|_| {
            ParseError::missing_token(TokenKind::OpenBrace, self.stream.current_span())
        })?;

        let mut variants = Vec::new();

        while !self.stream.at(TokenKind::CloseBrace) {
            let variant_attrs = self.parse_attrs()?;
            let variant_name = self.parse_ident()?;

            let body = match self.stream.peek_kind() {
                TokenKind::OpenBrace => {
                    self.stream.next();
                    let fields = self.parse_struct_fields()?;
                    self.stream.expect(TokenKind::CloseBrace).map_err(|_| {
                        ParseError::missing_token(TokenKind::CloseBrace, self.stream.current_span())
                    })?;
                    EnumVariantBody::Struct(fields)
                }

                TokenKind::OpenParen => {
                    self.stream.next();
                    let fields = self.parse_tuple_fields()?;
                    self.stream.expect(TokenKind::CloseParen).map_err(|_| {
                        ParseError::missing_token(TokenKind::CloseParen, self.stream.current_span())
                    })?;
                    EnumVariantBody::Tuple(fields)
                }

                _ => EnumVariantBody::Unit,
            };

            variants.push(EnumVariant {
                attrs: variant_attrs,
                name: variant_name,
                body,
            });

            if !self.stream.eat(TokenKind::Comma) {
                break;
            }
        }

        self.stream.expect(TokenKind::CloseBrace).map_err(|_| {
            ParseError::missing_token(TokenKind::CloseBrace, self.stream.current_span())
        })?;

        Ok(ItemKind::Enum(EnumDecl {
            attrs,
            public,
            name,
            generic_params,
            variants,
        }))
    }

    /// Parse a trait declaration (simplified).
    fn parse_trait(&mut self, attrs: Vec<Attr>, public: bool) -> ParseResult<ItemKind> {
        self.stream.expect(TokenKind::Trait).map_err(|_| {
            ParseError::missing_token(TokenKind::Trait, self.stream.current_span())
        })?;

        let name = self.parse_ident()?;
        let generic_params = self.parse_generic_params()?;
        let supertraits = Vec::new(); // Simplified

        self.stream.expect(TokenKind::OpenBrace).map_err(|_| {
            ParseError::missing_token(TokenKind::OpenBrace, self.stream.current_span())
        })?;

        let items = Vec::new(); // Simplified

        self.stream.expect(TokenKind::CloseBrace).map_err(|_| {
            ParseError::missing_token(TokenKind::CloseBrace, self.stream.current_span())
        })?;

        Ok(ItemKind::Trait(TraitDecl {
            attrs,
            public,
            name,
            generic_params,
            supertraits,
            items,
        }))
    }

    /// Parse an impl block (simplified).
    fn parse_impl(&mut self, attrs: Vec<Attr>) -> ParseResult<ItemKind> {
        self.stream.expect(TokenKind::Impl).map_err(|_| {
            ParseError::missing_token(TokenKind::Impl, self.stream.current_span())
        })?;

        let generic_params = self.parse_generic_params()?;
        let self_ty = self.parse_ty()?;

        self.stream.expect(TokenKind::OpenBrace).map_err(|_| {
            ParseError::missing_token(TokenKind::OpenBrace, self.stream.current_span())
        })?;

        let items = Vec::new(); // Simplified

        self.stream.expect(TokenKind::CloseBrace).map_err(|_| {
            ParseError::missing_token(TokenKind::CloseBrace, self.stream.current_span())
        })?;

        Ok(ItemKind::Impl(ImplDecl {
            attrs,
            generic_params,
            trait_: None,
            self_ty,
            items,
        }))
    }

    /// Parse a type alias.
    fn parse_type_alias(&mut self, attrs: Vec<Attr>, public: bool) -> ParseResult<ItemKind> {
        self.stream.expect(TokenKind::Type).map_err(|_| {
            ParseError::missing_token(TokenKind::Type, self.stream.current_span())
        })?;

        let name = self.parse_ident()?;
        let generic_params = self.parse_generic_params()?;

        self.stream.expect(TokenKind::Eq).map_err(|_| {
            ParseError::missing_token(TokenKind::Eq, self.stream.current_span())
        })?;

        let ty = self.parse_ty()?;

        self.expect_semi()?;

        Ok(ItemKind::TypeAlias(TypeAliasDecl {
            attrs,
            public,
            name,
            generic_params,
            ty,
        }))
    }

    /// Parse a const declaration.
    fn parse_const(&mut self, attrs: Vec<Attr>, public: bool) -> ParseResult<ItemKind> {
        self.stream.expect(TokenKind::Const).map_err(|_| {
            ParseError::missing_token(TokenKind::Const, self.stream.current_span())
        })?;

        let name = self.parse_ident()?;

        self.stream.expect(TokenKind::Colon).map_err(|_| {
            ParseError::missing_token(TokenKind::Colon, self.stream.current_span())
        })?;

        let ty = self.parse_ty()?;

        self.stream.expect(TokenKind::Eq).map_err(|_| {
            ParseError::missing_token(TokenKind::Eq, self.stream.current_span())
        })?;

        let value = self.parse_expr()?;

        self.expect_semi()?;

        Ok(ItemKind::Const(ConstDecl {
            attrs,
            public,
            name,
            ty,
            value,
        }))
    }

    /// Parse a static declaration.
    fn parse_static(&mut self, attrs: Vec<Attr>, public: bool) -> ParseResult<ItemKind> {
        self.stream.expect(TokenKind::Static).map_err(|_| {
            ParseError::missing_token(TokenKind::Static, self.stream.current_span())
        })?;

        let mutable = self.stream.eat(TokenKind::Mut);
        let name = self.parse_ident()?;

        self.stream.expect(TokenKind::Colon).map_err(|_| {
            ParseError::missing_token(TokenKind::Colon, self.stream.current_span())
        })?;

        let ty = self.parse_ty()?;

        self.stream.expect(TokenKind::Eq).map_err(|_| {
            ParseError::missing_token(TokenKind::Eq, self.stream.current_span())
        })?;

        let value = self.parse_expr()?;

        self.expect_semi()?;

        Ok(ItemKind::Static(StaticDecl {
            attrs,
            public,
            mutable,
            name,
            ty,
            value,
        }))
    }

    /// Parse a module declaration (simplified).
    fn parse_module(&mut self, attrs: Vec<Attr>, public: bool) -> ParseResult<ItemKind> {
        self.stream.expect(TokenKind::Module).map_err(|_| {
            ParseError::missing_token(TokenKind::Module, self.stream.current_span())
        })?;

        let name = self.parse_ident()?;

        if self.stream.eat(TokenKind::Semi) {
            Ok(ItemKind::Module(ModuleDecl {
                attrs,
                public,
                name,
                items: None,
            }))
        } else {
            self.stream.expect(TokenKind::OpenBrace).map_err(|_| {
                ParseError::missing_token(TokenKind::OpenBrace, self.stream.current_span())
            })?;

            let mut items = Vec::new();
            while !self.stream.at(TokenKind::CloseBrace) {
                items.push(self.parse_item()?);
            }

            self.stream.expect(TokenKind::CloseBrace).map_err(|_| {
                ParseError::missing_token(TokenKind::CloseBrace, self.stream.current_span())
            })?;

            Ok(ItemKind::Module(ModuleDecl {
                attrs,
                public,
                name,
                items: Some(items),
            }))
        }
    }

    /// Parse a use declaration.
    fn parse_use(&mut self, _attrs: Vec<Attr>) -> ParseResult<ItemKind> {
        self.stream.expect(TokenKind::Use).map_err(|_| {
            ParseError::missing_token(TokenKind::Use, self.stream.current_span())
        })?;

        let path = self.parse_path()?;

        self.expect_semi()?;

        Ok(ItemKind::Use(UseDecl {
            tree: UseTree::Path(path),
        }))
    }

    /// Parse a component declaration (simplified).
    fn parse_component(&mut self, attrs: Vec<Attr>) -> ParseResult<ItemKind> {
        self.stream.expect(TokenKind::Component).map_err(|_| {
            ParseError::missing_token(TokenKind::Component, self.stream.current_span())
        })?;

        let name = self.parse_ident()?;
        let generic_params = self.parse_generic_params()?;

        self.stream.expect(TokenKind::OpenBrace).map_err(|_| {
            ParseError::missing_token(TokenKind::OpenBrace, self.stream.current_span())
        })?;

        let items = Vec::new(); // Simplified

        self.stream.expect(TokenKind::CloseBrace).map_err(|_| {
            ParseError::missing_token(TokenKind::CloseBrace, self.stream.current_span())
        })?;

        Ok(ItemKind::Component(ComponentDecl {
            attrs,
            name,
            generic_params,
            items,
        }))
    }

    /// Parse a script declaration (simplified).
    fn parse_script(&mut self, attrs: Vec<Attr>) -> ParseResult<ItemKind> {
        self.stream.expect(TokenKind::Script).map_err(|_| {
            ParseError::missing_token(TokenKind::Script, self.stream.current_span())
        })?;

        let name = self.parse_ident()?;

        self.stream.expect(TokenKind::OpenBrace).map_err(|_| {
            ParseError::missing_token(TokenKind::OpenBrace, self.stream.current_span())
        })?;

        let items = Vec::new(); // Simplified

        self.stream.expect(TokenKind::CloseBrace).map_err(|_| {
            ParseError::missing_token(TokenKind::CloseBrace, self.stream.current_span())
        })?;

        Ok(ItemKind::Script(ScriptDecl { attrs, name, items }))
    }
}
