//! Defines a slightly enhanced data structure than the base AST
//!
//! Specifically:
//! * attributes are separated into `simple_attributes` and `complex_attributes`
//!   struct fields as [HashMap](std::collections::HashMap)s.
//! * `cell` and `pin` groups are brought out into [HashMap](std::collections::HashMap)s so they're
//!   easier to work with

use std::{
    collections::HashMap,
    fmt,
    ops::{Deref, DerefMut},
};

use crate::ast::{GroupItem, LibertyAst, Value};

/// Top-level data structure of a Liberty file
#[derive(Debug, PartialEq, Clone)]
pub struct Liberty(pub Vec<Library>);

impl Liberty {
    pub fn to_ast(self) -> LibertyAst {
        LibertyAst(
            self.0
                .into_iter()
                .map(|g| g.into_group().into_group_item())
                .collect(),
        )
    }
    pub fn from_ast(ast: LibertyAst) -> Self {
        Liberty(
            ast.0
                .into_iter()
                .map(|g| Library::from_group(Group::from_group_item(g)))
                .collect(),
        )
    }
}

impl Deref for Liberty {
    type Target = [Library];

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
    type Item = Library;
    type IntoIter = ::std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

/// Top-level `library` group
///
/// Every liberty file defines a `library` at the top-most level. Libraries contain
/// attributes, groups, and [Cell](Cell)s.
#[derive(Debug, PartialEq, Clone)]
pub struct Library {
    pub name: String,
    pub simple_attributes: HashMap<String, Value>,
    pub complex_attributes: HashMap<String, Vec<Value>>,
    pub groups: Vec<Group>,
    pub cells: HashMap<String, Cell>,
}

impl Library {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            simple_attributes: HashMap::new(),
            complex_attributes: HashMap::new(),
            groups: vec![],
            cells: HashMap::new(),
        }
    }
}

/// General group struct
///
/// Groups contain simple attributes, complex attributes, and other groups
#[derive(Debug, PartialEq, Clone)]
pub struct Group {
    pub type_: String,
    pub name: String,
    pub simple_attributes: HashMap<String, Value>,
    pub complex_attributes: HashMap<String, Vec<Value>>,
    pub groups: Vec<Group>,
}

impl Group {
    /// Create a group with empty attributes and sub-groups
    pub fn new(type_: &str, name: &str) -> Self {
        Self {
            type_: type_.to_string(),
            name: name.to_string(),
            simple_attributes: HashMap::new(),
            complex_attributes: HashMap::new(),
            groups: vec![],
        }
    }

    /// Convert an AST [GroupItem::Group] variant into a [Group] struct
    pub fn from_group_item(group_item: GroupItem) -> Self {
        let (type_, name, items) = group_item.group();
        let mut simple_attributes: HashMap<String, Value> = HashMap::new();
        let mut complex_attributes: HashMap<String, Vec<Value>> = HashMap::new();
        let mut groups: Vec<Self> = vec![];
        for item in items {
            match item {
                GroupItem::SimpleAttr(name, value) => {
                    simple_attributes.insert(name, value);
                }
                GroupItem::ComplexAttr(name, value) => {
                    complex_attributes.insert(name, value);
                }
                GroupItem::Group(type_, name, items) => {
                    groups.push(Group::from_group_item(GroupItem::Group(type_, name, items)));
                }
                _ => {}
            }
        }
        Self {
            name,
            type_,
            simple_attributes,
            complex_attributes,
            groups,
        }
    }

    /// Convert a [Liberty] struct into a [GroupItem::Group] variant
    pub fn into_group_item(self) -> GroupItem {
        let mut items: Vec<GroupItem> = Vec::with_capacity(
            self.simple_attributes.len() + self.complex_attributes.len() + self.groups.len(),
        );
        items.extend(
            self.simple_attributes
                .into_iter()
                .map(|(name, value)| GroupItem::SimpleAttr(name, value)),
        );
        items.extend(
            self.complex_attributes
                .into_iter()
                .map(|(name, value)| GroupItem::ComplexAttr(name, value)),
        );
        items.extend(self.groups.into_iter().map(|g| g.into_group_item()));
        GroupItem::Group(self.type_, self.name, items)
    }
}

/// `cell` group of a [Library](Library)
#[derive(Debug, PartialEq, Clone)]
pub struct Cell {
    pub name: String,
    pub simple_attributes: HashMap<String, Value>,
    pub complex_attributes: HashMap<String, Vec<Value>>,
    pub groups: Vec<Group>,
    pub pins: HashMap<String, Pin>,
}

impl Cell {
    /// Create a cell with empty attributes and sub-groups
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            simple_attributes: HashMap::new(),
            complex_attributes: HashMap::new(),
            groups: vec![],
            pins: HashMap::new(),
        }
    }
}

/// `pin` group of a [Cell](Cell)
#[derive(Debug, PartialEq, Clone)]
pub struct Pin {
    pub name: String,
    pub simple_attributes: HashMap<String, Value>,
    pub complex_attributes: HashMap<String, Vec<Value>>,
    pub groups: Vec<Group>,
}

impl Pin {
    /// Create a pin with empty attributes and sub-groups
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            simple_attributes: HashMap::new(),
            complex_attributes: HashMap::new(),
            groups: vec![],
        }
    }
}

/// Convert a general Group into a more specific type
///
/// Implemented by Pin, Cell, or Library
pub trait FromGroup {
    type Item;

    fn from_group(group: Group) -> Self::Item;
}

pub trait ToGroup {
    type Item;

    fn into_group(self) -> Group;
}

