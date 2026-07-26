use serde::Deserialize;

/// mdr 项目配置，从项目根目录的 `.mdr/config.toml` 读取。
#[derive(Debug, Clone, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub server: ServerConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    /// 监听端口（默认 8080）
    #[serde(default = "default_port")]
    pub port: u16,

    /// 文档根目录（默认当前目录）
    #[serde(default = "default_document_root")]
    pub document_root: String,

    /// 站点标题
    #[serde(default)]
    pub title: String,

    /// 默认主题：dark 或 light
    #[serde(default = "default_theme")]
    pub theme: String,

    /// 忽略的文件（glob 模式）
    #[serde(default)]
    pub ignore: Vec<String>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            port: default_port(),
            document_root: default_document_root(),
            title: String::new(),
            theme: default_theme(),
            ignore: Vec::new(),
        }
    }
}

fn default_port() -> u16 { 8080 }
fn default_document_root() -> String { ".".to_string() }
fn default_theme() -> String { "dark".to_string() }

impl Config {
    /// 从项目目录加载 `.mdr/config.toml`，不存在则返回默认值。
    pub fn load(project_dir: &std::path::Path) -> Self {
        let config_path = project_dir.join(".mdr").join("config.toml");
        match std::fs::read_to_string(&config_path) {
            Ok(content) => {
                match toml::from_str::<Config>(&content) {
                    Ok(cfg) => {
                        eprintln!("  [config] 已加载: {}", config_path.display());
                        cfg
                    }
                    Err(e) => {
                        eprintln!("  [config] 解析失败 ({}): {}，使用默认配置", config_path.display(), e);
                        Config::default()
                    }
                }
            }
            Err(_) => {
                Config::default()
            }
        }
    }
}

/// 创建一个示例配置文件。
pub fn create_example(project_dir: &std::path::Path) -> std::io::Result<()> {
    let dir = project_dir.join(".mdr");
    std::fs::create_dir_all(&dir)?;
    let path = dir.join("config.toml");
    if !path.exists() {
        std::fs::write(&path, EXAMPLE)?;
        eprintln!("  [config] 已创建示例: {}", path.display());
    }
    Ok(())
}

const EXAMPLE: &str = r#"# mdr 项目配置
# 放在项目根目录的 .mdr/config.toml

[server]
# 监听端口
port = 8080

# 文档根目录
document_root = "."

# 站点标题（显示在顶栏）
title = "My Docs"

# 默认主题：dark 或 light
theme = "dark"

# 忽略的文件（glob 模式）
ignore = ["README.md", "*.exe", "*.ini"]
"#;
