use markdown_it::{
    Node, NodeValue, Renderer,
    parser::inline::{InlineRule, InlineState},
};

use tylax::{DocumentWrapperMode, T2LOptions, typst_to_latex_with_options};

#[derive(Debug)]
struct MathEquation {
    equation: String,
    display_mode: bool,
}

impl NodeValue for MathEquation {
    fn render(&self, _node: &Node, fmt: &mut dyn Renderer) {
        let opts = katex::Opts::builder()
            .throw_on_error(false)
            .display_mode(self.display_mode)
            .build()
            .ok();
        let katex_equation = opts.and_then(|o| katex::render_with_opts(&self.equation, o).ok());
        fmt.text_raw(katex_equation.as_ref().unwrap_or(&self.equation))
    }
}

pub struct MathEquationScanner;

impl InlineRule for MathEquationScanner {
    const MARKER: char = '$';

    fn run(state: &mut InlineState) -> Option<(Node, usize)> {
        let input = &state.src[state.pos..state.pos_max];
        let math_marker = String::from(if input.starts_with("$$") { "$$" } else { "$" });
        let SEPARATOR_LENGTH: usize = math_marker.len();

        // True -> Block display
        // False -> Inline display
        let display_mode = input.starts_with(&(math_marker.clone() + "\n"))
            || input.starts_with(&(math_marker.clone() + " "));

        input[SEPARATOR_LENGTH..].find(&math_marker).map(|length| {
            let start = state.pos + SEPARATOR_LENGTH;
            let i = start + length;
            if start > i {
                return None;
            }
            let content = &state.src[start..i];
            let node = Node::new(MathEquation {
                //  equation: content.to_string(),
                equation: match math_marker.as_str() {
                    "$$" => content.to_string(), // LaTeX
                    "$" => {
                        // Typst
                        let tylax_opt: T2LOptions = T2LOptions {
                            full_document: false,
                            document_class: "article".to_string(),
                            title: None,
                            author: None,
                            math_only: true,
                            block_math_mode: true,
                            wrapper: DocumentWrapperMode::Default,
                        };
                        typst_to_latex_with_options(content, &tylax_opt)
                    }
                    _ => unreachable!(),
                },
                display_mode,
            });
            Some((node, i + SEPARATOR_LENGTH + state.pos))
        })?
    }
}

#[cfg(test)]
mod test {
    use crate::render_article_markdown;

    #[test]
    #[expect(clippy::unwrap_used)]
    fn test_markdown_equation_katex() {
        let rendered =
            render_article_markdown("here is a math equation: $$E=mc^2$$. Pretty cool, right?");
        assert_eq!(
            "<p>here is a math equation: ".to_owned()
                + &katex::render("E=mc^2").unwrap()
                + ". Pretty cool, right?</p>\n",
            rendered
        );
    }
}
