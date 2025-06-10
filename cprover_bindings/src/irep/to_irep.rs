// Copyright Kani Contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT
//! Converts a typed goto-program into the `Irep` serilization format of CBMC
// TODO: consider making a macro to replace `linear_map![])` for initilizing btrees.
use super::super::MachineModel;
use super::super::goto_program;
use super::{Irep, IrepId};
use crate::InternedString;
use crate::linear_map;
use goto_program::{
    BinaryOperator, CIntType, DatatypeComponent, Expr, ExprValue, Lambda, Location, Parameter,
    SelfOperator, Stmt, StmtBody, SwitchCase, SymbolValues, Type, UnaryOperator,
};

pub trait ToIrep {
    fn to_irep<'i>(&'i self, mm: &MachineModel, arena: &'i bumpalo::Bump) -> Irep<'i>;
}

/// Utility functions
fn arguments_irep<'a: 'i, 'i>(
    arguments: impl Iterator<Item = &'a Expr>,
    mm: &MachineModel,
    arena: &'i bumpalo::Bump,
) -> Irep<'i> {
    let mut sub = Vec::new_in(arena);

    for a in arguments {
        sub.push(a.to_irep(mm, arena));
    }

    Irep { id: IrepId::Arguments, sub, named_sub: linear_map![] }
}
fn code_irep<'i>(kind: IrepId, ops: Vec<Irep<'i>, &'i bumpalo::Bump>) -> Irep<'i> {
    Irep {
        id: IrepId::Code,
        named_sub: linear_map![(IrepId::Statement, Irep::just_id(kind, ops.allocator()))],
        sub: ops,
    }
}
fn side_effect_irep<'i>(kind: IrepId, ops: Vec<Irep<'i>, &'i bumpalo::Bump>) -> Irep<'i> {
    Irep {
        id: IrepId::SideEffect,
        named_sub: linear_map![(IrepId::Statement, Irep::just_id(kind, ops.allocator()))],
        sub: ops,
    }
}
fn switch_default_irep<'i>(
    body: &'i Stmt,
    mm: &MachineModel,
    arena: &'i bumpalo::Bump,
) -> Irep<'i> {
    let mut ops = Vec::new_in(arena);
    ops.push(Irep::nil(arena));
    ops.push(body.to_irep(mm, arena));

    code_irep(IrepId::SwitchCase, ops)
        .with_named_sub(IrepId::Default, Irep::one(arena))
        .with_location(body.location(), mm, arena)
}

/// ID Converters
pub trait ToIrepId {
    fn to_irep_id(&self) -> IrepId;
}

impl ToIrepId for BinaryOperator {
    fn to_irep_id(&self) -> IrepId {
        match self {
            BinaryOperator::And => IrepId::And,
            BinaryOperator::Ashr => IrepId::Ashr,
            BinaryOperator::Bitand => IrepId::Bitand,
            BinaryOperator::Bitnand => IrepId::Bitnand,
            BinaryOperator::Bitor => IrepId::Bitor,
            BinaryOperator::Bitxor => IrepId::Bitxor,
            BinaryOperator::Div => IrepId::Div,
            BinaryOperator::Equal => IrepId::Equal,
            BinaryOperator::FloatbvRoundToIntegral => IrepId::FloatbvRoundToIntegral,
            BinaryOperator::Ge => IrepId::Ge,
            BinaryOperator::Gt => IrepId::Gt,
            BinaryOperator::IeeeFloatEqual => IrepId::IeeeFloatEqual,
            BinaryOperator::IeeeFloatNotequal => IrepId::IeeeFloatNotequal,
            BinaryOperator::Implies => IrepId::Implies,
            BinaryOperator::Le => IrepId::Le,
            BinaryOperator::Lshr => IrepId::Lshr,
            BinaryOperator::Lt => IrepId::Lt,
            BinaryOperator::Minus => IrepId::Minus,
            BinaryOperator::Mod => IrepId::Mod,
            BinaryOperator::Mult => IrepId::Mult,
            BinaryOperator::Notequal => IrepId::Notequal,
            BinaryOperator::Or => IrepId::Or,
            BinaryOperator::OverflowMinus => IrepId::OverflowMinus,
            BinaryOperator::OverflowMult => IrepId::OverflowMult,
            BinaryOperator::OverflowPlus => IrepId::OverflowPlus,
            BinaryOperator::OverflowResultMinus => IrepId::OverflowResultMinus,
            BinaryOperator::OverflowResultMult => IrepId::OverflowResultMult,
            BinaryOperator::OverflowResultPlus => IrepId::OverflowResultPlus,
            BinaryOperator::Plus => IrepId::Plus,
            BinaryOperator::ROk => IrepId::ROk,
            BinaryOperator::Rol => IrepId::Rol,
            BinaryOperator::Ror => IrepId::Ror,
            BinaryOperator::Shl => IrepId::Shl,
            BinaryOperator::Xor => IrepId::Xor,
            BinaryOperator::VectorEqual => IrepId::VectorEqual,
            BinaryOperator::VectorNotequal => IrepId::VectorNotequal,
            BinaryOperator::VectorGe => IrepId::VectorGe,
            BinaryOperator::VectorLe => IrepId::VectorLe,
            BinaryOperator::VectorGt => IrepId::VectorGt,
            BinaryOperator::VectorLt => IrepId::VectorLt,
        }
    }
}

