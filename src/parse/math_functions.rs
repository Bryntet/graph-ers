use egui_plot::PlotPoints;
use itertools::Itertools;
use regex::Regex;
use lazy_static::lazy_static;

pub struct Function{
    rate_of_change: f64,
    y_offset: f64,
    x_value: f64,
    internal_offset: f64
}

struct Expression {
    var_name: String,
    multiplier: f64,
    exponent: f64,
    variable_value: f64,
    combination: Option<Box<dyn Calculate>>
}


trait Calculate {
    fn calculate(&self) -> f64;
}

impl Calculate for Expression {
    fn calculate(&self) -> f64 {
        self.variable_value.powf(self.exponent) * self.multiplier
    }
}
#[derive(thiserror::Error, Debug, PartialEq)]
enum ParseError {
    #[error("All variables defined in the function are not used.")]
    VariableDefinitionAndUseMismatch,
    #[error("Unknown variable in expression: \"{0}\"")]
    UnknownVariable(String),
    #[error("Unable to parse")]
    UnableToParse,
    #[error("Unable to find required argument: {0} in input")]
    UnableToFind(String)
}


impl Function {
    pub fn new(rate_of_change: f64, y_offset: f64) -> Self {
        Self {
            rate_of_change,
            y_offset,
            x_value: 0.,
            internal_offset: 0.01
        }
    }
    fn y_pos(&self) -> f64 {
        (self.x_value*self.rate_of_change).sin()+self.y_offset
    }
    
    fn current_point(&self) -> [f64;2] {
        [self.x_value,self.y_pos()]
    }
    
    fn regex(input: &str) -> Result<(), ParseError> {
        lazy_static! {
            static ref FUNCTION_MATCH: Regex = Regex::new(r"(?P<FunctionName>\w+)\((?P<FunctionVariables>(?:[a-z]+,?)+)\)=(?P<Expression>(?:\d*[a-z]*[+-]?)+)").expect("Regex should work");
        }
        let captures = FUNCTION_MATCH.captures(input).ok_or(ParseError::UnableToParse)?;

        let function_name = captures.name("FunctionName").ok_or(ParseError::UnableToFind("function name".to_string()))?.as_str();
        let function_args = captures.name("FunctionVariables").ok_or(ParseError::UnableToFind("function variables".to_string()))?.as_str().split(',').collect_vec();
        let expression = captures.name("Expression").ok_or(ParseError::UnableToFind("function expression".to_string()))?.as_str();
        for arg in &function_args {
            if !expression.contains(arg) {
                return Err(ParseError::VariableDefinitionAndUseMismatch);
            }
        }
        dbg!(function_name,function_args,expression);
        Ok(())

    }
    
    pub fn into_plot_points(mut self, min_x: f64, max_x:f64) -> PlotPoints {
        let mut points = Vec::new();

        self.internal_offset = (max_x - min_x)/2000.;
        self.x_value = min_x + self.internal_offset;
        while self.x_value < max_x {
            self.next();
            points.push(self.current_point());
        }
        PlotPoints::from(points)
    }
}



impl Iterator for Function {
    type Item = (f64,f64);

    fn next(&mut self) -> Option<Self::Item> {
        self.x_value += self.internal_offset;
        let y_pos = self.y_pos();
        Some((self.x_value,y_pos))
    }
}


#[cfg(test)]
mod test {
    use crate::parse::Function;
    use crate::parse::math_functions::ParseError;

    #[test]
    fn test() {
        let f = super::Function{rate_of_change: 1., y_offset: 0., x_value: 0., internal_offset: 0.0 };
        let mut iter = f.into_iter();
        assert_eq!(iter.next(), Some((0.000000001,0.000000001)));
        assert_eq!(iter.next(), Some((0.000000001*2.,0.000000001*2.)));
    }

    #[test]
    fn regex() {
        let test_fn = "f(t,b,c)=2t+5b";
        dbg!(Function::regex(test_fn));
    }
    
    #[test]
    fn unused_variables() {
        let test_fn = "f(t,b,c)=2t+5b";
        assert_eq!(Function::regex(test_fn), Err(ParseError::VariableDefinitionAndUseMismatch));
    }
}