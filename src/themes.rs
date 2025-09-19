/* ~~/src/themes.rs */

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeColor {
  pub hex: String,
  pub rgb: (u8, u8, u8),
}

impl ThemeColor {
  pub fn new(hex: &str) -> Self {
    let rgb = hex_to_rgb(hex).unwrap_or((255, 255, 255));
    Self {
      hex: hex.to_string(),
      rgb,
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
  pub name: String,
  pub background: ThemeColor,
  pub foreground: ThemeColor,
  pub comment: ThemeColor,
  pub keyword: ThemeColor,
  pub string: ThemeColor,
  pub number: ThemeColor,
  pub function: ThemeColor,
  pub type_color: ThemeColor,
  pub variable: ThemeColor,
  pub operator: ThemeColor,
  pub punctuation: ThemeColor,
  pub constant: ThemeColor,
  pub class: ThemeColor,
}

pub fn get_theme(name: &str) -> Option<Theme> {
  match name.to_lowercase().as_str() {
    "dracula" => Some(dracula_theme()),
    "monokai" => Some(monokai_theme()),
    "github" => Some(github_theme()),
    "nord" => Some(nord_theme()),
    "solarized-dark" => Some(solarized_dark_theme()),
    "solarized-light" => Some(solarized_light_theme()),
    "one-dark" => Some(one_dark_theme()),
    "gruvbox" => Some(gruvbox_theme()),
    _ => None,
  }
}

pub fn get_theme_names() -> Vec<String> {
  vec![
    "dracula".to_string(),
    "monokai".to_string(),
    "github".to_string(),
    "nord".to_string(),
    "solarized-dark".to_string(),
    "solarized-light".to_string(),
    "one-dark".to_string(),
    "gruvbox".to_string(),
  ]
}

fn dracula_theme() -> Theme {
  Theme {
    name: "Dracula".to_string(),
    background: ThemeColor::new("#282a36"),
    foreground: ThemeColor::new("#f8f8f2"),
    comment: ThemeColor::new("#6272a4"),
    keyword: ThemeColor::new("#ff79c6"),
    string: ThemeColor::new("#f1fa8c"),
    number: ThemeColor::new("#bd93f9"),
    function: ThemeColor::new("#50fa7b"),
    type_color: ThemeColor::new("#8be9fd"),
    variable: ThemeColor::new("#f8f8f2"),
    operator: ThemeColor::new("#ff79c6"),
    punctuation: ThemeColor::new("#f8f8f2"),
    constant: ThemeColor::new("#bd93f9"),
    class: ThemeColor::new("#8be9fd"),
  }
}

fn monokai_theme() -> Theme {
  Theme {
    name: "Monokai".to_string(),
    background: ThemeColor::new("#272822"),
    foreground: ThemeColor::new("#f8f8f2"),
    comment: ThemeColor::new("#75715e"),
    keyword: ThemeColor::new("#f92672"),
    string: ThemeColor::new("#e6db74"),
    number: ThemeColor::new("#ae81ff"),
    function: ThemeColor::new("#a6e22e"),
    type_color: ThemeColor::new("#66d9ef"),
    variable: ThemeColor::new("#f8f8f2"),
    operator: ThemeColor::new("#f92672"),
    punctuation: ThemeColor::new("#f8f8f2"),
    constant: ThemeColor::new("#ae81ff"),
    class: ThemeColor::new("#a6e22e"),
  }
}

fn github_theme() -> Theme {
  Theme {
    name: "GitHub".to_string(),
    background: ThemeColor::new("#ffffff"),
    foreground: ThemeColor::new("#24292e"),
    comment: ThemeColor::new("#6a737d"),
    keyword: ThemeColor::new("#d73a49"),
    string: ThemeColor::new("#032f62"),
    number: ThemeColor::new("#005cc5"),
    function: ThemeColor::new("#6f42c1"),
    type_color: ThemeColor::new("#005cc5"),
    variable: ThemeColor::new("#24292e"),
    operator: ThemeColor::new("#d73a49"),
    punctuation: ThemeColor::new("#24292e"),
    constant: ThemeColor::new("#005cc5"),
    class: ThemeColor::new("#6f42c1"),
  }
}

fn nord_theme() -> Theme {
  Theme {
    name: "Nord".to_string(),
    background: ThemeColor::new("#2e3440"),
    foreground: ThemeColor::new("#d8dee9"),
    comment: ThemeColor::new("#616e88"),
    keyword: ThemeColor::new("#81a1c1"),
    string: ThemeColor::new("#a3be8c"),
    number: ThemeColor::new("#b48ead"),
    function: ThemeColor::new("#88c0d0"),
    type_color: ThemeColor::new("#81a1c1"),
    variable: ThemeColor::new("#d8dee9"),
    operator: ThemeColor::new("#81a1c1"),
    punctuation: ThemeColor::new("#eceff4"),
    constant: ThemeColor::new("#b48ead"),
    class: ThemeColor::new("#8fbcbb"),
  }
}

fn solarized_dark_theme() -> Theme {
  Theme {
    name: "Solarized Dark".to_string(),
    background: ThemeColor::new("#002b36"),
    foreground: ThemeColor::new("#839496"),
    comment: ThemeColor::new("#586e75"),
    keyword: ThemeColor::new("#859900"),
    string: ThemeColor::new("#2aa198"),
    number: ThemeColor::new("#d33682"),
    function: ThemeColor::new("#268bd2"),
    type_color: ThemeColor::new("#b58900"),
    variable: ThemeColor::new("#839496"),
    operator: ThemeColor::new("#859900"),
    punctuation: ThemeColor::new("#93a1a1"),
    constant: ThemeColor::new("#cb4b16"),
    class: ThemeColor::new("#b58900"),
  }
}

fn solarized_light_theme() -> Theme {
  Theme {
    name: "Solarized Light".to_string(),
    background: ThemeColor::new("#fdf6e3"),
    foreground: ThemeColor::new("#657b83"),
    comment: ThemeColor::new("#93a1a1"),
    keyword: ThemeColor::new("#859900"),
    string: ThemeColor::new("#2aa198"),
    number: ThemeColor::new("#d33682"),
    function: ThemeColor::new("#268bd2"),
    type_color: ThemeColor::new("#b58900"),
    variable: ThemeColor::new("#657b83"),
    operator: ThemeColor::new("#859900"),
    punctuation: ThemeColor::new("#586e75"),
    constant: ThemeColor::new("#cb4b16"),
    class: ThemeColor::new("#b58900"),
  }
}

fn one_dark_theme() -> Theme {
  Theme {
    name: "One Dark".to_string(),
    background: ThemeColor::new("#282c34"),
    foreground: ThemeColor::new("#abb2bf"),
    comment: ThemeColor::new("#5c6370"),
    keyword: ThemeColor::new("#c678dd"),
    string: ThemeColor::new("#98c379"),
    number: ThemeColor::new("#d19a66"),
    function: ThemeColor::new("#61afef"),
    type_color: ThemeColor::new("#e06c75"),
    variable: ThemeColor::new("#abb2bf"),
    operator: ThemeColor::new("#c678dd"),
    punctuation: ThemeColor::new("#abb2bf"),
    constant: ThemeColor::new("#d19a66"),
    class: ThemeColor::new("#e5c07b"),
  }
}

fn gruvbox_theme() -> Theme {
  Theme {
    name: "Gruvbox".to_string(),
    background: ThemeColor::new("#282828"),
    foreground: ThemeColor::new("#ebdbb2"),
    comment: ThemeColor::new("#928374"),
    keyword: ThemeColor::new("#fb4934"),
    string: ThemeColor::new("#b8bb26"),
    number: ThemeColor::new("#d3869b"),
    function: ThemeColor::new("#fabd2f"),
    type_color: ThemeColor::new("#fe8019"),
    variable: ThemeColor::new("#ebdbb2"),
    operator: ThemeColor::new("#fb4934"),
    punctuation: ThemeColor::new("#ebdbb2"),
    constant: ThemeColor::new("#d3869b"),
    class: ThemeColor::new("#8ec07c"),
  }
}

fn hex_to_rgb(hex: &str) -> Result<(u8, u8, u8), &'static str> {
  let hex = hex.trim_start_matches('#');

  if hex.len() != 6 {
    return Err("Invalid hex color format");
  }

  let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| "Invalid red component")?;
  let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| "Invalid green component")?;
  let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| "Invalid blue component")?;

  Ok((r, g, b))
}