impl ToIrepId for SelfOperator {
    fn to_irep_id(&self) -> IrepId {
        match self {
            SelfOperator::Postdecrement => IrepId::Postdecrement,
            SelfOperator::Postincrement => IrepId::Postincrement,
            SelfOperator::Predecrement => IrepId::Predecrement,
            SelfOperator::Preincrement => IrepId::Preincrement,
        }
    }
}

impl ToIrepId for UnaryOperator {
    fn to_irep_id(&self) -> IrepId {
        match self {
            UnaryOperator::Bitnot => IrepId::Bitnot,
            UnaryOperator::BitReverse => IrepId::BitReverse,
            UnaryOperator::Bswap => IrepId::Bswap,
            UnaryOperator::CountLeadingZeros { .. } => IrepId::CountLeadingZeros,
            UnaryOperator::CountTrailingZeros { .. } => IrepId::CountTrailingZeros,
            UnaryOperator::IsDynamicObject => IrepId::IsDynamicObject,
            UnaryOperator::IsFinite => IrepId::IsFinite,
            UnaryOperator::Not => IrepId::Not,
            UnaryOperator::ObjectSize => IrepId::ObjectSize,
            UnaryOperator::PointerObject => IrepId::PointerObject,
            UnaryOperator::PointerOffset => IrepId::PointerOffset,
            UnaryOperator::Popcount => IrepId::Popcount,
            UnaryOperator::UnaryMinus => IrepId::UnaryMinus,
        }
    }
}

/// The main converters
impl ToIrep for DatatypeComponent {
    fn to_irep<'i>(&'i self, mm: &MachineModel, arena: &'i bumpalo::Bump) -> Irep<'i> {
        match self {
            DatatypeComponent::Field { name, typ } => Irep::just_named_sub(
                linear_map![
                    (IrepId::Name, Irep::just_string_id(name.to_string(), arena)),
                    (IrepId::CPrettyName, Irep::just_string_id(name.to_string(), arena)),
                    (IrepId::Type, typ.to_irep(mm, arena)),
                ],
                arena,
            ),
            DatatypeComponent::Padding { name, bits } => Irep::just_named_sub(
                linear_map![
                    (IrepId::CIsPadding, Irep::one(arena)),
                    (IrepId::Name, Irep::just_string_id(name.to_string(), arena)),
                    (IrepId::Type, Type::unsigned_int(*bits).to_irep(mm, arena)),
                ],
                arena,
            ),
        }
    }
}

impl ToIrep for Expr {
    fn to_irep<'i>(&'i self, mm: &MachineModel, arena: &'i bumpalo::Bump) -> Irep<'i> {
        if let ExprValue::IntConstant(i) = self.value() {
            let typ_width = self.typ().native_width(mm);
            let irep_value = if let Some(width) = typ_width {
                Irep::just_bitpattern_id(i.clone(), width, self.typ().is_signed(mm), arena)
            } else {
                Irep::just_int_id(i.clone(), arena)
            };
            Irep {
                id: IrepId::Constant,
                sub: Vec::new_in(arena),
                named_sub: linear_map![(IrepId::Value, irep_value,)],
            }
            .with_location(self.location(), mm, arena)
            .with_type(self.typ(), mm, arena)
        } else {
            self.value().to_irep(mm, arena).with_location(self.location(), mm, arena).with_type(
                self.typ(),
                mm,
                arena,
            )
        }
        .with_named_sub_option(
            IrepId::CCSizeofType,
            self.size_of_annotation().map(|ty| ty.to_irep(mm, arena)),
        )
    }
}

impl<'i> Irep<'i> {
    pub fn symbol(identifier: InternedString, arena: &'i bumpalo::Bump) -> Self {
        Irep {
            id: IrepId::Symbol,
            sub: Vec::new_in(arena),
            named_sub: linear_map![(IrepId::Identifier, Irep::just_string_id(identifier, arena))],
        }
    }
}

