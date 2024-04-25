use egui_plot::PlotPoints;

pub struct Function{
    rate_of_change: f64,
    y_offset: f64,
    x_value: f64,
    internal_offset: f64
}

struct VariableWithModifiers {
    var_name: String,
    multiplier: f64,
    exponent: f64,
    variable_value: f64,
    combination: Option<Box<dyn Calculate>>
}


trait Calculate {
    fn calculate(&self) -> f64;
}

impl Calculate for VariableWithModifiers {
    fn calculate(&self) -> f64 {
        self.variable_value.powf(self.exponent) * self.multiplier
    }
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
    #[test]
    fn test() {
        let f = super::Function{rate_of_change: 1., y_offset: 0., x_value: 0.};
        let mut iter = f.into_iter();
        assert_eq!(iter.next(), Some((0.000000001,0.000000001)));
        assert_eq!(iter.next(), Some((0.000000001*2.,0.000000001*2.)));
    }
}