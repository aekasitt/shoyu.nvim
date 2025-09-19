/* ~~/src/syntax.rs */

use crate::themes::{Theme, ThemeColor};
use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, ThemeSet};
use syntect::parsing::{SyntaxReference, SyntaxSet};
use syntect::util::LinesWithEndings;

pub struct SyntaxHighlighter {
  syntax_set: SyntaxSet,
  theme_set: ThemeSet,
}

#[derive(Debug, Clone)]
pub struct HighlightedLine {
  pub tokens: Vec<HighlightedToken>,
}

#[derive(Debug, Clone)]
pub struct HighlightedToken {
  pub text: String,
  pub color: ThemeColor,
}

impl SyntaxHighlighter {
  pub fn new() -> Self {
    let syntax_set = SyntaxSet::load_defaults_newlines();
    let theme_set = ThemeSet::load_defaults();

    Self {
      syntax_set,
      theme_set,
    }
  }

  pub fn highlight_code(&self, code: &str, language: &str, theme: &Theme) -> Vec<HighlightedLine> {
    // Try to use syntect for advanced highlighting
    if let Some(syntax) = self.find_syntax_by_language(language) {
      if let Some(syntect_theme) = self.theme_set.themes.get("base16-ocean.dark") {
        return self.highlight_with_syntect(code, syntax, syntect_theme, theme);
      }
    }

    // Fallback to pattern-based highlighting
    self.highlight_with_patterns(code, theme)
  }

  fn find_syntax_by_language(&self, language: &str) -> Option<&SyntaxReference> {
    let lang = language.to_lowercase();

    // Map common language names to syntect syntax names
    let syntax_name = match lang.as_str() {
      "js" | "javascript" => "JavaScript",
      "ts" | "typescript" => "TypeScript",
      "py" | "python" => "Python",
      "rs" | "rust" => "Rust",
      "go" => "Go",
      "java" => "Java",
      "c" => "C",
      "cpp" | "c++" => "C++",
      "html" => "HTML",
      "css" => "CSS",
      "json" => "JSON",
      "yaml" | "yml" => "YAML",
      "xml" => "XML",
      "md" | "markdown" => "Markdown",
      "bash" | "shell" | "sh" => "Bourne Again Shell (bash)",
      "sql" => "SQL",
      "php" => "PHP",
      "ruby" | "rb" => "Ruby",
      "swift" => "Swift",
      "kotlin" | "kt" => "Kotlin",
      "scala" => "Scala",
      "lua" => "Lua",
      "vim" => "VimL",
      _ => return self.syntax_set.find_syntax_by_extension(&lang),
    };

    self
      .syntax_set
      .find_syntax_by_name(syntax_name)
      .or_else(|| self.syntax_set.find_syntax_by_extension(&lang))
  }

  fn highlight_with_syntect(
    &self,
    code: &str,
    syntax: &SyntaxReference,
    syntect_theme: &syntect::highlighting::Theme,
    theme: &Theme,
  ) -> Vec<HighlightedLine> {
    let mut lines = Vec::new();
    let mut highlighter = HighlightLines::new(syntax, syntect_theme);

    for line in LinesWithEndings::from(code) {
      let ranges = highlighter
        .highlight_line(line, &self.syntax_set)
        .unwrap_or_default();
      let mut tokens = Vec::new();

      for (style, text) in ranges {
        let color = self.convert_syntect_style_to_theme_color(style, theme);
        // Strip newline characters to prevent tofu glyphs
        let clean_text = text.replace('\n', "").replace('\r', "");
        if !clean_text.is_empty() {
          tokens.push(HighlightedToken {
            text: clean_text,
            color,
          });
        }
      }

      lines.push(HighlightedLine { tokens });
    }

    lines
  }

  fn convert_syntect_style_to_theme_color(&self, style: Style, theme: &Theme) -> ThemeColor {
    // Map syntect colors to our theme colors based on style properties
    let fg = style.foreground;

    // Create a hex color from the syntect color
    let hex = format!("#{:02x}{:02x}{:02x}", fg.r, fg.g, fg.b);

    // Try to match the color to appropriate theme colors
    // This is a simplified mapping - you could make it more sophisticated
    if style
      .font_style
      .contains(syntect::highlighting::FontStyle::BOLD)
    {
      theme.keyword.clone()
    } else {
      ThemeColor::new(&hex)
    }
  }

  fn highlight_with_patterns(&self, code: &str, theme: &Theme) -> Vec<HighlightedLine> {
    let mut lines = Vec::new();

    for line in code.lines() {
      let highlighted_line = self.highlight_line(line, theme);
      lines.push(highlighted_line);
    }

    lines
  }