impl ToIrep for ExprValue {
    fn to_irep<'i>(&'i self, mm: &MachineModel, arena: &'i bumpalo::Bump) -> Irep {
        let mut sub = Vec::new_in(arena);
        match self {
            ExprValue::AddressOf(e) => {
                sub.push(e.to_irep(mm, arena));
                Irep { id: IrepId::AddressOf, sub, named_sub: linear_map![] }
            }
            ExprValue::Array { elems } => {
                for e in elems {
                    sub.push(e.to_irep(mm, arena));
                }

                Irep { id: IrepId::Array, sub, named_sub: linear_map![] }
            }
            ExprValue::ArrayOf { elem } => {
                sub.push(elem.to_irep(mm, arena));
                Irep { id: IrepId::ArrayOf, sub, named_sub: linear_map![] }
            }
            ExprValue::Assign { left, right } => {
                sub.extend_from_slice(&[left.to_irep(mm, arena), right.to_irep(mm, arena)]);
                side_effect_irep(IrepId::Assign, sub)
            }
            ExprValue::BinOp { op, lhs, rhs } => {
                sub.extend_from_slice(&[lhs.to_irep(mm, arena), rhs.to_irep(mm, arena)]);
                Irep { id: op.to_irep_id(), sub, named_sub: linear_map![] }
            }
            ExprValue::BoolConstant(c) => Irep {
                id: IrepId::Constant,
                sub: Vec::new_in(arena),
                named_sub: linear_map![(
                    IrepId::Value,
                    if *c {
                        Irep::just_id(IrepId::True, arena)
                    } else {
                        Irep::just_id(IrepId::False, arena)
                    },
                )],
            },
            ExprValue::ByteExtract { e, offset } => {
                sub.extend_from_slice(&[
                    e.to_irep(mm, arena),
                    Expr::int_constant(*offset, Type::ssize_t()).to_irep(mm, arena),
                ]);
                Irep {
                    id: if mm.is_big_endian {
                        IrepId::ByteExtractBigEndian
                    } else {
                        IrepId::ByteExtractLittleEndian
                    },
                    sub,
                    named_sub: linear_map![],
                }
            }
            ExprValue::CBoolConstant(i) => Irep {
                id: IrepId::Constant,
                sub: Vec::new_in(arena),
                named_sub: linear_map![(
                    IrepId::Value,
                    Irep::just_bitpattern_id(if *i { 1u8 } else { 0 }, mm.bool_width, false, arena)
                )],
            },
            ExprValue::Dereference(e) => {
                sub.push(e.to_irep(mm, arena));
                Irep { id: IrepId::Dereference, sub, named_sub: linear_map![] }
            }
            //TODO, determine if there is an endineness problem here
            ExprValue::DoubleConstant(i) => {
                let c: u64 = i.to_bits();
                Irep {
                    id: IrepId::Constant,
                    sub: Vec::new_in(arena),
                    named_sub: linear_map![(
                        IrepId::Value,
                        Irep::just_bitpattern_id(c, mm.double_width, false, arena)
                    )],
                }
            }
            ExprValue::EmptyUnion => Irep::just_id(IrepId::EmptyUnion, arena),
            ExprValue::FloatConstant(i) => {
                let c: u32 = i.to_bits();
                Irep {
                    id: IrepId::Constant,
                    sub: Vec::new_in(arena),
                    named_sub: linear_map![(
                        IrepId::Value,
                        Irep::just_bitpattern_id(c, mm.float_width, false, arena)
                    )],
                }
            }
            ExprValue::Float16Constant(i) => {
                let c: u16 = i.to_bits();
                Irep {
                    id: IrepId::Constant,
                    sub: Vec::new_in(arena),
                    named_sub: linear_map![(
                        IrepId::Value,
                        Irep::just_bitpattern_id(c, 16, false, arena)
                    )],
                }
            }
            ExprValue::Float128Constant(i) => {
                let c: u128 = i.to_bits();
                Irep {
                    id: IrepId::Constant,
                    sub: Vec::new_in(arena),
                    named_sub: linear_map![(
                        IrepId::Value,
                        Irep::just_bitpattern_id(c, 128, false, arena)
                    )],
                }
            }
            ExprValue::FunctionCall { function, arguments } => {
                sub.extend_from_slice(&[
                    function.to_irep(mm, arena),
                    arguments_irep(arguments.iter(), mm, arena),
                ]);
                side_effect_irep(IrepId::FunctionCall, sub)
            }
            ExprValue::If { c, t, e } => {
                sub.extend([c.to_irep(mm, arena), t.to_irep(mm, arena), e.to_irep(mm, arena)]);
                Irep { id: IrepId::If, sub, named_sub: linear_map![] }
            }
            ExprValue::Index { array, index } => {
                sub.extend([array.to_irep(mm, arena), index.to_irep(mm, arena)]);
                Irep { id: IrepId::Index, sub, named_sub: linear_map![] }
            }
            ExprValue::IntConstant(_) => {
                unreachable!("Should have been processed in previous step")
            }
            ExprValue::Member { lhs, field } => {
                sub.extend([lhs.to_irep(mm, arena)]);
                Irep {
                    id: IrepId::Member,
                    sub,
                    named_sub: linear_map![
                        (IrepId::CLvalue, Irep::one(arena)),
                        (IrepId::ComponentName, Irep::just_string_id(field.to_string(), arena)),
                    ],
                }
            }
            ExprValue::Nondet => side_effect_irep(IrepId::Nondet, Vec::new_in(arena)),
            ExprValue::PointerConstant(0) => Irep {
                id: IrepId::Constant,
                sub: Vec::new_in(arena),
                named_sub: linear_map![(IrepId::Value, Irep::just_id(IrepId::NULL, arena))],
            },
            ExprValue::PointerConstant(i) => Irep {
                id: IrepId::Constant,
                sub: Vec::new_in(arena),
                named_sub: linear_map![(
                    IrepId::Value,
                    Irep::just_bitpattern_id(*i, mm.pointer_width, false, arena)
                )],
            },
            ExprValue::ReadOk { ptr, size } => {
                sub.extend([ptr.to_irep(mm, arena), size.to_irep(mm, arena)]);
                Irep { id: IrepId::ROk, sub, named_sub: linear_map![] }
            }
            ExprValue::SelfOp { op, e } => {
                sub.extend([e.to_irep(mm, arena)]);
                side_effect_irep(op.to_irep_id(), sub)
            }
            ExprValue::StatementExpression { statements: ops, location: loc } => {
                sub.extend([Stmt::block(ops.to_vec(), *loc).to_irep(mm, arena)]);
                side_effect_irep(IrepId::StatementExpression, sub)
            }
            ExprValue::StringConstant { s } => Irep {
                id: IrepId::StringConstant,
                sub: Vec::new_in(arena),
                named_sub: linear_map![
                    (IrepId::Value, Irep::just_string_id(s.to_string(), arena),)
                ],
            },
            ExprValue::Struct { values } => {
                sub.extend(values.iter().map(|x| x.to_irep(mm, arena)));
                Irep { id: IrepId::Struct, sub, named_sub: linear_map![] }
            }
            ExprValue::Symbol { identifier } => Irep::symbol(*identifier, arena),
            ExprValue::Typecast(e) => {
                sub.extend([e.to_irep(mm, arena)]);
                Irep { id: IrepId::Typecast, sub, named_sub: linear_map![] }
            }
            ExprValue::Union { value, field } => {
                sub.extend([value.to_irep(mm, arena)]);
                Irep {
                    id: IrepId::Union,
                    sub,
                    named_sub: linear_map![(
                        IrepId::ComponentName,
                        Irep::just_string_id(field.to_string(), arena),
                    )],
                }
            }
            ExprValue::UnOp { op: UnaryOperator::Bswap, e } => {
                sub.extend([e.to_irep(mm, arena)]);
                Irep {
                    id: IrepId::Bswap,
                    sub,
                    named_sub: linear_map![(IrepId::BitsPerByte, Irep::just_int_id(8u8, arena))],
                }
            }
            ExprValue::UnOp { op: UnaryOperator::BitReverse, e } => {
                sub.extend([e.to_irep(mm, arena)]);
                Irep { id: IrepId::BitReverse, sub, named_sub: linear_map![] }
            }
            ExprValue::UnOp { op: UnaryOperator::CountLeadingZeros { allow_zero }, e } => {
                sub.extend([e.to_irep(mm, arena)]);
                Irep {
                    id: IrepId::CountLeadingZeros,
                    sub,
                    named_sub: linear_map![(
                        IrepId::CBoundsCheck,
                        if *allow_zero { Irep::zero(arena) } else { Irep::one(arena) }
                    )],
                }
            }
            ExprValue::UnOp { op: UnaryOperator::CountTrailingZeros { allow_zero }, e } => {
                sub.extend([e.to_irep(mm, arena)]);
                Irep {
                    id: IrepId::CountTrailingZeros,
                    sub,
                    named_sub: linear_map![(
                        IrepId::CBoundsCheck,
                        if *allow_zero { Irep::zero(arena) } else { Irep::one(arena) }
                    )],
                }
            }
            ExprValue::UnOp { op, e } => {
                sub.extend([e.to_irep(mm, arena)]);
                Irep { id: op.to_irep_id(), sub, named_sub: linear_map![] }
            }
            ExprValue::Vector { elems } => {
                sub.extend(elems.iter().map(|x| x.to_irep(mm, arena)));
                Irep { id: IrepId::Vector, sub, named_sub: linear_map![] }
            }
            ExprValue::Forall { variable, domain } => {
                let mut new_sub = Vec::new_in(arena);
                new_sub.extend([variable.to_irep(mm, arena)]);
                sub.extend([
                    Irep { id: IrepId::Tuple, sub: new_sub, named_sub: linear_map![] },
                    domain.to_irep(mm, arena),
                ]);
                Irep { id: IrepId::Forall, sub, named_sub: linear_map![] }
            }
            ExprValue::Exists { variable, domain } => {
                let mut new_sub = Vec::new_in(arena);
                new_sub.extend([variable.to_irep(mm, arena)]);
                sub.extend([
                    Irep { id: IrepId::Tuple, sub: new_sub, named_sub: linear_map![] },
                    domain.to_irep(mm, arena),
                ]);
                Irep { id: IrepId::Exists, sub, named_sub: linear_map![] }
            }
        }
    }
}

