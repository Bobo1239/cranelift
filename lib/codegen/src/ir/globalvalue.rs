//! Global values.

use ir::immediates::{Imm64, Offset32};
use ir::{ExternalName, GlobalValue, Type};
use isa::TargetIsa;
use std::fmt;

/// Information about a global value declaration.
#[derive(Clone)]
pub enum GlobalValueData {
    /// Value is the address of the VM context struct.
    VMContext,

    /// Value is pointed to by another global value.
    ///
    /// The `base` global value is assumed to contain a pointer. This global value is computed
    /// by loading from memory at that pointer value. The memory must be accessible, and
    /// naturally aligned to hold a value of the type.
    Load {
        /// The base pointer global value.
        base: GlobalValue,

        /// Offset added to the base pointer before doing the load.
        offset: Offset32,

        /// Type of the loaded value.
        global_type: Type,
    },

    /// Value is an offset from another global value.
    IAddImm {
        /// The base pointer global value.
        base: GlobalValue,

        /// Byte offset to be added to the value.
        offset: Imm64,

        /// Type of the iadd.
        global_type: Type,
    },

    /// Value is symbolic, meaning it's a name which will be resolved to an
    /// actual value later (eg. by linking). Cranelift itself does not interpret
    /// this name; it's used by embedders to link with other data structures.
    ///
    /// For now, symbolic values always have pointer type, and represent
    /// addresses, however in the future they could be used to represent other
    /// things as well.
    Symbol {
        /// The symbolic name.
        name: ExternalName,

        /// Offset from the symbol. This can be used instead of IAddImm to represent folding an
        /// offset into a symbol.
        offset: Imm64,

        /// Will this symbol be defined nearby, such that it will always be a certain distance
        /// away, after linking? If so, references to it can avoid going through a GOT. Note that
        /// symbols meant to be preemptible cannot be colocated.
        colocated: bool,
    },
}

impl GlobalValueData {
    /// Assume that `self` is an `GlobalValueData::Symbol` and return its name.
    pub fn symbol_name(&self) -> &ExternalName {
        match *self {
            GlobalValueData::Symbol { ref name, .. } => name,
            _ => panic!("only symbols have names"),
        }
    }

    /// Return the type of this global.
    pub fn global_type(&self, isa: &TargetIsa) -> Type {
        match *self {
            GlobalValueData::VMContext { .. } | GlobalValueData::Symbol { .. } => {
                isa.pointer_type()
            }
            GlobalValueData::IAddImm { global_type, .. }
            | GlobalValueData::Load { global_type, .. } => global_type,
        }
    }
}

impl fmt::Display for GlobalValueData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            GlobalValueData::VMContext => write!(f, "vmctx"),
            GlobalValueData::Load {
                base,
                offset,
                global_type,
            } => write!(f, "load.{} notrap aligned {}{}", global_type, base, offset),
            GlobalValueData::IAddImm {
                global_type,
                base,
                offset,
            } => write!(f, "iadd_imm.{} {}, {}", global_type, base, offset),
            GlobalValueData::Symbol {
                ref name,
                offset,
                colocated,
            } => {
                if colocated {
                    write!(f, "colocated ")?;
                }
                write!(f, "symbol {}", name)?;
                let offset_val: i64 = offset.into();
                if offset_val > 0 {
                    write!(f, "+")?;
                }
                if offset_val != 0 {
                    write!(f, "{}", offset)?;
                }
                Ok(())
            }
        }
    }
}
