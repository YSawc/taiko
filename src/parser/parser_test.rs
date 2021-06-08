#[cfg(test)]
mod test {
    use crate::parser::parser::*;
    use crate::util::annot::*;

    fn parse_expected_error(script: impl Into<String>, expected: ParseError) {
        let mut parser = Parser::new();
        let res = parser.parse_program(script.into()).unwrap_err();
        if res != expected.clone() {
            panic!("Expected:{:?} Got:{:?}", expected, res);
        }
    }

    #[test]
    fn literal_before_definition_error() {
        let program = "
            3 class Foo
            end
        ";
        let expected = ParseError::new(
            ParseErrorKind::LiteralBeforeDefinition,
            Loc::new(Loc(13, 13)),
        );
        parse_expected_error(program, expected);
    }

    #[test]
    fn inner_class_definition_in_method_definion_error() {
        let program = "
            def foo
              class bar
              end
            end
        ";
        let expected = ParseError::new(
            ParseErrorKind::InnerClassDefinitionInMethodDefinition,
            Loc::new(Loc(13, 15)),
        );
        parse_expected_error(program, expected);
    }
}