impl ToIrep for Location {
    fn to_irep<'i>(&'i self, _mm: &MachineModel, arena: &'i bumpalo::Bump) -> Irep<'i> {
        match self {
            Location::None => Irep::nil(arena),
            Location::BuiltinFunction { line, function_name } => Irep::just_named_sub(
                linear_map![
                    (
                        IrepId::File,
                        Irep::just_string_id(format!("<builtin-library-{function_name}>"), arena),
                    ),
                    (IrepId::Function, Irep::just_string_id(function_name.to_string(), arena)),
                ],
                arena,
            )
            .with_named_sub_option(IrepId::Line, line.map(|w| Irep::just_int_id(w, arena))),
            Location::Loc {
                file,
                function,
                start_line,
                start_col,
                end_line: _,
                end_col: _,
                pragmas,
            } => Irep::just_named_sub(
                linear_map![
                    (IrepId::File, Irep::just_string_id(file.to_string(), arena)),
                    (IrepId::Line, Irep::just_int_id(*start_line, arena)),
                ],
                arena,
            )
            .with_named_sub_option(IrepId::Column, start_col.map(|w| Irep::just_int_id(w, arena)))
            .with_named_sub_option(
                IrepId::Function,
                function.map(|w| Irep::just_string_id(w, arena)),
            )
            .with_named_sub_option(
                IrepId::Pragma,
                Some(Irep::just_named_sub(
                    pragmas
                        .iter()
                        .map(|pragma| {
                            (
                                IrepId::from_string(*pragma),
                                Irep::just_id(IrepId::EmptyString, arena),
                            )
                        })
                        .collect(),
                    arena,
                )),
            ),
            Location::Property { file, function, line, col, property_class, comment, pragmas } => {
                Irep::just_named_sub(
                    linear_map![
                        (IrepId::File, Irep::just_string_id(file.to_string(), arena)),
                        (IrepId::Line, Irep::just_int_id(*line, arena)),
                    ],
                    arena,
                )
                .with_named_sub_option(IrepId::Column, col.map(|w| Irep::just_int_id(w, arena)))
                .with_named_sub_option(
                    IrepId::Function,
                    function.map(|w| Irep::just_string_id(w, arena)),
                )
                .with_named_sub(IrepId::Comment, Irep::just_string_id(comment.to_string(), arena))
                .with_named_sub(
                    IrepId::PropertyClass,
                    Irep::just_string_id(property_class.to_string(), arena),
                )
                .with_named_sub_option(
                    IrepId::Pragma,
                    Some(Irep::just_named_sub(
                        pragmas
                            .iter()
                            .map(|pragma| {
                                (
                                    IrepId::from_string(*pragma),
                                    Irep::just_id(IrepId::EmptyString, arena),
                                )
                            })
                            .collect(),
                        arena,
                    )),
                )
            }
            Location::PropertyUnknownLocation { property_class, comment } => Irep::just_named_sub(
                linear_map![
                    (IrepId::Comment, Irep::just_string_id(comment.to_string(), arena)),
                    (
                        IrepId::PropertyClass,
                        Irep::just_string_id(property_class.to_string(), arena)
                    )
                ],
                arena,
            ),
        }
    }
}

