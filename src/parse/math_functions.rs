use crate::parse::TokenQueue;
use egui_plot::PlotPoints;
use itertools::Itertools;
use regex::Regex;
use std::collections::HashMap;

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum ParseError {
    #[error("All variables defined in the function are not used.")]
    VariableDefinitionAndUseMismatch,
    #[error("Unknown variable in expression: \"{0}\"")]
    UnknownVariable(String),
    #[error("Unable to parse")]
    UnableToParse,
    #[error("Unable to find required argument: {0} in input")]
    UnableToFind(String),
    #[error("Token in invalid position")]
    InvalidTokenPosition,
}
#[derive(Debug, PartialEq)]
pub struct Function {
    pub name: String,
    x_value: f64,
    internal_offset: f64,
    test_ex: TokenQueue,
    variables: Vec<String>,
}

impl TryFrom<String> for Function {
    type Error = ParseError;
    fn try_from(input: String) -> Result<Self, ParseError> {
        Self::parse(&input)
    }
}

impl TryFrom<&str> for Function {
    type Error = ParseError;
    fn try_from(input: &str) -> Result<Self, ParseError> {
        Self::parse(input)
    }
}

impl Function {
    pub fn y_pos(&self, variables: &HashMap<String, f64>) -> Result<f64, ParseError> {
        self.test_ex.calculate(variables)
    }

    fn current_point(&self, variables: &HashMap<String, f64>) -> Result<[f64; 2], ParseError> {
        Ok([self.x_value, self.y_pos(variables)?])
    }

    fn generate_naive_map(&self) -> HashMap<String, f64> {
        let mut map = HashMap::new();
        for var in &self.variables {
            map.insert(var.clone(), self.x_value);
        }
        map
    }

    fn parse(input: &str) -> Result<Self, ParseError> {
        let function_match = Regex::new(r"(?P<FunctionName>\w+)\((?P<FunctionVariables>(?:[a-z]+,?)+)\)=(?P<Expression>[a-z01-9^*/()+-]+)").expect("Regex should compile");
        let captures = function_match
            .captures(input)
            .ok_or(ParseError::UnableToParse)?;

        let function_name = captures
            .name("FunctionName")
            .ok_or(ParseError::UnableToFind("function name".to_string()))?
            .as_str();
        let function_variables = captures
            .name("FunctionVariables")
            .ok_or(ParseError::UnableToFind("function variables".to_string()))?
            .as_str()
            .split(',')
            .map(String::from)
            .collect_vec();

        let ex = captures
            .name("Expression")
            .ok_or(ParseError::UnableToFind("function expression".to_string()))?
            .as_str()
            .to_string();
        let test_ex = TokenQueue::new(&ex, &function_variables)?;
        dbg!(&test_ex);

        Ok(Function {
            name: function_name.to_string(),
            x_value: 0.0,
            internal_offset: 0.0,
            test_ex,
            variables: function_variables,
        })
    }

    pub fn plot_points(&mut self, min_x: f64, max_x: f64) -> Result<PlotPoints, ParseError> {
        let mut points = Vec::new();

        self.internal_offset = (max_x - min_x) / 2000.;
        self.x_value = min_x + self.internal_offset;
        while self.x_value < max_x {
            self.next();
            points.push(self.current_point(&self.generate_naive_map())?);
        }
        Ok(PlotPoints::from(points))
    }
}

impl Iterator for Function {
    type Item = Result<(f64, f64), ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.x_value += self.internal_offset;
        match self.y_pos(&self.generate_naive_map()) {
            Ok(y_pos) => Some(Ok((self.x_value, y_pos))),
            Err(e) => Some(Err(e)),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::parse::math_functions::ParseError;
    use crate::parse::Function;
    use std::collections::HashMap;

    #[test]
    fn parse_function() {
        let test_fn = "f(t,b)=2t+5b";
        assert!(dbg!(Function::try_from(test_fn)).is_ok());
    }

    #[test]
    fn math_function_calculate() {
        let input = "f(test)=2test";
        dbg!(Function::try_from(input));
    }

    #[test]
    fn unused_variables() {
        let test_fn = "f(t,b,c)=2t+5b";
        assert_eq!(
            Function::try_from(test_fn),
            Err(ParseError::VariableDefinitionAndUseMismatch)
        );
    }
}
