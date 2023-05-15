use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

#[allow(non_camel_case_types)]
#[derive(PartialEq)]
pub enum ParseErr {
    UnclosedString,
    Unexpected_EOF,
    Unexpected_EOF_AfterPars,
    Unexpected_EOF_AfterDot,
    UnexpectedToken(String),
    ClosingParExpected(String),
    ClosingParExpected_EOF,
}

#[derive(PartialEq)]
pub enum EvalErr {
    PairRequired(String),
    ListRequired(String),
    UnboundVariable(String),
    NumericArgsRequiredFor(String),
    DivisionByZero(),
    IllegalObjectAsAFunction(String),
    TooFewArguments(String),
    TooManyArguments(String),
    WrongAgrsNum(String, usize, usize),
    NeedAtLeastArgs(String, usize, usize),
    ExpectedSymbolForFunctionName(String),
    ExpectedSymbolForArgument(String),
    ArgumentDuplication(String),
    WrongArgsList(String),
    ApplyNeedsProperList(String),
    UnequalMapLists(),
    LetNeedSymbolForBinding(String),
    LetNeedListForBinding(String),
    WrongDefineArgument(String),
    CondNeedsClause(),
    CondEmptyClause(),
    EmptyFunctionBody(),
}

impl Error for EvalErr {}

impl Error for ParseErr {}

#[rustfmt::skip]
impl Display for ParseErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseErr::UnclosedString =>
                write!(f, "String literal didn't close"),
            ParseErr::Unexpected_EOF =>
                write!(f, "Unexpected end of input"),
            ParseErr::Unexpected_EOF_AfterPars =>
                write!(f, "Unexpected end of input after opening parenthesis"),
            ParseErr::Unexpected_EOF_AfterDot =>
                write!(f, "Unexpected end of input after a dot"),
            ParseErr::UnexpectedToken(token) =>
                write!(f, "Unexpected token: {:?}", token),
            ParseErr::ClosingParExpected(token) =>
                write!(f, "Closing parenthesis expected, '{:?}' found", token),
            ParseErr::ClosingParExpected_EOF =>
                write!(f, "Closing parenthesis expected, found end of input)")
        }
    }
}

#[rustfmt::skip]
impl Display for EvalErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            EvalErr::WrongAgrsNum(name, _, actual) =>
                write!(f, "Wrong number of arguments for {}: {}", name, actual),
            EvalErr::PairRequired(obj) =>
                write!(f, "pair required but got {}", obj),
            EvalErr::NeedAtLeastArgs(name, expected, actual) =>
                write!(f, "\"{}\" need at least {} arguments, found {}", name, expected, actual),
            EvalErr::IllegalObjectAsAFunction(obj) =>
                write!(f, "Illegal object used as a function: {}", obj),
            EvalErr::ListRequired(obj) => 
                write!(f, "Proper list required for function application: {}", obj),
            EvalErr::TooFewArguments(name) =>
                write!(f, "Too few arguments given to {}", name),
            EvalErr::TooManyArguments(name) =>
                write!(f, "Too many arguments given to {}", name),
            EvalErr::NumericArgsRequiredFor(name) =>
                write!(f, "Numeric arguments required for {}", name),
            EvalErr::DivisionByZero() =>
                write!(f, "Division by zero"),
            EvalErr::UnboundVariable(name) =>
                write!(f, "Unbound variable {}", name),
            EvalErr::ExpectedSymbolForFunctionName(obj) =>
                write!(f, "Expected a symbol for function name, found {}", obj),
            EvalErr::ExpectedSymbolForArgument(obj) =>
                write!(f, "Expected a symbol for argument id, found {}", obj),
            EvalErr::ArgumentDuplication(name) =>
                write!(f, "Duplication of argument id: {}", name),
            EvalErr::ApplyNeedsProperList(obj) =>
                write!(f, "'Apply' needs proper list as the last argument, got {}", obj),
            EvalErr::UnequalMapLists() =>
                write!(f, "List of wrong length for 'map'"),
            EvalErr::LetNeedSymbolForBinding(obj) =>
                write!(f, "Need symbol for 'let' binding, got {}", obj),
            EvalErr::LetNeedListForBinding(obj) => 
                write!(f, "'let' needs a list of length 2 for binding, got '{}'", obj),
            EvalErr::WrongDefineArgument(obj) =>
                write!(f, "Wrong first argument for 'define': {}", obj),
            EvalErr::WrongArgsList(obj) =>
                write!(f, "Wrong arguments list syntax: {}", obj),
            EvalErr::CondNeedsClause() =>
                write!(f, "'cond' needs at least 1 clause"),
            EvalErr::CondEmptyClause() =>
                write!(f, "Empty clause for 'cond'"),
            EvalErr::EmptyFunctionBody() =>
                write!(f, "Empty function body")
        }
    }
}

impl Debug for ParseErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl Debug for EvalErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}