impl ToIrep for Parameter {
    fn to_irep<'i>(&'i self, mm: &MachineModel, arena: &'i bumpalo::Bump) -> Irep<'i> {
        Irep {
            id: IrepId::Parameter,
            sub: Vec::new_in(arena),
            named_sub: linear_map![(IrepId::Type, self.typ().to_irep(mm, arena))],
        }
        .with_named_sub_option(
            IrepId::CIdentifier,
            self.identifier().map(|w| Irep::just_string_id(w, arena)),
        )
        .with_named_sub_option(
            IrepId::CBaseName,
            self.base_name().map(|w| Irep::just_string_id(w, arena)),
        )
    }
}

impl ToIrep for Stmt {
    fn to_irep<'i>(&'i self, mm: &MachineModel, arena: &'i bumpalo::Bump) -> Irep<'i> {
        self.body().to_irep(mm, arena).with_location(self.location(), mm, arena)
    }
}

impl ToIrep for StmtBody {
    fn to_irep<'i>(&'i self, mm: &MachineModel, arena: &'i bumpalo::Bump) -> Irep<'i> {
        let mut ops = Vec::new_in(arena);
        match self {
            StmtBody::Assign { lhs, rhs } => {
                ops.extend([lhs.to_irep(mm, arena), rhs.to_irep(mm, arena)]);
                code_irep(IrepId::Assign, ops)
            }
            StmtBody::Assert { cond, .. } => {
                ops.extend([cond.to_irep(mm, arena)]);
                code_irep(IrepId::Assert, ops)
            }
            StmtBody::Assume { cond } => {
                ops.extend([cond.to_irep(mm, arena)]);
                code_irep(IrepId::Assume, ops)
            }
            StmtBody::AtomicBlock(stmts) => {
                let mut irep_stmts = Vec::new_in(arena);
                irep_stmts.push(code_irep(IrepId::AtomicBegin, Vec::new_in(arena)));
                for x in stmts {
                    irep_stmts.push(x.to_irep(mm, arena));
                }
                // irep_stmts.append(&mut stmts.iter().map(|x| x.to_irep(mm, arena)).collect());
                irep_stmts.push(code_irep(IrepId::AtomicEnd, Vec::new_in(arena)));
                code_irep(IrepId::Block, irep_stmts)
            }
            StmtBody::Block(stmts) => {
                let mut sub = Vec::new_in(arena);
                for s in stmts {
                    sub.push(s.to_irep(mm, arena));
                }
                code_irep(IrepId::Block, sub)
            }
            StmtBody::Break => code_irep(IrepId::Break, Vec::new_in(arena)),
            StmtBody::Continue => code_irep(IrepId::Continue, Vec::new_in(arena)),
            StmtBody::Dead(symbol) => {
                ops.extend([symbol.to_irep(mm, arena)]);
                code_irep(IrepId::Dead, ops)
            }
            StmtBody::Decl { lhs, value } => {
                if value.is_some() {
                    ops.extend([
                        lhs.to_irep(mm, arena),
                        value.as_ref().unwrap().to_irep(mm, arena),
                    ]);
                    code_irep(IrepId::Decl, ops)
                } else {
                    ops.extend([lhs.to_irep(mm, arena)]);
                    code_irep(IrepId::Decl, ops)
                }
            }
            StmtBody::Deinit(place) => {
                // CBMC doesn't yet have a notion of poison (https://github.com/diffblue/cbmc/issues/7014)
                // So we translate identically to `nondet` here, but add a comment noting we wish it were poison
                // potentially for other backends to pick up and treat specially.
                ops.extend([place.to_irep(mm, arena), place.typ().nondet().to_irep(mm, arena)]);
                code_irep(IrepId::Assign, ops).with_comment("deinit")
            }
            StmtBody::Expression(e) => {
                ops.extend([e.to_irep(mm, arena)]);
                code_irep(IrepId::Expression, ops)
            }
            StmtBody::For { init, cond, update, body } => {
                ops.extend([
                    init.to_irep(mm, arena),
                    cond.to_irep(mm, arena),
                    update.to_irep(mm, arena),
                    body.to_irep(mm, arena),
                ]);
                code_irep(IrepId::For, ops)
            }
            StmtBody::FunctionCall { lhs, function, arguments } => {
                ops.extend([
                    lhs.as_ref().map_or(Irep::nil(arena), |x| x.to_irep(mm, arena)),
                    function.to_irep(mm, arena),
                    arguments_irep(arguments.iter(), mm, arena),
                ]);
                code_irep(IrepId::FunctionCall, ops)
            }
            StmtBody::Goto { dest, loop_invariants } => {
                let stmt_goto = code_irep(IrepId::Goto, Vec::new_in(arena)).with_named_sub(
                    IrepId::Destination,
                    Irep::just_string_id(dest.to_string(), arena),
                );
                if let Some(inv) = loop_invariants {
                    stmt_goto.with_named_sub(
                        IrepId::CSpecLoopInvariant,
                        inv.clone().and(Expr::bool_true()).to_irep(mm, arena),
                    )
                } else {
                    stmt_goto
                }
            }
            StmtBody::Ifthenelse { i, t, e } => {
                ops.extend([
                    i.to_irep(mm, arena),
                    t.to_irep(mm, arena),
                    e.as_ref().map_or(Irep::nil(arena), |x| x.to_irep(mm, arena)),
                ]);
                code_irep(IrepId::Ifthenelse, ops)
            }
            StmtBody::Label { label, body } => {
                ops.extend([body.to_irep(mm, arena)]);
                code_irep(IrepId::Label, ops)
            }
            .with_named_sub(IrepId::Label, Irep::just_string_id(label.to_string(), arena)),
            StmtBody::Return(e) => {
                ops.extend([e.as_ref().map_or(Irep::nil(arena), |x| x.to_irep(mm, arena))]);
                code_irep(IrepId::Return, ops)
            }
            StmtBody::Skip => code_irep(IrepId::Skip, Vec::new_in(arena)),
            StmtBody::Switch { control, cases, default } => {
                let mut switch_arms: Vec<Irep, _> = Vec::new_in(arena);
                for c in cases {
                    switch_arms.push(c.to_irep(mm, arena));
                }

                // cases.iter().map(|x| x.to_irep(mm, arena)).collect();
                if default.is_some() {
                    switch_arms.push(switch_default_irep(default.as_ref().unwrap(), mm, arena));
                }

                ops.extend([control.to_irep(mm, arena), code_irep(IrepId::Block, switch_arms)]);
                code_irep(IrepId::Switch, ops)
            }
            StmtBody::While { cond, body } => {
                ops.extend([cond.to_irep(mm, arena), body.to_irep(mm, arena)]);
                code_irep(IrepId::While, ops)
            }
        }
    }
}

