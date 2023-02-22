//! Defines a slightly enhanced data structure than the base AST
//!
//! Specifically:
//! * attributes are separated into `simple_attributes` and `complex_attributes`
//!   struct fields as [IndexMap](std::collections::IndexMap)s.
//! * `cell` and `pin` groups are brought out into [IndexMap](std::collections::IndexMap)s so they're
//!   easier to work with

use std::{
    fmt,
    ops::{Deref, DerefMut},
};

use indexmap::IndexMap;

use crate::ast::{GroupItem, LibertyAst, Value};

/// Top-level data structure of a Liberty file
#[derive(Debug, PartialEq, Clone)]
pub struct Liberty(pub Vec<Group>);

impl Liberty {
    pub fn to_ast(self) -> LibertyAst {
        LibertyAst(self.0.into_iter().map(|g| g.into_group_item()).collect())
    }
    pub fn from_ast(ast: LibertyAst) -> Self {
        Liberty(
            ast.0
                .into_iter()
                .map(|g| Group::from_group_item(g))
                .collect(),
        )
    }
}

impl Deref for Liberty {
    type Target = [Group];

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl DerefMut for Liberty {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.deref_mut()
    }
}

impl From<LibertyAst> for Liberty {
    fn from(ast: LibertyAst) -> Self {
        Liberty::from_ast(ast)
    }
}

impl fmt::Display for Liberty {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.clone().to_ast().fmt(f)
    }
}

impl IntoIterator for Liberty {
    type Item = Group;
    type IntoIter = ::std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

/// General attribute struct
///
/// Attributes can be simple or complex
#[derive(Debug, PartialEq, Clone)]
pub enum Attribute {
    Simple(Value),
    Complex(Vec<Value>),
}

/// General group struct
///
/// Groups contain simple attributes, complex attributes, and other groups
#[derive(Debug, PartialEq, Clone)]
pub struct Group {
    pub type_: String,
    pub name: String,
    pub attributes: IndexMap<String, Attribute>,
    pub subgroups: Vec<Group>,
}

impl Group {
    /// Create a group with empty attributes and sub-groups
    pub fn new(type_: &str, name: &str) -> Self {
        Self {
            type_: type_.to_string(),
            name: name.to_string(),
            attributes: IndexMap::new(),
            subgroups: vec![],
        }
    }

    /// Convert an AST [GroupItem::Group] variant into a [Group] struct
    pub fn from_group_item(group_item: GroupItem) -> Self {
        let (type_, name, items) = group_item.group();
        let mut attributes: IndexMap<String, Attribute> = IndexMap::new();
        let mut subgroups = vec![];
        for item in items {
            match item {
                GroupItem::SimpleAttr(name, value) => {
                    attributes.insert(name, Attribute::Simple(value));
                }
                GroupItem::ComplexAttr(name, value) => {
                    attributes.insert(name, Attribute::Complex(value));
                }
                GroupItem::Group(type_, name, items) => {
                    subgroups
                        .push(Group::from_group_item(GroupItem::Group(type_, name, items)).into());
                }
                _ => {}
            }
        }
        Self {
            name,
            type_,
            attributes,
            subgroups,
        }
    }

    /// Convert a [Group] struct into a [GroupItem::Group] variant
    pub fn into_group_item(self) -> GroupItem {
        let mut items: Vec<GroupItem> =
            Vec::with_capacity(self.attributes.len() + self.subgroups.len());
        items.extend(self.attributes.into_iter().map(|(name, attr)| match attr {
            Attribute::Simple(value) => GroupItem::SimpleAttr(name, value),
            Attribute::Complex(value) => GroupItem::ComplexAttr(name, value),
        }));
        items.extend(self.subgroups.into_iter().map(|g| g.into_group_item()));
        GroupItem::Group(self.type_, self.name, items)
    }

    /// Get a complex attribute by name
    pub fn complex_attribute(&self, name: &str) -> Option<&Vec<Value>> {
        self.attributes.get(name).and_then(|attr| match attr {
            Attribute::Complex(value) => Some(value),
            _ => None,
        })
    }

