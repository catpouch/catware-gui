use core::f64;
use std::collections::HashMap;

use pest::{iterators::{Pair, Pairs}, pratt_parser::{Assoc, Op, PrattParser}, Parser};
use pest_derive::Parser;

// CatwareParser does the conversion from text to pairs (pest's way of breaking up text)
#[derive(Parser)]
#[grammar="math.pest"]
struct CatwareParser;

// struct for user-defined functions
// i'm aware that reparsing the function every time is definitely not very fast, it will be changed in the future
struct CatwareFunc {
    signature: Vec<String>, // argument names in order
    definition: String // function body
}

impl CatwareFunc {
    fn new(sig: Vec<String>, def: String) -> Self {
        CatwareFunc {
            signature: sig,
            definition: def
        }
    }
}

// for pretty printing
impl std::fmt::Debug for CatwareFunc {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.debug_struct("CatwareFunc")
            .field("signature", &self.signature)
            .field("definition", &self.definition)
            .finish()
    }
}

// struct that takes in a string and outputs a float. will be changed to include actual error handling in the future
pub struct CatwareCalc {
    pratt: PrattParser<Rule>, // this is a pest struct. takes in pairs and runs code with them with operator precedence
    vars_context: HashMap<String, f64>, // contains user-defined and hardcoded values
    hardcoded_vars: [&'static str; 3], // list of names of hardcoded values to prevent user from overwriting them
    funcs_context: HashMap<String, CatwareFunc>, // contains user-defined functions
    hardcoded_funcs: [&'static str; 18] // list of names of hardcoded functions to prevent user from defining an existing function twice
}

impl CatwareCalc {
    pub fn new() -> Self {
        Self {
            // operator precendence defined by order they are applied (last ones applied will be of higher precendence)
            pratt: {
                PrattParser::new()
                .op(Op::infix(Rule::add, Assoc::Left) | Op::infix(Rule::sub, Assoc::Right))
                .op(Op::infix(Rule::mul, Assoc::Left) | Op::infix(Rule::div, Assoc::Left))
                .op(Op::infix(Rule::pow, Assoc::Right))
                .op(Op::postfix(Rule::fac))
                .op(Op::prefix(Rule::neg))
                .op(Op::infix(Rule::eq, Assoc::Left))
            },
            vars_context: HashMap::from([
                ("pi".to_owned(), f64::consts::PI),
                ("tau".to_owned(), f64::consts::TAU),
                ("e".to_owned(), f64::consts::E)
            ]),
            hardcoded_vars: ["pi", "tau", "e"],
            funcs_context: HashMap::new(),
            // i am aware this is kind of awful
            hardcoded_funcs: ["ln", "log2", "log10", "sin", "asin", "sinh", "asinh", "cos", "acos", "cosh", "acosh", "tan", "atan", "tanh", "atanh", "sqrt", "cbrt", "abs"]
        }
    }

    fn handle_func(&self, primary: Pair<Rule>, context: &HashMap<String, f64>) -> Option<f64> {
        let mut primary_inner = primary.into_inner();
        let first = primary_inner.next().unwrap();
        if first.as_rule() != Rule::id {
            Some(self.parse_expr(first.into_inner()))
        } else {
            let args: Vec<f64> = primary_inner.next().unwrap().into_inner().map(|a| self.parse_expr_context(a.into_inner(), &context)).collect();
            if self.funcs_context.contains_key(first.as_str()) {
                let func_def = &self.funcs_context[first.as_str()];
                if args.len() != func_def.signature.len() {return None}
                let new_context: HashMap<String, f64> = args.iter().enumerate().map(|a| {(func_def.signature[a.0].clone(), a.1.to_owned())}).collect::<HashMap<_, _, _>>();
                let parsed_pairs=  CatwareParser::parse(Rule::program, &func_def.definition).expect("parse failed").next().unwrap().into_inner().next().unwrap().into_inner();
                Some(self.parse_expr_context(parsed_pairs, &new_context))
            } else {
                match args.len() {
                    1 => {
                        let output: f64;
                        match first.as_str() {
                            "ln" => output = args[0].ln(),
                            "log2" => output = args[0].log2(),
                            "log10" => output = args[0].log10(),
                            "sin" => output = args[0].sin(),
                            "asin" => output = args[0].asin(),
                            "sinh" => output = args[0].sinh(),
                            "asinh" => output = args[0].asinh(),
                            "cos" => output = args[0].cos(),
                            "acos" => output = args[0].acos(),
                            "cosh" => output = args[0].cosh(),
                            "acosh" => output = args[0].acosh(),
                            "tan" => output = args[0].tan(),
                            "atan" => output = args[0].atan(),
                            "tanh" => output = args[0].tanh(),
                            "atanh" => output = args[0].atanh(),
                            "sqrt" => output = args[0].sqrt(),
                            "cbrt" => output = args[0].cbrt(),
                            "abs" => output = args[0].abs(),
                            _ => return None
                        }
                        return Some(output)
                    }
                    _ => None
                }
            }
        }
    }

    fn handle_assignment(&mut self, pairs: Pairs<Rule>) {
        let mut mut_pairs = pairs.clone();
        let lhs = mut_pairs.next().unwrap();
        if lhs.as_rule() == Rule::id {
            let val = mut_pairs.skip(1).next().unwrap().into_inner();
            if self.hardcoded_vars.contains(&lhs.as_str()) {return}
            self.vars_context.insert(lhs.as_str().to_owned(), self.parse_expr(val));
        } else {
            let mut id= lhs.into_inner();
            let name = id.next().unwrap().as_str();
            if self.hardcoded_funcs.contains(&name) {return}
            self.funcs_context.insert(name.to_owned(), CatwareFunc::new(id.next().unwrap().into_inner().map(|a| a.as_str().to_owned()).collect(), mut_pairs.skip(1).next().unwrap().as_str().to_owned()));
        }
    }

    fn parse_expr(&self, pairs: Pairs<Rule>) -> f64 {
        self.parse_expr_context(pairs, &self.vars_context)
    }

    fn parse_expr_context(&self, pairs: Pairs<Rule>, context: &HashMap<String, f64>) -> f64 {
        self.pratt
            .map_primary(|primary| match primary.as_rule() {
                Rule::num  => primary.as_str().parse().unwrap(),
                Rule::expr => self.parse_expr_context(primary.into_inner(), &context),
                Rule::func => self.handle_func(primary, context).unwrap(),
                Rule::id   => {context.get(primary.as_str()).unwrap().to_owned()},
                _          => unreachable!(),
            })
            .map_prefix(|op, rhs| match op.as_rule() {
                Rule::neg  => -rhs,
                _          => unreachable!(),
            })
            // .map_postfix(|lhs, op| match op.as_rule() {
            //     Rule::fac  => (1..lhs+1).product(),
            //     _          => unreachable!(),
            // })
            .map_infix(|lhs, op, rhs| match op.as_rule() {
                Rule::add  => lhs + rhs,
                Rule::sub  => lhs - rhs,
                Rule::mul  => lhs * rhs,
                Rule::div  => lhs / rhs,
                Rule::pow  => lhs.powf(rhs),
                _          => unreachable!(),
            })
            .parse(pairs)
    }

    pub fn parse_string(&mut self, input: &str) -> Option<f64> {
        // may god save us all
        let first_parsed = CatwareParser::parse(Rule::program, input).expect("parse failed").next().unwrap().into_inner().next().unwrap();
        if first_parsed.as_rule() == Rule::assignment {
            self.handle_assignment(first_parsed.into_inner());
            None
        } else {
            Some(self.parse_expr(first_parsed.into_inner()))
        }
    }
}

