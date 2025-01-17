use crate::StructHelpers;

use super::super::utilities::filter::{get_raw_type_name, is_option, is_vec};
use darling::{ast, util};
use proc_macro2::Ident;
use quote::format_ident;
use syn::Attribute;

pub struct FieldInfo {
    pub ident: Ident,
    pub attr: Vec<Attribute>,
}

pub trait FieldInfoExtract {
    fn get_field_names<'a>(&'a self) -> Vec<&'a Ident>;
}

impl FieldInfoExtract for Vec<FieldInfo> {
    fn get_field_names<'a>(&'a self) -> Vec<&'a Ident> {
        self.iter().map(|field| &field.ident).collect()
    }
}

pub struct StructFields {
    // [Name]: Type
    pub field_names: Vec<FieldInfo>,
    pub field_vec_names: Vec<FieldInfo>,
    pub field_option_names: Vec<FieldInfo>,

    // Field: [Type] -> Ident
    pub field_types_names: Vec<proc_macro2::Ident>,
    pub field_vec_types_names: Vec<proc_macro2::Ident>,
    pub field_option_types_names: Vec<proc_macro2::Ident>,

    // Field(Builder): Type
    pub field_builder_names: Vec<proc_macro2::Ident>,
    pub field_vec_builder_names: Vec<proc_macro2::Ident>,
    pub field_option_builder_names: Vec<proc_macro2::Ident>,
}

impl StructFields {
    pub fn get_field_names<'a>(&'a self) -> Vec<&'a Ident> {
        let mut ret = vec![];
        ret.extend(self.field_names.get_field_names());
        ret.extend(self.field_vec_names.get_field_names());
        ret.extend(self.field_option_names.get_field_names());
        ret
    }

    pub fn get_field_types<'a>(&'a self) -> Vec<&'a Ident> {
        let mut ret = vec![];
        ret.extend(&self.field_types_names);
        ret.extend(&self.field_vec_types_names);
        ret.extend(&self.field_option_types_names);
        ret
    }

    pub fn get_field_builder_names<'a>(&'a self) -> Vec<&'a Ident> {
        let mut ret = vec![];
        ret.extend(&self.field_builder_names);
        ret.extend(&self.field_vec_builder_names);
        ret.extend(&self.field_option_builder_names);
        ret
    }
}

pub fn match_struct_fields(data: &ast::Data<util::Ignored, StructHelpers>) -> StructFields {
    let mut ret_fields = StructFields {
        field_names: vec![],
        field_vec_names: vec![],
        field_option_names: vec![],

        field_types_names: vec![],
        field_vec_types_names: vec![],
        field_option_types_names: vec![],

        field_builder_names: vec![],
        field_vec_builder_names: vec![],
        field_option_builder_names: vec![],
    };

    data.as_ref()
        .take_struct()
        .unwrap()
        .fields
        .iter()
        .for_each(|field| {
            if let true = is_vec(&field.ty) {
                ret_fields.field_vec_names.push(FieldInfo {
                    ident: field.ident.as_ref().unwrap().clone(),
                    attr: vec![],
                });
                ret_fields
                    .field_vec_types_names
                    .push(format_ident!("{}", get_raw_type_name(&field.ty)));
                ret_fields
                    .field_vec_builder_names
                    .push(format_ident!("{}Builder", get_raw_type_name(&field.ty)));
            } else if let true = is_option(&field.ty) {
                ret_fields.field_option_names.push(FieldInfo {
                    ident: field.ident.as_ref().unwrap().clone(),
                    attr: vec![],
                });
                ret_fields
                    .field_option_types_names
                    .push(format_ident!("{}", get_raw_type_name(&field.ty)));
                ret_fields
                    .field_option_builder_names
                    .push(format_ident!("{}Builder", get_raw_type_name(&field.ty)));
            } else {
                ret_fields.field_names.push(FieldInfo {
                    ident: field.ident.as_ref().unwrap().clone(),
                    attr: vec![],
                });
                ret_fields
                    .field_types_names
                    .push(format_ident!("{}", get_raw_type_name(&field.ty)));
                ret_fields
                    .field_builder_names
                    .push(format_ident!("{}Builder", get_raw_type_name(&field.ty)));
            }
        });
    ret_fields
}

pub struct EnumFields {
    // [Name]: Type
    pub variant_names: Vec<proc_macro2::Ident>,

    // Variant: [Type] -> Ident
    pub variant_types_names: Vec<proc_macro2::Ident>,

    // Variant(Builder): Type
    pub variant_builder_names: Vec<proc_macro2::Ident>,
}

pub fn match_enum_fields(data: &syn::Data) -> EnumFields {
    let mut ret_fields = EnumFields {
        variant_names: vec![],

        variant_types_names: vec![],

        variant_builder_names: vec![],
    };
    match data {
        syn::Data::Enum(ref enum_data) => {
            for variant in &enum_data.variants {
                let variant_name = &variant.ident;
                match &variant.fields {
                    syn::Fields::Unnamed(fields) => {
                        let first_field = fields.unnamed.first().unwrap();
                        ret_fields.variant_names.push(variant_name.clone());
                        ret_fields
                            .variant_types_names
                            .push(format_ident!("{}", get_raw_type_name(&first_field.ty)));
                        ret_fields.variant_builder_names.push(format_ident!(
                            "{}Builder",
                            get_raw_type_name(&first_field.ty)
                        ));
                    }
                    _ => panic!("This proc macro only works with enums"),
                }
            }
        }
        _ => panic!("This proc macro only works with enums"),
    }

    ret_fields
}