impl FromGroup for Library {
    type Item = Library;
    fn from_group(group: Group) -> Self::Item {
        let (cells, groups) = group.groups.into_iter().partition(|g| g.type_ == "cell");
        Self {
            name: group.name,
            simple_attributes: group.simple_attributes,
            complex_attributes: group.complex_attributes,
            groups,
            cells: cells.into_iter().fold(HashMap::new(), |mut acc, cell| {
                acc.insert(cell.name.clone(), Cell::from_group(cell));
                acc
            }),
        }
    }
}

impl ToGroup for Library {
    type Item = Library;
    fn into_group(self) -> Group {
        let mut groups: Vec<Group> = Vec::with_capacity(self.groups.len() + self.cells.len());
        groups.extend(self.groups);
        groups.extend(self.cells.into_iter().map(|(_, cell)| cell.into_group()));
        Group {
            name: self.name,
            type_: String::from("library"),
            simple_attributes: self.simple_attributes,
            complex_attributes: self.complex_attributes,
            groups,
        }
    }
}

impl FromGroup for Cell {
    type Item = Cell;
    fn from_group(group: Group) -> Self::Item {
        let (pins, groups) = group.groups.into_iter().partition(|g| g.type_ == "pin");
        Self {
            name: group.name,
            simple_attributes: group.simple_attributes,
            complex_attributes: group.complex_attributes,
            groups,
            pins: pins.into_iter().fold(HashMap::new(), |mut acc, pin| {
                acc.insert(pin.name.clone(), Pin::from_group(pin));
                acc
            }),
        }
    }
}

impl ToGroup for Cell {
    type Item = Cell;

    fn into_group(self) -> Group {
        let mut groups: Vec<Group> = Vec::with_capacity(self.groups.len() + self.pins.len());
        groups.extend(self.pins.into_iter().map(|(_, pin)| pin.into_group()));
        groups.extend(self.groups);
        Group {
            name: self.name,
            type_: String::from("cell"),
            simple_attributes: self.simple_attributes,
            complex_attributes: self.complex_attributes,
            groups,
        }
    }
}

impl FromGroup for Pin {
    type Item = Pin;
    fn from_group(group: Group) -> Self::Item {
        Self {
            name: group.name,
            simple_attributes: group.simple_attributes,
            complex_attributes: group.complex_attributes,
            groups: group.groups,
        }
    }
}

impl ToGroup for Pin {
    type Item = Pin;
    fn into_group(self) -> Group {
        Group {
            name: self.name,
            type_: String::from("pin"),
            simple_attributes: self.simple_attributes,
            complex_attributes: self.complex_attributes,
            groups: self.groups,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_iter() {
        let lib = Liberty(vec![Library::new("mylib")]);
        let mut iter = lib.into_iter();
        assert_eq!(iter.next(), Some(Library::new("mylib")));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_pin_into_group() {
        let mut pin = Pin::new("my_pin");
        pin.groups.push(Group::new("gtype", "gname"));
        let group = pin.into_group();
        assert_eq!(group.type_, "pin");
        assert_eq!(group.name, "my_pin");
        assert_eq!(group.groups.len(), 1);
    }

    #[test]
    fn test_pin_from_group() {
        let mut group = Group::new("pin", "a");
        group.groups.push(Group::new("gtype", "gname"));
        let pin = Pin::from_group(group);
        assert_eq!(pin.name, "a");
        assert_eq!(pin.groups.len(), 1);
    }

    #[test]
    fn test_cell_into_group() {
        let mut cell = Cell::new("my_cell");
        cell.groups.push(Group::new("gtype", "gname"));
        cell.pins.insert("a".to_string(), Pin::new("a"));
        cell.pins.insert("b".to_string(), Pin::new("b"));
        let group = cell.into_group();
        assert_eq!(group.type_, "cell");
        assert_eq!(group.name, "my_cell");
        assert_eq!(group.groups.len(), 3);
    }

    #[test]
    fn test_cell_from_group() {
        let mut group = Group::new("cell", "AND2");
        group.groups.push(Group::new("gtype", "gname"));
        group.groups.push(Group::new("pin", "a"));
        group.groups.push(Group::new("pin", "b"));
        let cell = Cell::from_group(group);
        assert_eq!(cell.name, "AND2");
        assert_eq!(cell.groups.len(), 1);
        assert_eq!(cell.pins.len(), 2);
    }

    #[test]
    fn test_library_into_group() {
        let mut lib = Library::new("my_lib");
        lib.groups.push(Group::new("gtype", "gname"));
        lib.cells.insert("AND2".to_string(), Cell::new("AND2"));
        lib.cells.insert("NAND2".to_string(), Cell::new("NAND2"));
        let group = lib.into_group();
        assert_eq!(group.type_, "library");
        assert_eq!(group.name, "my_lib");
        assert_eq!(group.groups.len(), 3);
    }

    #[test]
    fn test_lib_from_group() {
        let mut group = Group::new("library", "mylib");
        group.groups.push(Group::new("gtype", "gname"));
        let mut cell = Group::new("cell", "AND2");
        cell.groups.push(Group::new("pin", "a"));
        cell.groups.push(Group::new("pin", "b"));
        group.groups.push(cell);
        let lib = Library::from_group(group);
        assert_eq!(lib.name, "mylib");
        assert_eq!(lib.groups.len(), 1);
        assert_eq!(lib.cells.len(), 1);
        let converted_cell = lib.cells.get("AND2").unwrap();
        assert_eq!(converted_cell.name, "AND2");
        assert_eq!(converted_cell.groups.len(), 0);
        assert_eq!(converted_cell.pins.len(), 2);
    }
}
