use std::str::FromStr;

use crate::SolAST;
use rand::seq::SliceRandom;
use rand_pcg::*;

/// Every kind of mutation implements this trait.
pub trait Mutation {
    fn is_mutation_point(&self, node: &SolAST) -> bool;
    fn mutate_randomly(&self, node: &SolAST, source: &[u8], rand: &mut Pcg64) -> String;
}

/// Kinds of mutations.
// Note: did not port Unchecked Block mutation from Gambit1.0 as feedback indicated that it was not too useful.
#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub enum MutationType {
    BinaryOpMutation,
    RequireMutation,
    AssignmentMutation,
    DeleteExpressionMutation,
    FunctionCallMutation,
    IfStatementMutation,
    // IntegerMutation,
    SwapArgumentsFunctionMutation,
    SwapArgumentsOperatorMutation,
    SwapLinesMutation,
    UnaryOperatorMutation,
}

impl FromStr for MutationType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "BinaryOpMutation" => Ok(MutationType::BinaryOpMutation),
            "RequireMutation" => Ok(MutationType::RequireMutation),
            "AssignmentMutation" => Ok(MutationType::AssignmentMutation),
            "DeleteExpressionMutation" => Ok(MutationType::DeleteExpressionMutation),
            "FunctionCallMutation" => Ok(MutationType::FunctionCallMutation),
            "IfStatementMutation" => Ok(MutationType::IfStatementMutation),
            //"IntegerMutation" => Ok(MutationType::IntegerMutation),
            "SwapArgumentsFunctionMutation" => Ok(MutationType::SwapArgumentsFunctionMutation),
            "SwapArgumentsOperatorMutation" => Ok(MutationType::SwapArgumentsOperatorMutation),
            "SwapLinesMutation" => Ok(MutationType::SwapLinesMutation),
            "UnaryOperatorMutation" => Ok(MutationType::UnaryOperatorMutation),
            _ => panic!("Undefined mutant!"),
        }
    }
}

impl Mutation for MutationType {
    fn is_mutation_point(&self, node: &SolAST) -> bool {
        match self {
            MutationType::BinaryOpMutation => {
                if let Some(n) = node.node_type() {
                    return n == "BinaryOperation";
                }
            }
            MutationType::RequireMutation => {
                return node.node_type().map_or_else(
                    || false,
                    |n| {
                        n == "FunctionCall"
                            && (node
                                .expression()
                                .name()
                                .map_or_else(|| false, |nm| nm == "require"))
                            && !node.arguments().is_empty()
                    },
                );
            }
            MutationType::AssignmentMutation => {
                if let Some(n) = node.node_type() {
                    return n == "Assignment";
                }
            }
            MutationType::DeleteExpressionMutation => {
                if let Some(n) = node.node_type() {
                    return n == "ExpressionStatement";
                }
            }
            MutationType::FunctionCallMutation => {
                if let Some(n) = node.node_type() {
                    return n == "FunctionCall" && !node.arguments().is_empty();
                }
            }
            MutationType::IfStatementMutation => {
                if let Some(n) = node.node_type() {
                    return n == "IfStatement";
                }
            }
            MutationType::SwapArgumentsFunctionMutation => {
                if let Some(n) = node.node_type() {
                    return n == "FunctionCall" && node.arguments().len() > 1;
                }
            }
            MutationType::SwapArgumentsOperatorMutation => {
                let non_comm_ops = vec!["-", "/", "%", "**", ">", "<", ">=", "<=", "<<", ">>"];
                if let Some(n) = node.node_type() {
                    let op = node
                        .operator()
                        .unwrap_or_else(|| panic!("Binary operator must have an operator!"));
                    return n == "BinaryOperation" && non_comm_ops.contains(&op.as_str());
                }
            }
            MutationType::SwapLinesMutation => {
                if let Some(n) = node.node_type() {
                    return n == "Block" && node.statements().len() > 1;
                }
            }
            MutationType::UnaryOperatorMutation => {
                if let Some(n) = node.node_type() {
                    return n == "UnaryOperation";
                }
            }
        }
        false
    }