    /// Get a simple attribute by name
    pub fn simple_attribute(&self, name: &str) -> Option<&Value> {
        self.attributes.get(name).and_then(|attr| match attr {
            Attribute::Simple(value) => Some(value),
            _ => None,
        })
    }

    /// Iterate over the complex attributes
    pub fn iter_complex_attributes(&self) -> impl Iterator<Item = (&String, &Vec<Value>)> {
        self.attributes
            .iter()
            .filter_map(|(name, attr)| match attr {
                Attribute::Complex(value) => Some((name, value)),
                _ => None,
            })
    }

    /// Iterate over the complex attributes mutably
    pub fn iter_complex_attributes_mut(
        &mut self,
    ) -> impl Iterator<Item = (&String, &mut Vec<Value>)> {
        self.attributes
            .iter_mut()
            .filter_map(|(name, attr)| match attr {
                Attribute::Complex(value) => Some((name, value)),
                _ => None,
            })
    }

    /// Iterate over the simple attributes
    pub fn iter_simple_attributes(&self) -> impl Iterator<Item = (&String, &Value)> {
        self.attributes
            .iter()
            .filter_map(|(name, attr)| match attr {
                Attribute::Simple(value) => Some((name, value)),
                _ => None,
            })
    }

    /// Iterate over the simple attributes mutably
    pub fn iter_simple_attributes_mut(&mut self) -> impl Iterator<Item = (&String, &mut Value)> {
        self.attributes
            .iter_mut()
            .filter_map(|(name, attr)| match attr {
                Attribute::Simple(value) => Some((name, value)),
                _ => None,
            })
    }

    /// Iterate over the subgroups of a given type
    pub fn iter_subgroups_of_type<'a>(&'a self, type_: &'a str) -> impl Iterator<Item = &'a Group> {
        self.subgroups.iter().filter(move |g| g.type_ == type_)
    }

    /// Iterate over the subgroups of a given type mutably
    pub fn iter_subgroups_of_type_mut<'a>(
        &'a mut self,
        type_: &'a str,
    ) -> impl Iterator<Item = &'a mut Group> {
        self.subgroups.iter_mut().filter(move |g| g.type_ == type_)
    }

    /// Iterate over the subgroups
    pub fn iter_subgroups(&self) -> impl Iterator<Item = &Group> {
        self.subgroups.iter()
    }

    /// Iterate over the subgroups mutably
    pub fn iter_subgroups_mut(&mut self) -> impl Iterator<Item = &mut Group> {
        self.subgroups.iter_mut()
    }

    /// Get cell by name
    pub fn get_cell(&self, name: &str) -> Option<&Group> {
        self.iter_subgroups_of_type("cell").find(|g| g.name == name)
    }

    /// Get cell by name mutably
    pub fn get_cell_mut(&mut self, name: &str) -> Option<&mut Group> {
        self.iter_subgroups_of_type_mut("cell")
            .find(|g| g.name == name)
    }

    /// Get pin by name
    pub fn get_pin(&self, name: &str) -> Option<&Group> {
        self.iter_subgroups_of_type("pin").find(|g| g.name == name)
    }

    /// Get pin by name mutably
    pub fn get_pin_mut(&mut self, name: &str) -> Option<&mut Group> {
        self.iter_subgroups_of_type_mut("pin")
            .find(|g| g.name == name)
    }

    /// Iterate over the cells
    pub fn iter_cells(&self) -> impl Iterator<Item = &Group> {
        self.iter_subgroups_of_type("cell")
    }

    /// Iterate over the cells mutably
    pub fn iter_cells_mut(&mut self) -> impl Iterator<Item = &mut Group> {
        self.iter_subgroups_of_type_mut("cell")
    }

    /// Iterate over the pins
    pub fn iter_pins(&self) -> impl Iterator<Item = &Group> {
        self.iter_subgroups_of_type("pin")
    }

    /// Iterate over the pins mutably
    pub fn iter_pins_mut(&mut self) -> impl Iterator<Item = &mut Group> {
        self.iter_subgroups_of_type_mut("pin")
    }
}
