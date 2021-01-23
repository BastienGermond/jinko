//! ShuntingYard parses operators and operands according to operator precedence,
//! returning a BinaryOp in the end

use crate::instruction::{BinaryOp, Instruction, Operator};
use crate::utils::{Queue, Stack};

use super::box_construct::BoxConstruct;
use super::constructs::Construct;
use super::tokens::Token;

use nom::{branch::alt, Err, IResult};

pub struct ShuntingYard {
    operators: Stack<Operator>,
    output: Queue<Box<dyn Instruction>>,
}

impl ShuntingYard {
    // FIXME: Ugly to take input as parameter just for the lifetime
    fn reduce_output<'i>(&mut self, _: &'i str) -> IResult<&'i str, ()> {
        // FIXME: Cleanup
        // FIXME: Order, lhs should be before rhs
        let rhs = match self.output.pop() {
            Some(rhs) => rhs,
            None => {
                return Err(nom::Err::Error((
                    "Invalid binary expression",
                    nom::error::ErrorKind::OneOf,
                )))
            }
        };

        let lhs = match self.output.pop() {
            Some(lhs) => lhs,
            None => {
                return Err(nom::Err::Error((
                    "Invalid binary expression",
                    nom::error::ErrorKind::OneOf,
                )))
            }
        };

        let op = match self.operators.pop() {
            Some(op) => op,
            None => {
                return Err(nom::Err::Error((
                    "Invalid binary expression",
                    nom::error::ErrorKind::OneOf,
                )))
            }
        };

        self.output.push(Box::new(BinaryOp::new(lhs, rhs, op)));

        Ok(("", ()))
    }

    fn operator<'i>(&mut self, input: &'i str) -> IResult<&'i str, ()> {
        let (input, _) = Token::maybe_consume_extra(input)?;

        let (input, op) = alt((
            Token::add,
            Token::sub,
            Token::mul,
            Token::div,
            Token::left_parenthesis,
            Token::right_parenthesis,
        ))(input)?;

        let (input, _) = Token::maybe_consume_extra(input)?;

        let op = Operator::new(op);

        // We can unwrap since we check that the stack is not empty
        if op != Operator::LeftParenthesis && op != Operator::RightParenthesis {
            while !self.operators.is_empty()
            // FIXME: Cleanup
            && (self.operators.peek().unwrap().precedence() > op.precedence()
            || (self.operators.peek().unwrap().precedence() == op.precedence() && op.is_left_associative()))
            {
                self.reduce_output(input)?;
            }
            self.operators.push(op)
        } else if op == Operator::LeftParenthesis {
            self.operators.push(op);
        } else if op == Operator::RightParenthesis {
            while self.operators.peek() != Some(&Operator::LeftParenthesis) {
                self.reduce_output(input)?;
            }

            match self.operators.peek() {
                Some(&Operator::LeftParenthesis) => self.operators.pop(),
                _ => {
                    return Err(nom::Err::Error((
                        "Unclosed right parenthesis",
                        nom::error::ErrorKind::OneOf,
                    )))
                }
            };
        }

        Ok((input, ()))
    }

    fn operand<'i>(&mut self, input: &'i str) -> IResult<&'i str, ()> {
        let (input, expr) = alt((
            BoxConstruct::function_call,
            Construct::constant,
            BoxConstruct::variable,
        ))(input)?;

        self.output.push(expr);

        Ok((input, ()))
    }

    fn handle_token<'i>(&mut self, input: &'i str) -> IResult<&'i str, ()> {
        let (input, _) = Token::maybe_consume_extra(input)?;

        let (input, _) = match input.chars().next() {
            None => {
                return Err(Err::Error((
                    "Not a valid binary expression",
                    nom::error::ErrorKind::OneOf,
                )))
            }
            Some(c) => match Token::is_operator(c) {
                true => self.operator(input)?,
                false => self.operand(input)?,
            },
        };

        let (input, _) = Token::maybe_consume_extra(input)?;

        Ok((input, ()))
    }

    /// Create a new, empty ShuntingYard parser
    fn new() -> ShuntingYard {
        ShuntingYard {
            operators: Stack::new(),
            output: Queue::new(),
        }
    }

    /// Create a BinaryOp from an input string, executing the shunting yard
    /// algorithm
    pub fn parse(i: &str) -> IResult<&str, Box<dyn Instruction>> {
        let mut sy = ShuntingYard::new();

        let mut input = i.clone();

        match sy.handle_token(input) {
            Err(nom::Err::Error(_)) => {
                return Err(Err::Error((
                    "Not a valid binary expression",
                    nom::error::ErrorKind::Many1,
                )))
            }
            Err(e) => return Err(e),
            Ok((new_i, _)) => {
                input = new_i;

                loop {
                    match sy.handle_token(input) {
                        // FIXME: Maybe don't use OneOf as error type?
                        Err(nom::Err::Error((_, nom::error::ErrorKind::OneOf))) => break,
                        Err(e) => return Err(e),
                        Ok((new_i, _)) => {
                            if new_i == input {
                                break;
                            }

                            input = new_i;
                        }
                    }
                }
            }
        }

        // We are done, pop everything from the different stacks
        while !sy.operators.is_empty() {
            sy.reduce_output(input)?;
        }

        match sy.output.pop() {
            Some(binop) => Ok((input, binop)),
            _ => Err(nom::Err::Error((
                "Invalid binary expression",
                nom::error::ErrorKind::OneOf,
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::*;

    #[test]
    fn t_sy_valid_add() {
        let boxed_output = ShuntingYard::parse("1 + 2").unwrap().1;
        let output = boxed_output.downcast_ref::<BinaryOp>().unwrap();
        let reference = BinaryOp::new(
            Box::new(JinkInt::from(1)),
            Box::new(JinkInt::from(2)),
            Operator::Add,
        );

        assert_eq!(output.operator(), reference.operator());
    }

    #[test]
    fn t_sy_valid_mul() {
        let boxed_output = ShuntingYard::parse("1 * 2").unwrap().1;
        let output = boxed_output.downcast_ref::<BinaryOp>().unwrap();
        let reference = BinaryOp::new(
            Box::new(JinkInt::from(1)),
            Box::new(JinkInt::from(2)),
            Operator::Mul,
        );

        assert_eq!(output.operator(), reference.operator());
    }

    #[test]
    fn t_sy_valid_normal_priority() {
        let boxed_output = ShuntingYard::parse("1 * 2 + 3").unwrap().1;
        let output = boxed_output.downcast_ref::<BinaryOp>().unwrap();
        let l_ref = BinaryOp::new(
            Box::new(JinkInt::from(1)),
            Box::new(JinkInt::from(2)),
            Operator::Mul,
        );
        let reference = BinaryOp::new(Box::new(l_ref), Box::new(JinkInt::from(3)), Operator::Add);

        assert_eq!(output.operator(), reference.operator());
    }

    #[test]
    fn t_sy_valid_back_priority() {
        let boxed_output = ShuntingYard::parse("3 + 1 * 2").unwrap().1;
        let output = boxed_output.downcast_ref::<BinaryOp>().unwrap();
        let l_ref = BinaryOp::new(
            Box::new(JinkInt::from(1)),
            Box::new(JinkInt::from(2)),
            Operator::Mul,
        );
        let reference = BinaryOp::new(Box::new(l_ref), Box::new(JinkInt::from(3)), Operator::Add);

        assert_eq!(output.operator(), reference.operator());
    }

    #[test]
    fn t_sy_valid_parentheses_priority() {
        let boxed_output = ShuntingYard::parse("(3 + 1) * 2").unwrap().1;
        let output = boxed_output.downcast_ref::<BinaryOp>().unwrap();
        let l_ref = BinaryOp::new(
            Box::new(JinkInt::from(1)),
            Box::new(JinkInt::from(3)),
            Operator::Add,
        );
        let reference = BinaryOp::new(Box::new(l_ref), Box::new(JinkInt::from(2)), Operator::Mul);

        assert_eq!(output.operator(), reference.operator());
    }

    #[test]
    fn t_sy_valid_parentheses_priority_reverse() {
        let boxed_output = ShuntingYard::parse("2 * (3 + 1)").unwrap().1;
        let output = boxed_output.downcast_ref::<BinaryOp>().unwrap();
        let l_ref = BinaryOp::new(
            Box::new(JinkInt::from(1)),
            Box::new(JinkInt::from(3)),
            Operator::Add,
        );
        let reference = BinaryOp::new(Box::new(l_ref), Box::new(JinkInt::from(2)), Operator::Mul);

        assert_eq!(output.operator(), reference.operator());
    }

    // FIXME: Add more tests with more operators

    fn sy_assert_l(input: &str, result: i64) {
        use crate::instance::ToInstance;
        use crate::{InstrKind, Interpreter};

        let boxed_output = ShuntingYard::parse(input).unwrap().1;
        let output = boxed_output.downcast_ref::<BinaryOp>().unwrap();

        let mut i = Interpreter::new();

        assert_eq!(
            output.lhs().execute(&mut i).unwrap(),
            InstrKind::Expression(Some(JinkInt::from(result).to_instance()))
        );
    }

    #[test]
    fn t_sy_execute_natural_order() {
        sy_assert_l("4 + 7 + 3", 11);
    }

    // FIXME: Don't ignore once ShuntingYard is fixed

    #[test]
    #[ignore]
    fn t_sy_execute_mult_priority() {
        sy_assert_l("4 + 2 * 3", 6);
    }

    #[test]
    #[ignore]
    fn t_sy_execute_mult_natural_priority() {
        sy_assert_l("2 * 3 + 4", 6);
    }
}
