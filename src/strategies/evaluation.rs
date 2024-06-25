/// Strategy for evaluating a game state of type `StateType`.
pub trait EvalStrategy<StateType> {
    /// Evaluate a game state. The more positive the better.
    fn eval(&mut self, state: &StateType) -> f32;
    /// Return a weighted version of the evaluation function.
    fn weighted<U>(&self, weight: &U) -> Weighted<Self, U>
    where
        U: EvalStrategy<StateType> + Clone,
        Self: Sized + Clone,
    {
        Weighted {
            strategy: self.clone(),
            weight: weight.clone(),
        }
    }
    fn to_box(self) -> Box<dyn EvalStrategy<StateType>>
    where
        Self: EvalStrategy<StateType> + Sized + 'static,
    {
        Box::new(self)
    }
}

/// An `EvalStrategy` can just be a function.
impl<F, T> EvalStrategy<T> for F
where
    F: FnMut(&T) -> f32,
{
    fn eval(&mut self, state: &T) -> f32 {
        self(state)
    }
}

/// Represents the sum of multiple evaluation strategies.
pub struct Sum<T> {
    strategies: Vec<Box<dyn EvalStrategy<T>>>,
}

impl<T> EvalStrategy<T> for Sum<T> {
    fn eval(&mut self, state: &T) -> f32 {
        self.strategies.iter_mut().map(|x| x.eval(state)).sum()
    }
}

impl<T> Sum<T> {
    #[must_use]
    pub fn new(strategies: Vec<Box<dyn EvalStrategy<T>>>) -> Sum<T> {
        Sum { strategies }
    }
}

/// Sum several evaluation functions together.
#[macro_export]
macro_rules! sum {
    ( $( $x:expr ),* ) => {
        {
            let temp_vec = vec![$($x.to_box()),*];
            Sum::new(temp_vec)
        }
    };
}

/// Represents the product of multiple evaluation strategies.
pub struct Product<T> {
    strategies: Vec<Box<dyn EvalStrategy<T>>>,
}

impl<T> EvalStrategy<T> for Product<T> {
    fn eval(&mut self, state: &T) -> f32 {
        self.strategies.iter_mut().map(|x| x.eval(state)).product()
    }
}

impl<T> Product<T> {
    #[must_use]
    pub fn new(strategies: Vec<Box<dyn EvalStrategy<T>>>) -> Product<T> {
        Product { strategies }
    }
}

/// Multiply several evaluation functions together.
#[macro_export]
macro_rules! product {
    ( $( $x:expr ),* ) => {
        {
            let temp_vec = vec![$($x.to_box()),*];
            Product::new(temp_vec)
        }
    };
}

/// Represents a strategy with a weight.
///
/// Technically this can be implemented with `Product`,
/// but this happens often enough and it makes some sense to consider them separately.
pub struct Weighted<StrategyType, WeightType> {
    strategy: StrategyType,
    weight: WeightType,
}

impl<S, W, T> EvalStrategy<T> for Weighted<S, W>
where
    S: EvalStrategy<T>,
    W: EvalStrategy<T>,
{
    fn eval(&mut self, state: &T) -> f32 {
        self.strategy.eval(state) * self.weight.eval(state)
    }
}

#[cfg(test)]
mod tests {
    use approx::relative_eq;
    use assert_approx_eq::assert_approx_eq;

    use super::*;
    fn const_eval<S>(f: f32) -> impl Fn(&S) -> f32 + Clone {
        move |_: &S| f
    }
    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn id_eval(f: &f32) -> f32 {
        *f
    }
    #[test]
    fn test_sum() {
        let a = const_eval(0.0);
        let b = const_eval(1.0);
        let c = id_eval;
        let mut e = sum!(a, b, c);
        assert_approx_eq!(3.0, e.eval(&2.0));
    }
    #[test]
    fn test_product() {
        let const1 = const_eval(1.0);
        let const2 = const_eval(2.0);
        let const3 = const_eval(3.0);
        let const4 = const_eval(4.0);
        let mut e = product!(const1, const2, const3, const4);
        assert_approx_eq!(24.0, e.eval(&()));
    }
    #[test]
    fn test_weighted() {
        let a = const_eval(2.0);
        let mut w = a.weighted(&id_eval);
        let mut w2 = a.weighted(&a);
        assert!(relative_eq!(2.0, w.eval(&1.0)));
        assert!(relative_eq!(1.0, w.eval(&0.5)));
        assert!(relative_eq!(4.0, w2.eval(&0.0)));
    }
}