impl ToIrep for SwitchCase {
    fn to_irep<'i>(&'i self, mm: &MachineModel, arena: &'i bumpalo::Bump) -> Irep<'i> {
        let mut ops = Vec::new_in(arena);
        ops.extend([self.case().to_irep(mm, arena), self.body().to_irep(mm, arena)]);
        code_irep(IrepId::SwitchCase, ops).with_location(self.body().location(), mm, arena)
    }
}

impl ToIrep for Lambda {
    /// At the moment this function assumes that this lambda is used for a
    /// `modifies` contract. It should work for any other lambda body, but
    /// the parameter names use "modifies" in their generated names.
    fn to_irep(&self, mm: &MachineModel) -> Irep {
        let (ops_ireps, types) = self
            .arguments
            .iter()
            .enumerate()
            .map(|(index, param)| {
                let ty_rep = param.typ().to_irep(mm, arena);
                (
                    Irep::symbol(
                        param.identifier().unwrap_or_else(|| format!("_modifies_{index}").into()),
                    )
                    .with_named_sub(IrepId::Type, ty_rep.clone()),
                    ty_rep,
                )
            })
            .unzip();
        let typ = Irep {
            id: IrepId::MathematicalFunction,
            sub: vec![Irep::just_sub(types), self.body.typ().to_irep(mm, arena)],
            named_sub: Default::default(),
        };
        Irep {
            id: IrepId::Lambda,
            sub: vec![Irep::tuple(ops_ireps), self.body.to_irep(mm, arena)],
            named_sub: linear_map!((IrepId::Type, typ)),
        }
    }
}

