use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CharacterNode {
    pub char: char,
    pub position: usize,
    pub transitions: Vec<usize>, // Indices to next characters
    pub frequency: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RegexPattern {
    pub pattern: String,
    pub matches: Vec<(usize, usize)>, // Start, end positions in char graph
    pub token_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LexerToken {
    pub token_type: String,
    pub value: String,
    pub char_range: (usize, usize),
    pub regex_pattern: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ParseNode {
    pub node_type: String,
    pub tokens: Vec<LexerToken>,
    pub children: Vec<ParseNode>,
    pub semantic_value: Option<String>,
}

pub struct CodeTransformationGraph {
    pub char_graph: Vec<CharacterNode>,
    pub regex_patterns: Vec<RegexPattern>,
    pub lexer_tokens: Vec<LexerToken>,
    pub parse_tree: Vec<ParseNode>,
}

impl CodeTransformationGraph {
    pub fn new() -> Self {
        Self {
            char_graph: Vec::new(),
            regex_patterns: Vec::new(),
            lexer_tokens: Vec::new(),
            parse_tree: Vec::new(),
        }
    }

    pub fn build_from_source(&mut self, source: &str) -> Result<(), String> {
        // 1. Build character graph
        self.build_character_graph(source);

        // 2. Apply regex patterns to find tokens
        self.apply_regex_patterns();

        // 3. Generate lexer tokens
        self.generate_lexer_tokens();

        // 4. Build parse tree
        self.build_parse_tree();

        Ok(())
    }

    fn build_character_graph(&mut self, source: &str) {
        let chars: Vec<char> = source.chars().collect();

        for (i, &ch) in chars.iter().enumerate() {
            let mut transitions = Vec::new();

            // Add transition to next character
            if i + 1 < chars.len() {
                transitions.push(i + 1);
            }

            self.char_graph.push(CharacterNode {
                char: ch,
                position: i,
                transitions,
                frequency: 1,
            });
        }
    }

    fn apply_regex_patterns(&mut self) {
        let patterns = vec![
            (r"\d+", "NUMBER"),
            (r"[a-zA-Z_][a-zA-Z0-9_]*", "IDENTIFIER"),
            (r#""[^"]*""#, "STRING"),
            (r"let|const|fn|struct", "KEYWORD"),
            (r"[+\-*/=]", "OPERATOR"),
            (r"[;,(){}]", "PUNCTUATION"),
        ];

        let source: String = self.char_graph.iter().map(|n| n.char).collect();

        for (pattern, token_type) in patterns {
            if let Ok(regex) = regex::Regex::new(pattern) {
                for mat in regex.find_iter(&source) {
                    self.regex_patterns.push(RegexPattern {
                        pattern: pattern.to_string(),
                        matches: vec![(mat.start(), mat.end())],
                        token_type: token_type.to_string(),
                    });
                }
            }
        }
    }

    fn generate_lexer_tokens(&mut self) {
        let source: String = self.char_graph.iter().map(|n| n.char).collect();

        for pattern in &self.regex_patterns {
            for &(start, end) in &pattern.matches {
                let value = source[start..end].to_string();

                self.lexer_tokens.push(LexerToken {
                    token_type: pattern.token_type.clone(),
                    value,
                    char_range: (start, end),
                    regex_pattern: pattern.pattern.clone(),
                });
            }
        }

        // Sort by position
        self.lexer_tokens.sort_by_key(|t| t.char_range.0);
    }

    fn build_parse_tree(&mut self) {
        // Simple expression parser
        let mut i = 0;
        while i < self.lexer_tokens.len() {
            if let Some(node) = self.parse_statement(&mut i) {
                self.parse_tree.push(node);
            }
        }
    }

    fn parse_statement(&self, i: &mut usize) -> Option<ParseNode> {
        if *i >= self.lexer_tokens.len() {
            return None;
        }

        let token = &self.lexer_tokens[*i];

        match token.token_type.as_str() {
            "KEYWORD" if token.value == "let" || token.value == "const" => {
                self.parse_variable_declaration(i)
            }
            _ => {
                *i += 1;
                Some(ParseNode {
                    node_type: "EXPRESSION".to_string(),
                    tokens: vec![token.clone()],
                    children: Vec::new(),
                    semantic_value: Some(token.value.clone()),
                })
            }
        }
    }

    fn parse_variable_declaration(&self, i: &mut usize) -> Option<ParseNode> {
        let mut tokens = Vec::new();
        let _start = *i;

        // Collect tokens until semicolon
        while *i < self.lexer_tokens.len() {
            tokens.push(self.lexer_tokens[*i].clone());
            *i += 1;

            if self.lexer_tokens.get(*i - 1)?.value == ";" {
                break;
            }
        }

        Some(ParseNode {
            node_type: "VARIABLE_DECLARATION".to_string(),
            tokens,
            children: Vec::new(),
            semantic_value: None,
        })
    }

    pub fn get_transformation_summary(&self) -> String {
        format!(
            "Transformation Graph:\n\
            📝 Characters: {}\n\
            🔍 Regex Patterns: {}\n\
            🏷️  Lexer Tokens: {}\n\
            🌳 Parse Nodes: {}",
            self.char_graph.len(),
            self.regex_patterns.len(),
            self.lexer_tokens.len(),
            self.parse_tree.len()
        )
    }
}
