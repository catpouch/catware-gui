use core::f64;
use std::{cell::RefCell, collections::HashMap};

use egui_plot::PlotBounds;
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
    hardcoded_funcs: [&'static str; 18], // list of names of hardcoded functions to prevent user from defining an existing function twice
    plot_resolution: usize,
    pub plot_points: RefCell<Vec<[f64; 2]>>,
    plot_func: RefCell<String>,
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
            hardcoded_funcs: ["ln", "log2", "log10", "sin", "asin", "sinh", "asinh", "cos", "acos", "cosh", "acosh", "tan", "atan", "tanh", "atanh", "sqrt", "cbrt", "abs"],
            plot_resolution: 100,
            plot_points: Vec::new().into(),
            plot_func: RefCell::new("x".to_owned())
        }
    }

    // evaluates functions (user-defined and built in)
    fn handle_func(&self, primary: Pair<Rule>, context: &HashMap<String, f64>) -> Result<f64, Box<dyn std::error::Error>> {
        let mut primary_inner = primary.into_inner();
        let first = primary_inner.next().unwrap();
        if first.as_rule() != Rule::id {
            self.eval_expr(first.into_inner())
        } else {
            // let args: Vec<f64> = primary_inner.next().unwrap().into_inner().map(|a| self.eval_expr_context(a.into_inner(), &context)).collect();
            let mut arg_pairs: Pairs<Rule> = primary_inner.next().unwrap().into_inner();
            if self.funcs_context.contains_key(first.as_str()) {
                let func_def = &self.funcs_context[first.as_str()];
                if arg_pairs.len() != func_def.signature.len() {return Err(Box::new(std::fmt::Error))}
                let args: Vec<f64> = arg_pairs.map(|a| -> Result<f64, Box<dyn std::error::Error>> {Ok(self.eval_expr_context(a.into_inner(), &context)?)}).collect::<Result<_, _>>()?; // may god have mercy on my soul
                let new_context: HashMap<String, f64> = args.iter().enumerate().map(|a| {(func_def.signature[a.0].clone(), a.1.to_owned())}).collect::<HashMap<_, _, _>>();
                let parsed_pairs=  CatwareParser::parse(Rule::program, &func_def.definition)?.next().unwrap().into_inner().next().unwrap().into_inner();
                self.eval_expr_context(parsed_pairs, &new_context)
            } else {
                // TODO: this is not good
                match arg_pairs.len() {
                    1 => {
                        let output: f64;
                        if first.as_str() == "plot" {
                            println!("{:#?}", arg_pairs);
                            let mut plot_func_ref = self.plot_func.borrow_mut();
                            *plot_func_ref = arg_pairs.as_str().to_owned();
                            let _ = self.plot_func(&mut arg_pairs, [-10.0, 10.0]);
                            return Err(Box::new(std::fmt::Error))
                        }
                        let args: Vec<f64> = arg_pairs.map(|a| -> Result<f64, Box<dyn std::error::Error>> {Ok(self.eval_expr_context(a.into_inner(), &context)?)}).collect::<Result<_, _>>()?;
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
                            _ => return Err(Box::new(std::fmt::Error))
                        }
                        Ok(output)
                    },
                    2 => {
                        let args: Vec<f64> = arg_pairs.map(|a| -> Result<f64, Box<dyn std::error::Error>> {Ok(self.eval_expr_context(a.into_inner(), &context)?)}).collect::<Result<_, _>>()?;
                        match first.as_str() {
                            "nrt" => Ok(args[0].powf(1.0 / args[1])),
                            _ => Err(Box::new(std::fmt::Error))
                        }
                    }
                    _ => Err(Box::new(std::fmt::Error))
                }
            }
        }
    }

    fn plot_func(&self, arg_pairs: &mut Pairs<Rule>, bounds: [f64; 2]) -> Result<(), Box<dyn std::error::Error>> {
        let expression = arg_pairs.next().unwrap();
        // let x_var_pair = arg_pairs.next().unwrap().into_inner().next().unwrap();
        // if x_var_pair.as_rule() != Rule::id {return}
        // let x_var = x_var_pair.as_str().to_owned();
        let mut points = self.plot_points.borrow_mut();
        points.clear();
        for i in 0..self.plot_resolution {
            let x_val = ( i as f64 / self.plot_resolution as f64 ) * (bounds[1] - bounds[0]) + bounds[0];
            let context = HashMap::from([
                ("x".to_owned(), x_val)
            ]);
            let point_y = self.eval_expr_context(expression.clone().into_inner(), &context)?;
            points.push([x_val, point_y]);
        }
        Ok(())
    }

    // handles assigning values to vars & functions
    fn handle_assignment(&mut self, pairs: Pairs<Rule>) -> Result<(), Box<dyn std::error::Error>> {
        let mut mut_pairs = pairs.clone();
        let lhs = mut_pairs.next().unwrap();
        if lhs.as_rule() == Rule::id {
            let val = mut_pairs.skip(1).next().unwrap().into_inner();
            if self.hardcoded_vars.contains(&lhs.as_str()) {return Err(Box::new(std::fmt::Error))}
            self.vars_context.insert(lhs.as_str().to_owned(), self.eval_expr(val)?);
        } else {
            let mut id= lhs.into_inner();
            let name = id.next().unwrap().as_str();
            if self.hardcoded_funcs.contains(&name) {return Err(Box::new(std::fmt::Error))}
            self.funcs_context.insert(name.to_owned(), CatwareFunc::new(id.next().unwrap().into_inner().map(|a| a.as_str().to_owned()).collect(), mut_pairs.skip(1).next().unwrap().as_str().to_owned()));
        }
        Ok(())
    }

    // wrapper to parse expressions using the CatwareCalc's context
    fn eval_expr(&self, pairs: Pairs<Rule>) -> Result<f64, Box<dyn std::error::Error>> {
        self.eval_expr_context(pairs, &self.vars_context)
    }

    // handles the variety of operations within a statement
    fn eval_expr_context(&self, pairs: Pairs<Rule>, context: &HashMap<String, f64>) -> Result<f64, Box<dyn std::error::Error>> {
        self.pratt
            .map_primary(|primary| match primary.as_rule() {
                Rule::num  => Ok(primary.as_str().parse::<f64>()?),
                Rule::expr => self.eval_expr_context(primary.into_inner(), &context),
                Rule::func => self.handle_func(primary, context),
                Rule::id   => Ok(context.get(primary.as_str()).copied().ok_or(Box::new(std::fmt::Error))?), // dear god
                _          => unreachable!(),
            })
            .map_prefix(|op, rhs| match op.as_rule() {
                Rule::neg  => Ok(-rhs?),
                _          => unreachable!(),
            })
            // .map_postfix(|lhs, op| match op.as_rule() {
            //     Rule::fac  => (1..lhs+1).product(),
            //     _          => unreachable!(),
            // })
            .map_infix(|lhs, op, rhs| {
                let lhs_val = lhs?;
                let rhs_val = rhs?;
                match op.as_rule() {
                    Rule::add  => Ok(lhs_val + rhs_val),
                    Rule::sub  => Ok(lhs_val - rhs_val),
                    Rule::mul  => Ok(lhs_val * rhs_val),
                    Rule::div  => Ok(lhs_val / rhs_val),
                    Rule::pow  => Ok(lhs_val.powf(rhs_val)),
                    _          => unreachable!(),
                }
            })
            .parse(pairs)
    }

    pub fn refresh_graph(&self, bounds: PlotBounds) -> Result<(), Box<dyn std::error::Error>> {
        let plot_func = self.plot_func.borrow(); // just realized these names are terrible. i will not be fixing it
        let mut pairs = CatwareParser::parse(Rule::program, plot_func.as_str())?.next().unwrap().into_inner();
        self.plot_func(&mut pairs, [bounds.min()[0], bounds.max()[0]])
        // self.plot_func(&mut CatwareParser::parse(Rule::program, self.plot_func.borrow().as_str()).unwrap(), [bounds.min()[0], bounds.max()[0]]);
    }

    pub fn eval_string(&mut self, input: &str) -> Result<f64, Box<dyn std::error::Error>> {
        let first_parsed = CatwareParser::parse(Rule::program, input)?.next().unwrap().into_inner().next().unwrap();
        if first_parsed.as_rule() == Rule::assignment {
            let _ = self.handle_assignment(first_parsed.into_inner());
            Err(Box::new(std::fmt::Error))
        } else {
            self.eval_expr(first_parsed.into_inner())
        }
    }
}