impl goto_program::Symbol {
    pub fn to_irep(&self, mm: &MachineModel) -> super::Symbol {
        let mut typ = self.typ.to_irep(mm, arena);
        if let Some(contract) = &self.contract {
            typ = typ.with_named_sub(
                IrepId::CSpecAssigns,
                Irep::just_sub(contract.assigns.iter().map(|req| req.to_irep(mm, arena)).collect()),
            );
        }
        if self.is_static_const {
            // Add a `const` to the type.
            typ = typ.with_named_sub(IrepId::CConstant, Irep::just_id(IrepId::from_int(1)))
        }
        super::Symbol {
            typ,
            value: match &self.value {
                SymbolValues::Expr(e) => e.to_irep(mm, arena),
                SymbolValues::Stmt(s) => s.to_irep(mm, arena),
                SymbolValues::None => Irep::nil(arena),
            },
            location: self.location.to_irep(mm, arena),
            // Unique identifier, same as key in symbol table `foo::x`
            name: self.name,
            // Only used by verilog
            module: self.module.unwrap_or("".into()),
            // Local identifier `x`
            base_name: self.base_name.unwrap_or("".into()),
            // Almost always the same as `base_name`, but with name mangling can be relevant
            pretty_name: self.pretty_name.unwrap_or("".into()),
            // Currently set to C. Consider creating a "rust" mode and using it in cbmc
            // https://github.com/model-checking/kani/issues/1
            mode: self.mode.to_string().into(),

            // global properties
            is_type: self.is_type,
            is_macro: self.is_macro,
            is_exported: self.is_exported,
            is_input: self.is_input,
            is_output: self.is_output,
            is_state_var: self.is_state_var,
            is_property: self.is_property,

            // ansi-C properties
            is_static_lifetime: self.is_static_lifetime,
            is_thread_local: self.is_thread_local,
            is_lvalue: self.is_lvalue,
            is_file_local: self.is_file_local,
            is_extern: self.is_extern,
            is_volatile: self.is_volatile,
            is_parameter: self.is_parameter,
            is_auxiliary: self.is_auxiliary,
            is_weak: self.is_weak,
        }
    }
}

impl goto_program::SymbolTable {
    pub fn to_irep(&self) -> super::SymbolTable {
        let mm = self.machine_model();
        let mut st = super::SymbolTable::new();
        for (_key, value) in self.iter() {
            st.insert(value.to_irep(mm, arena))
        }
        st
    }
}

