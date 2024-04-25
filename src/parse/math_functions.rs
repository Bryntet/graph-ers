use std::collections::HashMap;
use egui_plot::PlotPoints;
use itertools::Itertools;
use regex::Regex;
use lazy_static::lazy_static;



#[derive(Debug, PartialEq)]
struct Expression {
    var_name: Option<String>,
    multiplier: Option<f64>,
    exponent: Option<f64>,
    combination: Option<Box<Expression>>,
    is_negative: bool,
}
impl Expression {
    fn new(input: &str) -> Result<Self, ParseError> {
        Self::split_on_operation(input,input.starts_with('-'))
    }
    fn split_on_operation(input: &str, current_is_negative: bool) -> Result<Self, ParseError> {
        let mut other_is_negative = false;
        let (me, other) = match input.split_once('+') {Some((me,other)) => (Some(me),Some(other)),None=> { match input.split_once('-') {
            Some((me,other)) => {
                other_is_negative = true;
                (Some(me),Some(other))
            },
            None => (Some(input),None)
        }}};
        let me = me.unwrap_or(input);

        Self::parse(me, current_is_negative, if let Some(other) = other { Some(Self::split_on_operation(other, other_is_negative)?)} else {None})
    }


    fn parse(input:&str, is_negative: bool, combination: Option<Self>) -> Result<Self, ParseError> {
        let mut divisions = 0.;
        let combination = combination.map(Box::new);
        let expression_match = Regex::new(r"(?P<Multiplier>-?\d+)?\*?((?P<Variable>[a-z]+)(?:\^(?P<VariableExponent>\d+))?)?").expect("Regex should compile");
        let caps = expression_match.captures(input).ok_or(ParseError::UnableToParse)?;
        let (mult, var,var_exponent) = (caps.name("Multiplier"), caps.name("Variable"), caps.name("VariableExponent"));
         match (mult, var, var_exponent) {
            (Some(constant_num),None,None) => Ok(Self {
                var_name: None,
                multiplier: Some(constant_num.as_str().parse().expect("Regex proves should be digits")),
                exponent: None,
                combination,
                is_negative
            }),
            (None,None,None) => Err(ParseError::UnableToParse),
            _ => {
                Ok(Self {
                    var_name: var.map(|v| v.as_str().to_string()),
                    multiplier: mult.map(|m| m.as_str().parse().expect("Regex proves should be digits")),
                    exponent: var_exponent.map(|e| e.as_str().parse().expect("Regex proves should be digits")),
                    combination,
                    is_negative
                })
            }
        }
    }

    fn calculate(&self, variable_values: &HashMap<String,f64>) -> f64 {
        ((match self.var_name.as_ref().and_then(|name|variable_values.get(name)) {
            Some(var) => {
               (match  self.exponent {
                    Some(exp) => var.powf(exp),
                    None => *var
                }) * self.multiplier.unwrap_or(1.)
            }
            None => {
                if let Some(num) = self.multiplier {
                    num
                } else {
                    0.
                }
            }
        }) + self.combination.as_ref().map(|c| c.calculate(variable_values)).unwrap_or(0.)) * if self.is_negative {-1.} else {1.}
    }

    pub fn variables(&self) -> Vec<String> {
        let mut out_vec = if let Some(name) = self.var_name.clone() {
            vec![name]
        } else {vec![]};
        if let Some(comb) = self.combination.as_ref() {
            out_vec.append(&mut comb.variables());
        }
        out_vec
    }
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum ParseError {
    #[error("All variables defined in the function are not used.")]
    VariableDefinitionAndUseMismatch,
    #[error("Unknown variable in expression: \"{0}\"")]
    UnknownVariable(String),
    #[error("Unable to parse")]
    UnableToParse,
    #[error("Unable to find required argument: {0} in input")]
    UnableToFind(String)
}
#[derive(Debug, PartialEq)]
pub struct Function{
    name: String,
    expression: Expression,
    x_value: f64,
    internal_offset: f64
}

impl Function {
    pub fn new(input: &str) -> Result<Self, ParseError> {
        Self::regex(input)
    }

    pub fn y_pos(&self, variables: &HashMap<String,f64>) -> f64 {
        self.expression.calculate(variables)
    }
    
    fn current_point(&self,variables:&HashMap<String,f64>) -> [f64;2] {
        [self.x_value,self.y_pos(variables)]
    }
    
    fn generate_naive_map(&self) -> HashMap<String,f64> {
        let mut map = HashMap::new();
        for var in self.expression.variables() {
            map.insert(var,self.x_value);
        }
        map
    }
    
    fn regex(input: &str) -> Result<Self, ParseError> {
        let function_match = Regex::new(r"(?P<FunctionName>\w+)\((?P<FunctionVariables>(?:[a-z]+,?)+)\)=(?P<Expression>(?:[a-z01-9^*/]*[+-]?)+)").expect("Regex should compile");
        let captures = function_match.captures(input).ok_or(ParseError::UnableToParse)?;

        let function_name = captures.name("FunctionName").ok_or(ParseError::UnableToFind("function name".to_string()))?.as_str();
        let function_args = captures.name("FunctionVariables").ok_or(ParseError::UnableToFind("function variables".to_string()))?.as_str().split(',').map(String::from).collect_vec();
        let expression = Expression::new(captures.name("Expression").ok_or(ParseError::UnableToFind("function expression".to_string()))?.as_str())?;

        for arg in &function_args {
            if !expression.variables().contains(arg) {
                return Err(ParseError::VariableDefinitionAndUseMismatch);
            }
        }
        Ok(Self {
            name: function_name.to_string(),
            expression,
            x_value: 0.0,
            internal_offset: 0.0,
        })

    }
    
    pub fn into_plot_points(mut self, min_x: f64, max_x:f64) -> PlotPoints {
        let mut points = Vec::new();

        self.internal_offset = (max_x - min_x)/2000.;
        self.x_value = min_x + self.internal_offset;
        while self.x_value < max_x {
            self.next();
            points.push(self.current_point(&self.generate_naive_map()));
        }
        PlotPoints::from(points)
    }
}



impl Iterator for Function {
    type Item = (f64,f64);

    fn next(&mut self) -> Option<Self::Item> {
        self.x_value += self.internal_offset;
        let y_pos = self.y_pos(&self.generate_naive_map());
        Some((self.x_value,y_pos))
    }
}


#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use crate::parse::Function;
    use crate::parse::math_functions::{Expression, ParseError};

   

    #[test]
    fn regex() {
        let test_fn = "f(t,b)=2t+5b";
        assert!(dbg!(Function::regex(test_fn)).is_ok());
    }

    #[test]
    fn math_function_calculate() {
        let input = "f(test)=2test";
        dbg!(Function::regex(input));
    }

    #[test]
    fn unused_variables() {
        let test_fn = "f(t,b,c)=2t+5b";
        assert_eq!(Function::regex(test_fn), Err(ParseError::VariableDefinitionAndUseMismatch));
    }

    #[test]
    fn expression_parse() {
        assert_eq!(Expression::new("1+2-54").unwrap().calculate(&HashMap::new()),-51.);
    }
    #[test]
    fn empty_expression() {
        dbg!(Expression::new("10").unwrap().calculate(&HashMap::new()));
    }
}