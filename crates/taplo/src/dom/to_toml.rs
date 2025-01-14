use super::{
    node::{ArrayKind, DomNode, IntegerRepr, IntegerValue, TableKind},
    Keys, Node,
};
use std::fmt::{Formatter, Write};

impl Node {
    pub fn to_toml(&self, inline: bool) -> String {
        let mut s = String::new();
        self.to_toml_fmt(&mut s, inline).unwrap();
        s
    }

    pub fn to_toml_fmt(&self, f: &mut impl Write, inline: bool) -> core::fmt::Result {
        self.to_toml_impl(f, Keys::empty(), inline, false)
    }

    fn to_toml_impl(
        &self,
        f: &mut impl Write,
        parent_keys: Keys,
        inline: bool,
        no_header: bool,
    ) -> core::fmt::Result {
        if let Node::Bool(_) | Node::Str(_) | Node::Integer(_) | Node::Float(_) | Node::Date(_) =
            self
        {
            if !parent_keys.is_empty() {
                f.write_str(parent_keys.dotted())?;
                f.write_str(" = ")?;
            }

            // Use the original representation of primitives if available.
            if let Some(syntax) = self.syntax() {
                return write!(f, "{}", syntax);
            }
        }

        match self {
            Node::Table(table) => {
                if table.inner.kind == TableKind::Inline || inline {
                    if !parent_keys.is_empty() {
                        f.write_str(parent_keys.dotted())?;
                        f.write_str(" = ")?;
                    }

                    f.write_str("{ ")?;

                    let entries = table.entries().read();

                    let mut first = true;
                    for (key, node) in entries.iter() {
                        if !first {
                            f.write_str(", ")?;
                        }
                        node.to_toml_impl(f, key.clone().into(), true, false)?;
                        first = false;
                    }

                    f.write_str(" }")?;
                } else {
                    if !parent_keys.is_empty() && !no_header {
                        f.write_str("[")?;
                        f.write_str(parent_keys.dotted())?;
                        f.write_str("]\n")?;
                    }

                    let entries = table.entries().read();

                    // We make two runs to put tables and array of tables last.
                    // No tables:
                    for (key, node) in entries.iter().filter(|(_, n)| {
                        !n.is_table()
                            && !n
                                .as_array()
                                .map(|n| n.inner.kind == ArrayKind::Tables)
                                .unwrap_or(false)
                    }) {
                        node.to_toml_impl(f, key.clone().into(), false, false)?;
                        f.write_char('\n')?;
                    }

                    // Tables only:
                    for (key, node) in entries.iter().filter(|(_, n)| {
                        n.is_table()
                            || n.as_array()
                                .map(|n| n.inner.kind == ArrayKind::Tables)
                                .unwrap_or(false)
                    }) {
                        node.to_toml_impl(f, parent_keys.join(key.clone()), false, false)?;
                    }
                }
            }
            Node::Array(array) => {
                if array.inner.kind == ArrayKind::Inline || inline {
                    if !parent_keys.is_empty() {
                        f.write_str(parent_keys.dotted())?;
                        f.write_str(" = ")?;
                    }

                    f.write_str("[ ")?;

                    let items = array.items().read();

                    let mut first = true;
                    for node in items.iter() {
                        if !first {
                            f.write_str(", ")?;
                        }
                        node.to_toml_impl(f, Keys::empty(), true, false)?;
                        first = false;
                    }

                    f.write_str(" ]")?;
                } else {
                    let items = array.items().read();

                    for node in items.iter() {
                        f.write_str("[[")?;
                        f.write_str(parent_keys.dotted())?;
                        f.write_str("]]\n")?;
                        node.to_toml_impl(f, parent_keys.clone(), false, true)?;
                    }
                }
            }
            Node::Bool(b) => write!(f, "{}", b.value())?,
            Node::Str(s) => {
                write!(f, "{}", serde_json::to_string(s.value()).unwrap())?;
            }
            Node::Integer(i) => match i.inner.repr {
                IntegerRepr::Dec => match i.value() {
                    IntegerValue::Negative(i) => write!(f, "{i}")?,
                    IntegerValue::Positive(i) => write!(f, "{i}")?,
                },
                IntegerRepr::Bin => write!(f, "{:#b}", i.value().as_positive().unwrap())?,
                IntegerRepr::Oct => write!(f, "{:#o}", i.value().as_positive().unwrap())?,
                IntegerRepr::Hex => write!(f, "{:#X}", i.value().as_positive().unwrap())?,
            },
            Node::Float(float) => {
                write!(f, "{}", float.value())?;
            }
            Node::Date(d) => write!(f, "{}", d.value())?,
            Node::Invalid(_) => {}
        }

        Ok(())
    }
}

impl core::fmt::Display for Node {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.to_toml_impl(f, Keys::empty(), false, false)
    }
}