impl ToIrep for Type {
    fn to_irep(&self, mm: &MachineModel) -> Irep {
        match self {
            Type::Array { typ, size } => {
                //CBMC expects the size to be a signed int constant.
                let size = Expr::int_constant(*size, Type::ssize_t());
                Irep {
                    id: IrepId::Array,
                    sub: vec![typ.to_irep(mm, arena)],
                    named_sub: linear_map![(IrepId::Size, size.to_irep(mm, arena))],
                }
            }
            //TODO make from_irep that matches this.
            Type::CBitField { typ, width } => Irep {
                id: IrepId::CBitField,
                sub: vec![typ.to_irep(mm, arena)],
                named_sub: linear_map![(IrepId::Width, Irep::just_int_id(*width))],
            },
            Type::Bool => Irep::just_id(IrepId::Bool),
            Type::CInteger(CIntType::Bool) => Irep {
                id: IrepId::CBool,
                sub: vec![],
                named_sub: linear_map![(IrepId::Width, Irep::just_int_id(mm.bool_width))],
            },
            Type::CInteger(CIntType::Char) => Irep {
                id: if mm.char_is_unsigned { IrepId::Unsignedbv } else { IrepId::Signedbv },
                sub: vec![],
                named_sub: linear_map![(IrepId::Width, Irep::just_int_id(mm.char_width),)],
            },
            Type::CInteger(CIntType::Int) => Irep {
                id: IrepId::Signedbv,
                sub: vec![],
                named_sub: linear_map![(IrepId::Width, Irep::just_int_id(mm.int_width),)],
            },
            Type::CInteger(CIntType::LongInt) => Irep {
                id: IrepId::Signedbv,
                sub: vec![],
                named_sub: linear_map![(IrepId::Width, Irep::just_int_id(mm.long_int_width),)],
            },
            Type::CInteger(CIntType::SizeT) => Irep {
                id: IrepId::Unsignedbv,
                sub: vec![],
                named_sub: linear_map![(IrepId::Width, Irep::just_int_id(mm.pointer_width),)],
            },
            Type::CInteger(CIntType::SSizeT) => Irep {
                id: IrepId::Signedbv,
                sub: vec![],
                named_sub: linear_map![(IrepId::Width, Irep::just_int_id(mm.pointer_width),)],
            },
            Type::Code { parameters, return_type } => Irep {
                id: IrepId::Code,
                sub: vec![],
                named_sub: linear_map![
                    (
                        IrepId::Parameters,
                        Irep::just_sub(parameters.iter().map(|x| x.to_irep(mm, arena)).collect()),
                    ),
                    (IrepId::ReturnType, return_type.to_irep(mm, arena)),
                ],
            },
            Type::Constructor => Irep::just_id(IrepId::Constructor),
            Type::Double => Irep {
                id: IrepId::Floatbv,
                sub: vec![],
                named_sub: linear_map![
                    (IrepId::F, Irep::just_int_id(52)),
                    (IrepId::Width, Irep::just_int_id(64)),
                    (IrepId::CCType, Irep::just_id(IrepId::Double)),
                ],
            },
            Type::Empty => Irep::just_id(IrepId::Empty),
            // CMBC currently represents these as 0 length arrays.
            Type::FlexibleArray { typ } => {
                //CBMC expects the size to be a signed int constant.
                let size = Type::ssize_t().zero();
                Irep {
                    id: IrepId::Array,
                    sub: vec![typ.to_irep(mm, arena)],
                    named_sub: linear_map![(IrepId::Size, size.to_irep(mm, arena))],
                }
            }
            Type::Float => Irep {
                id: IrepId::Floatbv,
                sub: vec![],
                named_sub: linear_map![
                    (IrepId::F, Irep::just_int_id(23)),
                    (IrepId::Width, Irep::just_int_id(32)),
                    (IrepId::CCType, Irep::just_id(IrepId::Float)),
                ],
            },
            Type::Float16 => Irep {
                id: IrepId::Floatbv,
                sub: vec![],
                // Fraction bits: 10
                // Exponent width bits: 5
                // Sign bit: 1
                named_sub: linear_map![
                    (IrepId::F, Irep::just_int_id(10)),
                    (IrepId::Width, Irep::just_int_id(16)),
                    (IrepId::CCType, Irep::just_id(IrepId::Float16)),
                ],
            },
            Type::Float128 => Irep {
                id: IrepId::Floatbv,
                sub: vec![],
                // Fraction bits: 112
                // Exponent width bits: 15
                // Sign bit: 1
                named_sub: linear_map![
                    (IrepId::F, Irep::just_int_id(112)),
                    (IrepId::Width, Irep::just_int_id(128)),
                    (IrepId::CCType, Irep::just_id(IrepId::Float128)),
                ],
            },
            Type::IncompleteStruct { tag } => Irep {
                id: IrepId::Struct,
                sub: vec![],
                named_sub: linear_map![
                    (IrepId::Tag, Irep::just_string_id(tag.to_string())),
                    (IrepId::Incomplete, Irep::one()),
                ],
            },
            Type::IncompleteUnion { tag } => Irep {
                id: IrepId::Union,
                sub: vec![],
                named_sub: linear_map![
                    (IrepId::Tag, Irep::just_string_id(tag.to_string())),
                    (IrepId::Incomplete, Irep::one()),
                ],
            },
            Type::InfiniteArray { typ } => {
                let infinity = Irep::just_id(IrepId::Infinity).with_type(&Type::ssize_t(), mm);
                Irep {
                    id: IrepId::Array,
                    sub: vec![typ.to_irep(mm, arena)],
                    named_sub: linear_map![(IrepId::Size, infinity)],
                }
            }
            Type::Integer => Irep::just_id(IrepId::Integer),
            Type::Pointer { typ } => Irep {
                id: IrepId::Pointer,
                sub: vec![typ.to_irep(mm, arena)],
                named_sub: linear_map![(IrepId::Width, Irep::just_int_id(mm.pointer_width),)],
            },
            Type::Signedbv { width } => Irep {
                id: IrepId::Signedbv,
                sub: vec![],
                named_sub: linear_map![(IrepId::Width, Irep::just_int_id(*width))],
            },
            Type::Struct { tag, components } => Irep {
                id: IrepId::Struct,
                sub: vec![],
                named_sub: linear_map![
                    (IrepId::Tag, Irep::just_string_id(tag.to_string())),
                    (
                        IrepId::Components,
                        Irep::just_sub(components.iter().map(|x| x.to_irep(mm, arena)).collect()),
                    ),
                ],
            },
            Type::StructTag(name) => Irep {
                id: IrepId::StructTag,
                sub: vec![],
                named_sub: linear_map![(
                    IrepId::Identifier,
                    Irep::just_string_id(name.to_string()),
                )],
            },
            Type::TypeDef { name, typ } => typ
                .to_irep(mm, arena)
                .with_named_sub(IrepId::CTypedef, Irep::just_string_id(name.to_string())),

            Type::Union { tag, components } => Irep {
                id: IrepId::Union,
                sub: vec![],
                named_sub: linear_map![
                    (IrepId::Tag, Irep::just_string_id(tag.to_string())),
                    (
                        IrepId::Components,
                        Irep::just_sub(components.iter().map(|x| x.to_irep(mm, arena)).collect()),
                    ),
                ],
            },
            Type::UnionTag(name) => Irep {
                id: IrepId::UnionTag,
                sub: vec![],
                named_sub: linear_map![(
                    IrepId::Identifier,
                    Irep::just_string_id(name.to_string()),
                )],
            },
            Type::Unsignedbv { width } => Irep {
                id: IrepId::Unsignedbv,
                sub: Vec::new(),
                named_sub: linear_map![(IrepId::Width, Irep::just_int_id(*width))],
            },
            Type::VariadicCode { parameters, return_type } => Irep {
                id: IrepId::Code,
                sub: vec![],
                named_sub: linear_map![
                    (
                        IrepId::Parameters,
                        Irep::just_sub(parameters.iter().map(|x| x.to_irep(mm, arena)).collect())
                            .with_named_sub(IrepId::Ellipsis, Irep::one()),
                    ),
                    (IrepId::ReturnType, return_type.to_irep(mm, arena)),
                ],
            },
            Type::Vector { typ, size } => {
                let size = Expr::int_constant(*size, Type::ssize_t());
                Irep {
                    id: IrepId::Vector,
                    sub: vec![typ.to_irep(mm, arena)],
                    named_sub: linear_map![(IrepId::Size, size.to_irep(mm, arena))],
                }
            }
        }
    }
}
