use crate::model::Field;
use proc_macro2::TokenStream;

mod array;
mod hidden_controller;
mod list;
mod normal;
mod option;
mod padding;

impl Field {
    pub fn read_code(&self, struct_ident: &syn::Ident) -> TokenStream {
        let struct_name = proc_macro2::Literal::string(&struct_ident.to_string());
        match self {
            Field::Normal(normal_field) => normal::read(normal_field, &struct_name),
            Field::PaddBits(n_bits) => padding::read(*n_bits, &struct_name),
            Field::Option {
                inner_type,
                controller,
                ..
            } => option::read(inner_type, controller, &struct_name),
            Field::List {
                inner_type,
                controller,
                ..
            } => list::read(inner_type, controller, &struct_name),
            Field::HiddenController { field, .. } => normal::read(field, &struct_name),
            Field::Array {
                length,
                inner_type,
                field,
            } => array::read(length, inner_type, field, &struct_name),
        }
    }

    pub fn write_code(&self, struct_ident: &syn::Ident) -> TokenStream {
        let struct_name = proc_macro2::Literal::string(&struct_ident.to_string());
        match self {
            Field::Normal(normal_field) => normal::write(normal_field),
            Field::PaddBits(n_bits) => padding::write(*n_bits, &struct_name),
            Field::Option { inner_type, .. } => option::write(inner_type),
            Field::List { inner_type, .. } => list::write(inner_type),
            Field::HiddenController {
                field,
                controlled,
                presence,
            } => hidden_controller::write(field, controlled, *presence),
            Field::Array { field, .. } => array::write(field),
        }
    }

    pub fn min_bits_code(&self) -> TokenStream {
        match self {
            Field::Normal(normal_field) => normal::min_bits(normal_field),
            Field::PaddBits(n_bits) => padding::min_bits(*n_bits),
            Field::Option { inner_type, .. } => option::min_bits(inner_type),
            Field::List { inner_type, .. } => list::min_bits(inner_type),
            Field::HiddenController { field, .. } => normal::min_bits(field),
            Field::Array {
                inner_type, length, ..
            } => array::min_bits(inner_type, length),
        }
    }

    pub fn max_bits_code(&self) -> TokenStream {
        match self {
            Field::Normal(normal_field) => normal::max_bits(normal_field),
            Field::PaddBits(n_bits) => padding::max_bits(*n_bits),
            Field::Option { inner_type, .. } => option::max_bits(inner_type),
            Field::List {
                inner_type,
                max_len,
                ..
            } => list::max_bits(inner_type, *max_len),
            Field::HiddenController { field, .. } => normal::max_bits(field),
            Field::Array {
                inner_type, length, ..
            } => array::max_bits(inner_type, length),
        }
    }
}
