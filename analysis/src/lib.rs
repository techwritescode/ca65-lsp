mod arena;

use codespan::Span;
use parser::{Line, LineKind, };
use std::collections::HashMap;

pub struct ScopeAnalyzer {
    lines: Vec<Line>,
    symtab: HashMap<String, Span>,
}

impl ScopeAnalyzer {
    pub fn new(lines: Vec<Line>) -> ScopeAnalyzer {
        ScopeAnalyzer {
            lines,
            symtab: HashMap::new(),
        }
    }

    pub fn parse(mut self) -> HashMap<String, Span> {
        for line in self.lines.iter() {
            match &line.kind {
                LineKind::Label(label) => {
                    self.symtab.insert(label.lexeme.clone(), label.span);
                }
                _ => {}
            }
        }
        
        self.symtab
    }
}

#[cfg(test)]
mod tests {
    use codespan::Span;
    use parser::Instructions;
    use parser::{
        Expression, ExpressionKind, Instruction, Line, LineKind, Parser, Token, TokenType,
        Tokenizer,
    };

    #[test]
    fn it_works() {
        let instructions = Instructions::load();
        let tokens = Tokenizer::new(&"adc a + b".to_string(), &instructions)
            .parse()
            .unwrap();

        let ast = Parser::new(&tokens).parse();
        assert_eq!(
            ast,
            vec![Line {
                kind: LineKind::Instruction(Instruction {
                    mnemonic: "adc".to_owned(),
                    parameters: vec![Expression {
                        kind: ExpressionKind::SimpleExpression(
                            Token::new(TokenType::Plus, "+".to_string(), 6),
                            Box::from(Expression {
                                kind: ExpressionKind::Literal("a".to_owned(),),
                                span: Span::new(4, 5)
                            }),
                            Box::from(Expression {
                                kind: ExpressionKind::Literal("b".to_owned(),),
                                span: Span::new(8, 9)
                            }),
                        ),
                        span: Span::new(4, 9)
                    },],
                },),
                span: Span::new(0, 9),
            }]
        )
    }
}