    fn mutate_randomly(&self, node: &SolAST, source: &[u8], rand: &mut Pcg64) -> String {
        match self {
            MutationType::BinaryOpMutation => {
                assert!(&self.is_mutation_point(node));
                let ops = vec!["+", "-", "*", "/", "%", "**"];
                let (_, endl) = node.left_expression().get_bounds();
                let (startr, _) = node.right_expression().get_bounds();
                node.replace_part(
                    source,
                    " ".to_string() + ops.choose(rand).unwrap() + " ",
                    endl,
                    startr,
                )
            }
            MutationType::RequireMutation => {
                assert!(&self.is_mutation_point(node));
                let arg = &node.arguments()[0];
                arg.replace_in_source(source, "!(".to_string() + &arg.get_text(source) + ")")
            }
            MutationType::DeleteExpressionMutation => {
                assert!(&self.is_mutation_point(node));
                node.comment_out(source)
            }
            MutationType::FunctionCallMutation => {
                assert!(&self.is_mutation_point(node));
                if let Some(arg) = node.arguments().choose(rand) {
                    node.replace_in_source(source, arg.get_text(source))
                } else {
                    node.get_text(source)
                }
            }
            MutationType::IfStatementMutation => {
                assert!(&self.is_mutation_point(node));
                let cond = node.condition();
                let bs = vec![true, false];
                if *bs.choose(rand).unwrap() {
                    cond.replace_in_source(source, (*bs.choose(rand).unwrap()).to_string())
                } else {
                    cond.replace_in_source(source, "!(".to_owned() + &cond.get_text(source) + ")")
                }
            }
            MutationType::SwapArgumentsFunctionMutation => {
                assert!(&self.is_mutation_point(node));
                let mut children = node.arguments();
                children.shuffle(rand);
                if children.len() == 2 {
                    node.replace_multiple(
                        source,
                        vec![
                            (children[0].clone(), children[1].get_text(source)),
                            (children[1].clone(), children[0].get_text(source)),
                        ],
                    )
                } else {
                    node.get_text(source)
                }
            }
            MutationType::SwapArgumentsOperatorMutation => {
                assert!(&self.is_mutation_point(node));
                let left = node.left_expression();
                let right = node.right_expression();
                node.replace_multiple(
                    source,
                    vec![
                        (left.clone(), right.get_text(source)),
                        (right, left.get_text(source)),
                    ],
                )
            }
            MutationType::SwapLinesMutation => {
                assert!(&self.is_mutation_point(node));
                let mut stmts = node.statements();
                stmts.shuffle(rand);
                if stmts.len() == 2 {
                    node.replace_multiple(
                        source,
                        vec![
                            (stmts[0].clone(), stmts[1].get_text(source)),
                            (stmts[1].clone(), stmts[0].get_text(source)),
                        ],
                    )
                } else {
                    node.get_text(source)
                }
            }
            MutationType::UnaryOperatorMutation => {
                let prefix_ops = vec!["++", "--", "~"];
                let suffix_ops = vec!["++", "--"];
                let is_prefix =
                    |source: &[u8], op: &String| -> bool { return source[0] == op.as_bytes()[0] };
                assert!(&self.is_mutation_point(node));
                let (start, end) = node.get_bounds();
                let op = node
                    .operator()
                    .expect("Unary operation must have an operator!");
                return if is_prefix(source, &op) {
                    node.replace_part(
                        source,
                        prefix_ops.choose(rand).unwrap().to_string(),
                        start,
                        start + op.len(),
                    )
                } else {
                    node.replace_part(
                        source,
                        suffix_ops.choose(rand).unwrap().to_string(),
                        end - op.len(),
                        end,
                    )
                };
            }
            MutationType::AssignmentMutation => todo!(),
        }
    }
}