  fn highlight_line(&self, line: &str, theme: &Theme) -> HighlightedLine {
    let mut tokens = Vec::new();
    let _current_pos = 0;

    // Simple tokenization by splitting on whitespace and common delimiters
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;

    while i < chars.len() {
      let start = i;

      // Skip whitespace
      while i < chars.len() && chars[i].is_whitespace() {
        i += 1;
      }

      if i > start {
        tokens.push(HighlightedToken {
          text: chars[start..i].iter().collect(),
          color: theme.foreground.clone(),
        });
        continue;
      }

      // Handle strings
      if i < chars.len() && (chars[i] == '"' || chars[i] == '\'') {
        let quote = chars[i];
        i += 1;
        let string_start = start;

        while i < chars.len() && chars[i] != quote {
          if chars[i] == '\\' && i + 1 < chars.len() {
            i += 2; // Skip escaped character
          } else {
            i += 1;
          }
        }

        if i < chars.len() {
          i += 1; // Include closing quote
        }

        tokens.push(HighlightedToken {
          text: chars[string_start..i].iter().collect(),
          color: theme.string.clone(),
        });
        continue;
      }

      // Handle comments
      if i + 1 < chars.len() && chars[i] == '/' && chars[i + 1] == '/' {
        tokens.push(HighlightedToken {
          text: chars[i..].iter().collect(),
          color: theme.comment.clone(),
        });
        break;
      }

      // Handle other tokens (words, numbers, operators)
      let token_start = i;
      while i < chars.len()
        && !chars[i].is_whitespace()
        && !matches!(
          chars[i],
          '"' | '\'' | '(' | ')' | '{' | '}' | '[' | ']' | ';' | ',' | '.'
        )
      {
        i += 1;
      }

      if i > token_start {
        let token_text: String = chars[token_start..i].iter().collect();
        let color = self.determine_color_by_pattern(&token_text, theme);

        tokens.push(HighlightedToken {
          text: token_text,
          color,
        });
      } else if i < chars.len() {
        // Single character tokens
        tokens.push(HighlightedToken {
          text: chars[i].to_string(),
          color: theme.punctuation.clone(),
        });
        i += 1;
      }
    }

    HighlightedLine { tokens }
  }

  fn determine_color_by_pattern(&self, text: &str, theme: &Theme) -> ThemeColor {
    // Pattern-based highlighting
    if text.parse::<f64>().is_ok() {
      theme.number.clone()
    } else if is_keyword(text) {
      theme.keyword.clone()
    } else if is_type(text) {
      theme.type_color.clone()
    } else if text.chars().all(|c| c.is_uppercase() || c == '_') && text.len() > 1 {
      theme.constant.clone()
    } else {
      theme.foreground.clone()
    }
  }
}

fn is_keyword(text: &str) -> bool {
  matches!(
    text,
    "fn"
      | "let"
      | "mut"
      | "const"
      | "if"
      | "else"
      | "while"
      | "for"
      | "loop"
      | "match"
      | "return"
      | "break"
      | "continue"
      | "pub"
      | "mod"
      | "use"
      | "struct"
      | "enum"
      | "trait"
      | "impl"
      | "where"
      | "async"
      | "await"
      | "move"
      | "static"
      | "function"
      | "var"
      | "class"
      | "import"
      | "export"
      | "from"
      | "as"
      | "default"
      | "def"
      | "lambda"
      | "pass"
      | "with"
      | "try"
      | "except"
      | "finally"
      | "raise"
      | "and"
      | "or"
      | "not"
      | "is"
      | "in"
      | "True"
      | "False"
      | "None"
      | "null"
      | "undefined"
  )
}

fn is_type(text: &str) -> bool {
  matches!(
    text,
    "i8"
      | "i16"
      | "i32"
      | "i64"
      | "u8"
      | "u16"
      | "u32"
      | "u64"
      | "f32"
      | "f64"
      | "bool"
      | "char"
      | "str"
      | "String"
      | "Vec"
      | "Option"
      | "Result"
      | "Box"
      | "int"
      | "float"
      | "string"
      | "boolean"
      | "object"
      | "array"
  )
}

pub fn is_language_supported(language: &str) -> bool {
  matches!(
    language.to_lowercase().as_str(),
    "javascript"
      | "js"
      | "typescript"
      | "ts"
      | "python"
      | "py"
      | "rust"
      | "rs"
      | "go"
      | "java"
      | "c"
      | "cpp"
      | "c++"
      | "html"
      | "css"
      | "json"
      | "yaml"
      | "yml"
      | "xml"
      | "markdown"
      | "md"
      | "bash"
      | "shell"
      | "sh"
      | "sql"
      | "php"
      | "ruby"
      | "rb"
      | "swift"
      | "kotlin"
      | "kt"
      | "scala"
      | "clojure"
      | "clj"
      | "haskell"
      | "hs"
      | "lua"
      | "vim"
      | "dockerfile"
      | "text"
      | "plain"
  )
}
